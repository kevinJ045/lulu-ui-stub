use crate::shape::{self, LuaShape, from_lua_table};
use eframe::egui::*;
use eframe::egui::{self, Align2, FontId, ahash::HashMap};
use lulu::lulu::{Lulu, LuluModSource};
use mlua::{LuaSerdeExt, UserData, UserDataMethods};

pub fn color_from_lua_table(table: mlua::Table) -> Option<Color32> {
  let r: f32 = table.get(1).ok()?;
  let g: f32 = table.get(2).ok()?;
  let b: f32 = table.get(3).ok()?;
  let a: f32 = table.get(4).unwrap_or(1.0);
  Some(Color32::from_rgba_unmultiplied(
    (r * 255.0) as u8,
    (g * 255.0) as u8,
    (b * 255.0) as u8,
    (a * 255.0) as u8,
  ))
}

fn to_align(s: &str) -> egui::Align {
  match s {
    "start" => egui::Align::Min,
    "center" => egui::Align::Center,
    "end" => egui::Align::Max,
    _ => egui::Align::Min,
  }
}

macro_rules! get_size_attrib {
  ($ui:expr, $size:expr) => {
    if $size == "fill" {
      $ui.available_width()
    } else if $size.ends_with('%') {
      if let Ok(percent) = $size.trim_end_matches('%').parse::<f32>() {
        $ui.available_width() * (percent / 100.0)
      } else {
        0.0
      }
    } else if let Ok(px) = $size.parse::<f32>() {
      px
    } else {
      0.0
    }
  };
}

macro_rules! set_attrib {
  (($name:expr, $type:ty), $table:expr, $setter:expr) => {
    if let Ok(val) = $table.get::<$type>($name) {
      $setter(val);
    }
  };
}

macro_rules! is_color {
  ($table:expr, $setter:expr) => {
    if let Some(color) = color_from_lua_table($table) {
      $setter = color;
    }
  };
}

macro_rules! ui_resp {
  ($call:expr) => {
    Ok(LuaUiResponse {
      res: $call,
      value: None,
    })
  };
}

macro_rules! table_into_margin {
  ($val:expr) => {
    match $val {
      mlua::Value::Table(t) => {
        if t.len().unwrap() > 2 {
          Margin::symmetric(t.get(1).unwrap(), t.get(2).unwrap())
        } else {
          Margin {
            top: t.get(1).unwrap(),
            left: t.get(2).unwrap(),
            right: t.get(3).unwrap(),
            bottom: t.get(4).unwrap(),
          }
        }
      }
      mlua::Value::Number(n) => Margin::same(n as f32),
      _ => Margin::same(1.0),
    }
  };
}
macro_rules! table_into_rounding {
  ($val:expr) => {
    match $val {
      mlua::Value::Table(t) => Rounding {
        ne: t.get(1).unwrap(),
        nw: t.get(2).unwrap(),
        se: t.get(3).unwrap(),
        sw: t.get(4).unwrap(),
      },
      mlua::Value::Number(n) => Rounding::same(n as f32),
      _ => Rounding::same(1.0),
    }
  };
}

macro_rules! widget_style {
  ($state:ident, $table:expr, $style:expr) => {
    set_attrib!(
      (stringify!($state), mlua::Table),
      $table,
      |style_table: mlua::Table| {
        set_attrib!(("bg_fill", mlua::Table), style_table, |val: mlua::Table| {
          is_color!(val, $style.visuals.widgets.$state.bg_fill);
        });
        set_attrib!(
          ("weak_bg_fill", mlua::Table),
          style_table,
          |val: mlua::Table| {
            is_color!(val, $style.visuals.widgets.$state.weak_bg_fill);
          }
        );
        // set_attrib!( ("bg_stroke", mlua::Table), style_table, |val: mlua::Table| {
        // $style.visuals.widgets.$state.bg_stroke
        // });
        set_attrib!(
          ("rounding", mlua::Value),
          style_table,
          |val: mlua::Value| {
            $style.visuals.widgets.$state.rounding = table_into_rounding!(val);
          }
        );
      }
    );
  };
}

macro_rules! scoped_function_call {
  ($lua:expr, $ui:expr,  $func:expr) => {
    $lua
      .scope(|scope| {
        let lua_ui = scope.create_userdata(LuaUi { ui: $ui }).unwrap();
        let temp_func = scope
          .create_function(move |_lua, ()| $func.call::<()>(lua_ui.clone()))
          .unwrap();
        temp_func.call::<()>(()).unwrap();
        Ok(())
      })
      .unwrap();
  };
}

macro_rules! stylize_element {
  ($element:ident, $style:ident) => {
    if let Ok(color_tbl) = $style.get::<mlua::Table>("color") {
      $element = $element.fill(color_from_lua_table(color_tbl).unwrap());
    }

    if let Ok(stroke_tbl) = $style.get::<mlua::Table>("stroke") {
      let width: f32 = stroke_tbl.get(5).unwrap_or(1.0);
      $element = $element.stroke(Stroke::new(
        width,
        color_from_lua_table(stroke_tbl).unwrap(),
      ));
    }

    if let Ok(rounding) = $style.get::<mlua::Value>("rounding") {
      $element = $element.rounding(table_into_rounding!(rounding));
    }
  };
}

struct LuaVisuals(egui::Visuals);

impl UserData for LuaVisuals {
  fn add_methods<'lua, M: UserDataMethods<Self>>(methods: &mut M) {
    methods.add_method_mut("set_window_fill", |_, this, color: (f32, f32, f32, f32)| {
      let c = egui::Color32::from_rgba_unmultiplied(
        (color.0 * 255.0) as u8,
        (color.1 * 255.0) as u8,
        (color.2 * 255.0) as u8,
        (color.3 * 255.0) as u8,
      );
      this.0.window_fill = c;
      Ok(())
    });
    methods.add_method_mut("set_text_color", |_, this, color: (f32, f32, f32, f32)| {
      let c = egui::Color32::from_rgba_unmultiplied(
        (color.0 * 255.0) as u8,
        (color.1 * 255.0) as u8,
        (color.2 * 255.0) as u8,
        (color.3 * 255.0) as u8,
      );
      this.0.override_text_color = Some(c);
      Ok(())
    });

    // add more setters/getters as needed
  }
}

struct LuluUiApp {
  lulu: Lulu,
  init_error: Option<String>,
}

#[derive(Clone)]
struct LuaUiResponse {
  res: Response,
  value: Option<mlua::Value>,
}

impl UserData for LuaUiResponse {
  fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
    fields.add_field_method_get("value", |_, this| {
      Ok(match this.value.clone() {
        Some(v) => v,
        _ => mlua::Value::Nil,
      })
    });
    fields.add_field_method_get("clicked", |_, this| Ok(this.res.clicked()));
    fields.add_field_method_get("middle_clicked", |_, this| Ok(this.res.middle_clicked()));
    fields.add_field_method_get("double_clicked", |_, this| Ok(this.res.double_clicked()));
    fields.add_field_method_get("triple_clicked", |_, this| Ok(this.res.triple_clicked()));
    fields.add_field_method_get("clicked_elsewhere", |_, this| {
      Ok(this.res.clicked_elsewhere())
    });
    fields.add_field_method_get("lost_focus", |_, this| Ok(this.res.lost_focus()));
    fields.add_field_method_get("gained_focus", |_, this| Ok(this.res.gained_focus()));
    fields.add_field_method_get("has_focus", |_, this| Ok(this.res.has_focus()));
    fields.add_field_method_get("drag_delta", |_, this| {
      Ok(vec![this.res.drag_delta().x, this.res.drag_delta().y])
    });
    fields.add_field_method_get("hovered", |_, this| Ok(this.res.hovered()));
    fields.add_field_method_get("changed", |_, this| {
      if let Some(mlua::Value::Table(v)) = this.value.clone() {
        if let Ok(r) = v.get::<bool>("changed") {
          Ok(r)
        } else {
          Ok(this.res.changed())
        }
      } else {
        Ok(this.res.changed())
      }
    });
    fields.add_field_method_get("highlighted", |_, this| Ok(this.res.highlighted()));
    fields.add_field_method_get("contains_pointer", |_, this| {
      Ok(this.res.contains_pointer())
    });
    fields.add_field_method_get("long_touched", |_, this| Ok(this.res.long_touched()));
    fields.add_field_method_get("drag_started", |_, this| Ok(this.res.drag_started()));
    fields.add_field_method_get("drag_stopped", |_, this| Ok(this.res.drag_stopped()));
    fields.add_field_method_get("dragged", |_, this| Ok(this.res.dragged()));

    fields.add_field_method_get("interact_pointer_pos", |_, this| {
      Ok(this.res.interact_pointer_pos().map(|p| vec![p.x, p.y]))
    });
    fields.add_field_method_get("is_pointer_button_down_on", |_, this| {
      Ok(this.res.is_pointer_button_down_on())
    });
  }
}

struct LuaUi<'ui> {
  ui: &'ui mut egui::Ui,
}

impl<'ui> UserData for LuaUi<'ui> {
  fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
    methods.add_method_mut(
      "button",
      |_lua, this: &mut LuaUi, (text, style): (String, Option<mlua::Table>)| {
        let mut button = Button::new(text);

        if let Some(style_table) = style {
          stylize_element!(button, style_table);
        }

        let res: Response = this.ui.add(button);

        Ok(LuaUiResponse { res, value: None })
      },
    );
    methods.add_method_mut(
      "colored_label",
      |_lua, this: &mut LuaUi, (text, color): (String, mlua::Table)| {
        Ok(LuaUiResponse {
          res: this
            .ui
            .colored_label(color_from_lua_table(color).unwrap(), text),
          value: None,
        })
      },
    );
    methods.add_method_mut("label", |_lua, this: &mut LuaUi, text: String| {
      Ok(LuaUiResponse {
        res: this.ui.label(text),
        value: None,
      })
    });
    methods.add_method_mut("heading", |_lua, this: &mut LuaUi, text: String| {
      Ok(LuaUiResponse {
        res: this.ui.heading(text),
        value: None,
      })
    });
    methods.add_method_mut("small", |_lua, this: &mut LuaUi, text: String| {
      Ok(LuaUiResponse {
        res: this.ui.small(text),
        value: None,
      })
    });
    methods.add_method_mut("monospace", |_lua, this: &mut LuaUi, text: String| {
      Ok(LuaUiResponse {
        res: this.ui.monospace(text),
        value: None,
      })
    });
    methods.add_method_mut("strong", |_lua, this: &mut LuaUi, text: String| {
      Ok(LuaUiResponse {
        res: this.ui.strong(text),
        value: None,
      })
    });
    methods.add_method_mut("weak", |_lua, this: &mut LuaUi, text: String| {
      Ok(LuaUiResponse {
        res: this.ui.weak(text),
        value: None,
      })
    });
    methods.add_method_mut(
      "selectable_value",
      |_lua, this: &mut LuaUi, (mut current, selected, text): (String, String, String)| {
        Ok(LuaUiResponse {
          res: this.ui.selectable_value(&mut current, selected, text),
          value: None,
        })
      },
    );
    methods.add_method_mut(
      "text_edit_singleline",
      |lua, this: &mut LuaUi, text: String| {
        let mut value = text;
        let res = this.ui.text_edit_singleline(&mut value);

        let lua_value = lua.create_string(&value)?;

        let lua_response = lua.create_userdata(LuaUiResponse {
          res,
          value: Some(mlua::Value::String(lua_value)),
        })?;

        Ok(lua_response)
      },
    );

    methods.add_method_mut(
      "image",
      |_lua, this: &mut LuaUi, (source, options): (mlua::Value, Option<mlua::Table>)| {
        let mut img = match source {
          mlua::Value::UserData(ud) => {
            if let Ok(bytes) = ud.borrow::<lulu::ops::LuluByteArray>() {
              Image::from_bytes("lua_image", bytes.bytes.clone())
            } else {
              Image::new(include_image!("../assets/images/image-load-failed.png"))
            }
          }

          mlua::Value::String(s) => {
            let src = s.to_str().unwrap();

            if src.starts_with("http://") || src.starts_with("https://") {
              Image::from_uri(src.to_string())
            } else {
              if let Ok(bytes) = std::fs::read(src.to_string()) {
                Image::from_bytes(src.to_string(), bytes)
              } else {
                Image::new(include_image!("../assets/images/image-load-failed.png"))
              }
            }
          }

          _ => {
            eprintln!("Unsupported value passed to ui:image");
            Image::new(include_image!("../assets/images/image-load-failed.png"))
          }
        };

        if let Some(options) = options {
          if let Ok(size) = options.get::<f32>("fit_original") {
            img = img.fit_to_original_size(size);
          }
          if let Ok(maintain_aspect_ratio) = options.get::<bool>("maintain_aspect_ratio") {
            img = img.maintain_aspect_ratio(maintain_aspect_ratio);
          }
          if let Ok(fit) = options.get::<mlua::Table>("fit_to") {
            let size = Vec2::new(fit.get(1).unwrap(), fit.get(2).unwrap());
            img = img.fit_to_exact_size(size);
          }
          if let Ok(rotate) = options.get::<mlua::Table>("rotate") {
            let origin = Vec2::new(rotate.get(1).unwrap(), rotate.get(2).unwrap());
            img = img.rotate(rotate.get(3).unwrap(), origin);
          }
          if let Ok(rounding) = options.get::<mlua::Value>("rounding") {
            img = img.rounding(table_into_rounding!(rounding));
          }
          if let Ok(spinner) = options.get::<bool>("spinner") {
            img = img.show_loading_spinner(spinner);
          }
        }

        Ok(LuaUiResponse {
          res: this.ui.add(img),
          value: None,
        })
      },
    );

    methods.add_method_mut(
      "combobox",
      |lua,
       this: &mut LuaUi,
       (label, mut selected_key, values, func): (
        String,
        String,
        HashMap<String, String>,
        Option<mlua::Function>,
      )| {
        let selected_key_old = selected_key.clone();

        let mut keys: Vec<_> = values.keys().cloned().collect();
        keys.sort();

        let res = egui::ComboBox::from_label(label)
          .selected_text(values.get(&selected_key).unwrap_or(&selected_key))
          .show_ui(this.ui, |ui| {
            for key in keys {
              let value = values.get(&key).unwrap();
              if let Some(func) = func.clone() {
                let selected_key_old = selected_key_old.clone();
                let new_selected = lua
                  .scope(|_scope| {
                    let lua_ui = _scope.create_userdata(LuaUi { ui }).unwrap();
                    let temp_func = _scope
                      .create_function(move |_lua, selected_key_old: String| {
                        func.call::<Option<String>>((
                          lua_ui.clone(),
                          selected_key_old,
                          key.clone(),
                          value.clone(),
                        ))
                      })
                      .unwrap();
                    temp_func.call::<Option<String>>(selected_key_old)
                  })
                  .unwrap();
                if let Some(new_selected) = new_selected {
                  selected_key = new_selected
                }
              } else {
                ui.selectable_value(&mut selected_key, key.clone(), value);
              }
            }
          })
          .response;

        let value = if selected_key_old != selected_key {
          let table = lua.create_table()?;
          table.set("changed", true)?;
          table.set("__value", selected_key)?;
          Some(mlua::Value::Table(table))
        } else {
          None
        };

        let lua_response = lua.create_userdata(LuaUiResponse { res, value })?;

        Ok(lua_response)
      },
    );

    methods.add_method_mut(
      "text_edit_multiline",
      |lua, this: &mut LuaUi, text: String| {
        let mut value = text;
        let res = this.ui.text_edit_multiline(&mut value);

        let lua_value = lua.create_string(&value)?;

        let lua_response = lua.create_userdata(LuaUiResponse {
          res,
          value: Some(mlua::Value::String(lua_value)),
        })?;

        Ok(lua_response)
      },
    );

    methods.add_method_mut("code_editor", |lua, this: &mut LuaUi, text: String| {
      let mut value = text;
      let res = this.ui.code_editor(&mut value);

      let lua_value = lua.create_string(&value)?;

      let lua_response = lua.create_userdata(LuaUiResponse {
        res,
        value: Some(mlua::Value::String(lua_value)),
      })?;

      Ok(lua_response)
    });
    methods.add_method_mut(
      "checkbox",
      |_lua, this: &mut LuaUi, (text, checked): (String, bool)| {
        let mut value = checked;
        let response = this.ui.checkbox(&mut value, text);
        Ok(LuaUiResponse {
          res: response,
          value: Some(mlua::Value::Boolean(value)),
        })
      },
    );
    methods.add_method_mut(
      "slider",
      |_lua, this: &mut LuaUi, (text, min, max, value): (String, f32, f32, f32)| {
        let mut val = value;
        let response = this
          .ui
          .add(egui::Slider::new(&mut val, min..=max).text(text));
        Ok(LuaUiResponse {
          res: response,
          value: Some(mlua::Value::Number(val as f64)),
        })
      },
    );
    methods.add_method_mut(
      "drag_value",
      |_lua, this: &mut LuaUi, (text, value): (String, f64)| {
        let mut val = value;
        let response = this.ui.add(egui::DragValue::new(&mut val).prefix(text));
        Ok(LuaUiResponse {
          res: response,
          value: Some(mlua::Value::Number(val)),
        })
      },
    );
    methods.add_method_mut("hyperlink", |_lua, this: &mut LuaUi, url: String| {
      ui_resp!(this.ui.hyperlink(url))
    });
    methods.add_method_mut(
      "hyperlink_to",
      |_lua, this: &mut LuaUi, (text, url): (String, String)| {
        ui_resp!(this.ui.hyperlink_to(text, url))
      },
    );
    methods.add_method_mut("link", |_lua, this: &mut LuaUi, url: String| {
      ui_resp!(this.ui.link(url))
    });
    methods.add_method_mut("code", |_lua, this: &mut LuaUi, text: String| {
      ui_resp!(this.ui.code(text))
    });
    methods.add_method_mut(
      "radio_button",
      |_lua, this: &mut LuaUi, (text, current_value, my_value): (String, String, String)| {
        let mut current = current_value;
        let response = this.ui.radio_value(&mut current, my_value.clone(), text);
        Ok(LuaUiResponse {
          res: response,
          value: Some(mlua::Value::String(_lua.create_string(current)?)),
        })
      },
    );
    methods.add_method_mut("separator", |_lua, this: &mut LuaUi, ()| {
      ui_resp!(this.ui.separator())
    });
    methods.add_method_mut("spinner", |_lua, this: &mut LuaUi, ()| {
      ui_resp!(this.ui.spinner())
    });
    methods.add_method_mut(
      "progress_bar",
      |_lua, this: &mut LuaUi, (fraction, text): (f32, String)| {
        ui_resp!(this.ui.add(egui::ProgressBar::new(fraction).text(text)))
      },
    );
    methods.add_method_mut(
      "horizontal",
      |lua, this: &mut LuaUi, func: mlua::Function| {
        this.ui.horizontal(|ui| {
          lua
            .scope(|_scope| {
              let lua_ui = _scope.create_userdata(LuaUi { ui }).unwrap();
              let temp_func = _scope
                .create_function(move |_lua, ()| func.call::<()>(lua_ui.clone()))
                .unwrap();
              temp_func.call::<()>(()).unwrap();
              Ok(())
            })
            .unwrap();
        });
        Ok(())
      },
    );

    methods.add_method_mut("vertical", |lua, this: &mut LuaUi, func: mlua::Function| {
      this.ui.vertical(|ui| {
        lua
          .scope(|scope| {
            let lua_ui = scope.create_userdata(LuaUi { ui }).unwrap();
            let temp_func = scope
              .create_function(move |_lua, ()| func.call::<()>(lua_ui.clone()))
              .unwrap();
            temp_func.call::<()>(()).unwrap();
            Ok(())
          })
          .unwrap();
      });
      Ok(())
    });

    methods.add_method_mut(
      "horizontal_wrapped",
      |lua, this: &mut LuaUi, func: mlua::Function| {
        this.ui.horizontal_wrapped(|ui| {
          lua
            .scope(|scope| {
              let lua_ui = scope.create_userdata(LuaUi { ui }).unwrap();
              let temp_func = scope
                .create_function(move |_lua, ()| func.call::<()>(lua_ui.clone()))
                .unwrap();
              temp_func.call::<()>(()).unwrap();
              Ok(())
            })
            .unwrap();
        });
        Ok(())
      },
    );

    methods.add_method_mut(
      "vertical_centered",
      |lua, this: &mut LuaUi, func: mlua::Function| {
        this.ui.vertical_centered(|ui| {
          lua
            .scope(|scope| {
              let lua_ui = scope.create_userdata(LuaUi { ui }).unwrap();
              let temp_func = scope
                .create_function(move |_lua, ()| func.call::<()>(lua_ui.clone()))
                .unwrap();
              temp_func.call::<()>(()).unwrap();
              Ok(())
            })
            .unwrap();
        });
        Ok(())
      },
    );

    methods.add_method_mut(
      "vertical_centered_justified",
      |lua, this: &mut LuaUi, func: mlua::Function| {
        this.ui.vertical_centered_justified(|ui| {
          lua
            .scope(|scope| {
              let lua_ui = scope.create_userdata(LuaUi { ui }).unwrap();
              let temp_func = scope
                .create_function(move |_lua, ()| func.call::<()>(lua_ui.clone()))
                .unwrap();
              temp_func.call::<()>(()).unwrap();
              Ok(())
            })
            .unwrap();
        });
        Ok(())
      },
    );

    methods.add_method_mut("group", |lua, this: &mut LuaUi, func: mlua::Function| {
      this.ui.group(|ui| {
        lua
          .scope(|scope| {
            let lua_ui = scope.create_userdata(LuaUi { ui }).unwrap();
            let temp_func = scope
              .create_function(move |_lua, ()| func.call::<()>(lua_ui.clone()))
              .unwrap();
            temp_func.call::<()>(()).unwrap();
            Ok(())
          })
          .unwrap();
      });
      Ok(())
    });

    methods.add_method_mut(
      "scroll_area",
      |lua, this: &mut LuaUi, func: mlua::Function| {
        egui::ScrollArea::vertical().show(this.ui, |ui| {
          lua
            .scope(|scope| {
              let lua_ui = scope.create_userdata(LuaUi { ui }).unwrap();
              let temp_func = scope
                .create_function(move |_lua, ()| func.call::<()>(lua_ui.clone()))
                .unwrap();
              temp_func.call::<()>(()).unwrap();
              Ok(())
            })
            .unwrap();
        });
        Ok(())
      },
    );

    methods.add_method_mut("set_width", |_, this: &mut LuaUi, width: f32| {
      this.ui.set_width(width);
      Ok(())
    });

    methods.add_method_mut("set_height", |_, this: &mut LuaUi, height: f32| {
      this.ui.set_height(height);
      Ok(())
    });

    methods.add_method_mut("scope", |lua, this: &mut LuaUi, func: mlua::Function| {
      this.ui.scope(|ui| {
        lua
          .scope(|scope| {
            let lua_ui = scope.create_userdata(LuaUi { ui }).unwrap();
            let temp_func = scope
              .create_function(move |_lua, ()| func.call::<()>(lua_ui.clone()))
              .unwrap();
            temp_func.call::<()>(()).unwrap();
            Ok(())
          })
          .unwrap();
      });
      Ok(())
    });

    // methods.add_method_mut(
    //   "popup_below_widget",
    //   |lua, this: &mut LuaUi, (id, func): (String, mlua::Function)| {
    //     egui::popup_below_widget(
    //       this.ui,
    //       egui::Id::new(id),
    //       &this.ui.available_rect_before_wrap(),
    //       |ui| {
    //         lua
    //           .scope(|scope| {
    //             let lua_ui = scope.create_userdata(LuaUi { ui }).unwrap();
    //             let temp_func = scope
    //               .create_function(move |_lua, ()| func.call::<()>(lua_ui.clone()))
    //               .unwrap();
    //             temp_func.call::<()>(()).unwrap();
    //             Ok(())
    //           })
    //           .unwrap();
    //       },
    //     );
    //     Ok(())
    //   },
    // );

    methods.add_method_mut(
      "window",
      |lua, this: &mut LuaUi, (title, func): (String, mlua::Function)| {
        egui::Window::new(title).show(this.ui.ctx(), |ui| {
          lua
            .scope(|scope| {
              let lua_ui = scope.create_userdata(LuaUi { ui }).unwrap();
              let temp_func = scope
                .create_function(move |_lua, ()| func.call::<()>(lua_ui.clone()))
                .unwrap();
              temp_func.call::<()>(()).unwrap();
              Ok(())
            })
            .unwrap();
        });
        Ok(())
      },
    );

    methods.add_method_mut(
      "color_picker",
      |_lua, this: &mut LuaUi, color_table: mlua::Table| {
        let mut color = color_from_lua_table(color_table).unwrap();
        let response = egui::widgets::color_picker::color_edit_button_srgba(
          this.ui,
          &mut color,
          egui::color_picker::Alpha::Opaque,
        );
        Ok(LuaUiResponse {
          res: response,
          value: Some(mlua::Value::Table(_lua.create_table_from([
            (1, color.r()),
            (2, color.g()),
            (3, color.b()),
            (4, color.a()),
          ])?)),
        })
      },
    );

    methods.add_method_mut(
      "color_edit_button",
      |_, this: &mut LuaUi, (r, g, b): (f32, f32, f32)| {
        let mut color =
          egui::Color32::from_rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8);
        this.ui.color_edit_button_srgba(&mut color);
        Ok(())
      },
    );

    methods.add_method_mut(
      "grid",
      |lua, this: &mut LuaUi, (id, func): (String, mlua::Function)| {
        egui::Grid::new(id).show(this.ui, |ui| {
          lua
            .scope(|scope| {
              let lua_ui = scope.create_userdata(LuaUi { ui }).unwrap();
              let temp_func = scope
                .create_function(move |_lua, ()| func.call::<()>(lua_ui.clone()))
                .unwrap();
              temp_func.call::<()>(()).unwrap();
              Ok(())
            })
            .unwrap();
        });
        Ok(())
      },
    );

    methods.add_method_mut(
      "collapsing_header",
      |lua, this: &mut LuaUi, (label, func): (String, mlua::Function)| {
        this.ui.collapsing(label, |ui| {
          lua
            .scope(|scope| {
              let lua_ui = scope.create_userdata(LuaUi { ui }).unwrap();
              let temp_func = scope
                .create_function(move |_lua, ()| func.call::<()>(lua_ui.clone()))
                .unwrap();
              temp_func.call::<()>(()).unwrap();
              Ok(())
            })
            .unwrap();
        });
        Ok(())
      },
    );

    // methods.add_method_mut("table", |lua, this: &mut LuaUi, (id, func): (String, LuaFunction)| {
    //   let mut table = egui_extras::TableBuilder::new(this.ui)
    //     .striped(true)
    //     .resizable(true)
    //     .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
    //     .column(egui_extras::Column::auto().resizable(true));

    //   table.body(|body| {
    //     body.rows(18.0, 1, |mut row| {
    //       row.col(|ui| {
    //         lua
    //           .scope(|scope| {
    //             let lua_ui = scope.create_userdata(LuaUi { ui }).unwrap();
    //             let temp_func = scope
    //               .create_function(move |_lua, ()| func.call::<()>(lua_ui.clone()))
    //               .unwrap();
    //             temp_func.call::<()>(()).unwrap();
    //             Ok(())
    //           })
    //           .unwrap();
    //       });
    //     });
    //   });
    //   Ok(())
    // });

    methods.add_method_mut(
      "set_attribs",
      |_, this: &mut LuaUi, style_table: mlua::Table| {
        set_attrib!(("height", String), style_table, |val: String| {
          this.ui.set_height(get_size_attrib!(this.ui, val))
        });
        set_attrib!(("width", String), style_table, |val: String| {
          this.ui.set_width(get_size_attrib!(this.ui, val))
        });

        set_attrib!(("visible", bool), style_table, |val: bool| {
          this.ui.set_visible(val);
        });

        Ok(())
      },
    );

    methods.add_method_mut(
      "set_style",
      |_, this: &mut LuaUi, (style_table, context): (mlua::Table, bool)| {
        let ctx = this.ui.ctx();
        let mut style = (*ctx.style()).clone();

        if let Ok(wrap) = style_table.get::<bool>("wrap") {
          style.wrap = Some(wrap);
        }

        set_attrib!(
          ("spacing", mlua::Table),
          style_table,
          |style_table: mlua::Table| {
            set_attrib!(
              ("item_spacing", mlua::Table),
              style_table,
              |val: mlua::Table| {
                style.spacing.item_spacing =
                  Vec2::new(val.get(1).unwrap_or(1.0), val.get(2).unwrap_or(1.0))
              }
            );

            set_attrib!(
              ("button_padding", mlua::Table),
              style_table,
              |val: mlua::Table| {
                style.spacing.button_padding =
                  Vec2::new(val.get(1).unwrap_or(1.0), val.get(2).unwrap_or(1.0))
              }
            );

            set_attrib!(
              ("interact_size", mlua::Table),
              style_table,
              |val: mlua::Table| {
                style.spacing.interact_size =
                  Vec2::new(val.get(1).unwrap_or(1.0), val.get(2).unwrap_or(1.0))
              }
            );

            set_attrib!(
              ("menu_margin", mlua::Value),
              style_table,
              |val: mlua::Value| { style.spacing.menu_margin = table_into_margin!(val) }
            );

            set_attrib!(("indent", f32), style_table, |val: f32| style
              .spacing
              .indent = val);

            set_attrib!(("slider_width", f32), style_table, |val: f32| style
              .spacing
              .slider_width =
              val);

            set_attrib!(("combo_width", f32), style_table, |val: f32| style
              .spacing
              .combo_width =
              val);

            set_attrib!(("text_edit_width", f32), style_table, |val: f32| style
              .spacing
              .text_edit_width =
              val);

            set_attrib!(("icon_width", f32), style_table, |val: f32| style
              .spacing
              .icon_width =
              val);

            set_attrib!(("icon_width_inner", f32), style_table, |val: f32| style
              .spacing
              .icon_width_inner =
              val);

            set_attrib!(("icon_spacing", f32), style_table, |val: f32| style
              .spacing
              .icon_spacing =
              val);

            set_attrib!(("tooltip_width", f32), style_table, |val: f32| style
              .spacing
              .tooltip_width =
              val);

            set_attrib!(("combo_height", f32), style_table, |val: f32| style
              .spacing
              .combo_height =
              val);
          }
        );

        set_attrib!(
          ("visuals", mlua::Table),
          style_table,
          |style_table: mlua::Table| {
            set_attrib!(("dark_mode", bool), style_table, |val: bool| style
              .visuals
              .dark_mode =
              val);

            widget_style!(noninteractive, style_table, style);
            widget_style!(inactive, style_table, style);
            widget_style!(hovered, style_table, style);
            widget_style!(active, style_table, style);
            widget_style!(open, style_table, style);

            set_attrib!(
              ("hyperlink_color", mlua::Table),
              style_table,
              |val: mlua::Table| is_color!(val, style.visuals.hyperlink_color)
            );

            set_attrib!(
              ("faint_bg_color", mlua::Table),
              style_table,
              |val: mlua::Table| is_color!(val, style.visuals.faint_bg_color)
            );

            set_attrib!(
              ("extreme_bg_color", mlua::Table),
              style_table,
              |val: mlua::Table| is_color!(val, style.visuals.extreme_bg_color)
            );

            set_attrib!(
              ("code_bg_color", mlua::Table),
              style_table,
              |val: mlua::Table| is_color!(val, style.visuals.code_bg_color)
            );

            set_attrib!(
              ("warn_fg_color", mlua::Table),
              style_table,
              |val: mlua::Table| is_color!(val, style.visuals.warn_fg_color)
            );

            set_attrib!(
              ("error_fg_color", mlua::Table),
              style_table,
              |val: mlua::Table| is_color!(val, style.visuals.error_fg_color)
            );

            set_attrib!(
              ("window_rounding", mlua::Value),
              style_table,
              |val: mlua::Value| style.visuals.window_rounding = table_into_rounding!(val)
            );

            set_attrib!(
              ("window_fill", mlua::Table),
              style_table,
              |val: mlua::Table| is_color!(val, style.visuals.window_fill)
            );

            set_attrib!(
              ("panel_fill", mlua::Table),
              style_table,
              |val: mlua::Table| is_color!(val, style.visuals.panel_fill)
            );

            if let Ok(text_table) = style_table.get::<mlua::Table>("text_color") {
              if let Some(color) = color_from_lua_table(text_table) {
                style.visuals.override_text_color = Some(color);
              }
            }
          }
        );

        if context {
          ctx.set_style(style);
        } else {
          this.ui.set_style(style);
        }

        Ok(())
      },
    );

    methods.add_method_mut("set_spacing", |_, this: &mut LuaUi, spacing: f32| {
      let ctx = this.ui.ctx();
      let mut style = (*ctx.style()).clone();

      style.spacing.item_spacing = egui::vec2(spacing, spacing);
      this.ui.set_style(style);
      Ok(())
    });

    methods.add_method_mut(
      "columns",
      |lua, this: &mut LuaUi, (n, func): (usize, mlua::Function)| {
        this.ui.columns(n, |columns| {
          for (i, column) in columns.iter_mut().enumerate() {
            let func = func.clone(); // clone here
            lua
              .scope(|scope| {
                let lua_ui = scope.create_userdata(LuaUi { ui: column }).unwrap();
                let temp_func = scope
                  .create_function(move |_lua, ()| func.call::<()>((i + 1, lua_ui.clone())))
                  .unwrap();
                temp_func.call::<()>(()).unwrap();
                Ok(())
              })
              .unwrap();
          }
        });
        Ok(())
      },
    );

    // allocate_ui_with_layout
    methods.add_method_mut(
      "allocate_ui_with_layout",
      |lua, this: &mut LuaUi, (layout_name, func): (String, mlua::Function)| {
        let layout = match layout_name.as_str() {
          "left_to_right" => Layout::left_to_right(Align::Center),
          "right_to_left" => Layout::right_to_left(Align::Center),
          "top_down" => Layout::top_down(Align::Center),
          "bottom_up" => Layout::bottom_up(Align::Center),
          _ => Layout::default(),
        };
        this
          .ui
          .allocate_ui_with_layout(egui::Vec2::INFINITY, layout, |ui| {
            lua
              .scope(|scope| {
                let lua_ui = scope.create_userdata(LuaUi { ui }).unwrap();
                let temp_func = scope
                  .create_function(move |_lua, ()| func.call::<()>(lua_ui.clone()))
                  .unwrap();
                temp_func.call::<()>(()).unwrap();
                Ok(())
              })
              .unwrap();
          });
        Ok(())
      },
    );

    methods.add_method_mut(
      "visuals_mut",
      |_, this: &mut LuaUi, func: mlua::Function| {
        let visuals = LuaVisuals(this.ui.visuals_mut().clone());
        func.call::<()>(visuals)?;
        Ok(())
      },
    );

    // menu_button
    methods.add_method_mut(
      "menu_button",
      |lua, this: &mut LuaUi, (label, func): (String, mlua::Function)| {
        this.ui.menu_button(label, |ui| {
          lua
            .scope(|scope| {
              let lua_ui = scope.create_userdata(LuaUi { ui }).unwrap();
              let temp_func = scope
                .create_function(move |_lua, ()| func.call::<()>(lua_ui.clone()))
                .unwrap();
              temp_func.call::<()>(()).unwrap();
              Ok(())
            })
            .unwrap();
        });
        Ok(())
      },
    );

    methods.add_method_mut(
      "frame_block",
      |lua, this: &mut LuaUi, (style, func): (mlua::Table, mlua::Function)| {
        let mut frame = Frame::none();

        stylize_element!(frame, style);

        if let Ok(padding) = style.get::<mlua::Value>("padding") {
          frame = frame.inner_margin(table_into_margin!(padding));
        }

        let min_size = if let Ok(size_tbl) = style.get::<mlua::Table>("min_size") {
          let w: f32 = size_tbl.get(1).unwrap_or(0.0);
          let h: f32 = size_tbl.get(2).unwrap_or(0.0);
          Vec2::new(w, h)
        } else {
          Vec2::ZERO
        };

        frame.show(this.ui, |ui| {
          if min_size != Vec2::ZERO {
            ui.set_min_size(min_size);
          } else {
            if let Ok(w) = style.get::<String>("width") {
              ui.set_min_width(get_size_attrib!(ui, w));
            }

            if let Ok(h) = style.get::<String>("height") {
              ui.set_min_width(get_size_attrib!(ui, h));
            }
          }

          lua
            .scope(|scope| {
              let lua_ui = scope.create_userdata(LuaUi { ui }).unwrap();
              let temp_func = scope
                .create_function(move |_lua, ()| func.call::<()>(lua_ui.clone()))
                .unwrap();
              temp_func.call::<()>(()).unwrap();
              Ok(())
            })
            .unwrap();
        });

        Ok(())
      },
    );

    methods.add_method_mut(
      "align",
      |lua, this: &mut LuaUi, (layout_name, align, func): (String, String, mlua::Function)| {
        let alignment = to_align(&align);
        let layout = match layout_name.as_str() {
          "left_to_right" => Layout::left_to_right(alignment),
          "right_to_left" => Layout::right_to_left(alignment),
          "top_down" => Layout::top_down(alignment),
          "bottom_up" => Layout::bottom_up(alignment),
          _ => Layout::default(),
        };
        if layout_name.as_str() == "center_both" {
          this.ui.centered_and_justified(|ui| {
            scoped_function_call!(lua, ui, func);
          });
        } else {
          this.ui.with_layout(layout, |ui| {
            scoped_function_call!(lua, ui, func);
          });
        }
        Ok(())
      },
    );

    // context_menu
    // methods.add_method_mut("context_menu", |lua, this: &mut LuaUi, func: mlua::Function| {
    //   this.ui.context_menu(|ui| {
    //     let lua_ui = LuaUi { ui };
    //     func.call::<()>(lua_ui).unwrap();
    //   });
    //   Ok(())
    // });

    methods.add_method_mut(
      "place_ui_at",
      |lua, this: &mut LuaUi, (x, y, w, h, func): (f32, f32, f32, f32, mlua::Function)| {
        let rect = Rect::from_min_size(egui::pos2(x, y), Vec2::new(w, h));
        this.ui.allocate_ui(rect.size(), |ui| {
          ui.set_clip_rect(rect); // optional
          lua
            .scope(|scope| {
              let lua_ui = scope.create_userdata(LuaUi { ui }).unwrap();
              let temp_func = scope
                .create_function(move |_lua, ()| func.call::<()>(lua_ui.clone()))
                .unwrap();
              temp_func.call::<()>(()).unwrap();
              Ok(())
            })
            .unwrap();
        });
        Ok(())
      },
    );

    // clip_rect
    methods.add_method_mut("clip_rect", |_, this: &mut LuaUi, ()| {
      let rect = this.ui.clip_rect();
      Ok((rect.min.x, rect.min.y, rect.max.x, rect.max.y))
    });

    methods.add_method_mut("painter", |_, this: &mut LuaUi, ()| {
      let painter = this.ui.painter().clone();
      Ok(LuaPainter { painter })
    });
  }
}

#[derive(Clone)]
struct LuaPainter {
  painter: Painter,
}

impl UserData for LuaPainter {
  fn add_methods<'lua, M: UserDataMethods<Self>>(methods: &mut M) {
    methods.add_method_mut(
      "rect_filled",
      |_, this: &mut LuaPainter, (x, y, w, h, color): (f32, f32, f32, f32, mlua::Table)| {
        let rect = Rect::from_min_size(egui::pos2(x, y), Vec2::new(w, h));
        let color = color_from_lua_table(color).unwrap();
        this.painter.rect_filled(rect, 0.0, color);
        Ok(())
      },
    );

    methods.add_method_mut(
      "circle_filled",
      |_, this: &mut LuaPainter, (x, y, radius, color): (f32, f32, f32, mlua::Table)| {
        let center = egui::pos2(x, y);
        let color = color_from_lua_table(color).unwrap();
        this.painter.circle_filled(center, radius, color);
        Ok(())
      },
    );

    methods.add_method_mut(
      "line_segment",
      |_,
       this: &mut LuaPainter,
       (x1, y1, x2, y2, color, width): (f32, f32, f32, f32, mlua::Table, f32)| {
        let points = [egui::pos2(x1, y1), egui::pos2(x2, y2)];
        let color = color_from_lua_table(color).unwrap();
        this.painter.line_segment(points, Stroke::new(width, color));
        Ok(())
      },
    );

    methods.add_method_mut(
      "circle_stroke",
      |_,
       this: &mut LuaPainter,
       (x, y, radius, color, width): (f32, f32, f32, mlua::Table, f32)| {
        let center = egui::pos2(x, y);
        let color = color_from_lua_table(color).unwrap();
        this
          .painter
          .circle_stroke(center, radius, Stroke::new(width, color));
        Ok(())
      },
    );

    methods.add_method_mut(
      "rect_stroke",
      |_,
       this: &mut LuaPainter,
       (x, y, w, h, color, width): (f32, f32, f32, f32, mlua::Table, f32)| {
        let rect = Rect::from_min_size(egui::pos2(x, y), Vec2::new(w, h));
        let color = color_from_lua_table(color).unwrap();
        this
          .painter
          .rect_stroke(rect, 0.0, Stroke::new(width, color));
        Ok(())
      },
    );
    
    // methods.add_method_mut(
    //   "image",
    //   |_,
    //    this: &mut LuaPainter,
    //    (): ()| {
    //     this
    //       .painter
    //       .image(rect, 0.0, Stroke::new(width, color));
    //     Ok(())
    //   },
    // );

    methods.add_method_mut(
      "update",
      |_,
       _: &mut LuaPainter,
       ()| {
        Ok(())
      },
    );

    methods.add_method_mut(
      "text",
      |_,
       this: &mut LuaPainter,
       (x, y, text, font_size, color): (f32, f32, String, f32, mlua::Table)| {
        let pos = egui::pos2(x, y);
        let color = color_from_lua_table(color).unwrap();
        this.painter.text(
          pos,
          Align2::LEFT_TOP,
          text,
          FontId::proportional(font_size),
          color,
        );
        Ok(())
      },
    );

    methods.add_method_mut(
      "arrow",
      |_,
       this: &mut LuaPainter,
       (x, y, dx, dy, color, width): (f32, f32, f32, f32, mlua::Table, f32)| {
        let origin = egui::pos2(x, y);
        let vec = egui::vec2(dx, dy);
        let color = color_from_lua_table(color).unwrap();
        this.painter.arrow(origin, vec, Stroke::new(width, color));
        Ok(())
      },
    );

    methods.add_method_mut(
      "add_shape_from",
      |_, this: &mut LuaPainter, table: mlua::Table| {
        if let Some(shape) = shape::from_lua_table(table) {
          this.painter.add(shape);
        }
        Ok(())
      },
    );

    methods.add_method_mut(
      "add_shape",
      |_, this: &mut LuaPainter, shape: mlua::AnyUserData| {
        let shape = shape.borrow::<LuaShape>().unwrap();
        this.painter.extend(vec![shape.shape.clone()]);
        Ok(())
      },
    );

    methods.add_method_mut(
      "extend_shapes_from",
      |_, this: &mut LuaPainter, tables: mlua::Table| {
        let mut shapes = Vec::new();
        for table in tables.sequence_values::<mlua::Table>() {
          if let Ok(table) = table {
            if let Some(shape) = shape::from_lua_table(table) {
              shapes.push(shape);
            }
          }
        }
        this.painter.extend(shapes);
        Ok(())
      },
    );

    methods.add_method_mut(
      "extend_shapes",
      |_, this: &mut LuaPainter, tables: mlua::Table| {
        let mut shapes = Vec::new();
        for shape in tables.sequence_values::<mlua::AnyUserData>() {
          if let Ok(shape) = shape {
            let shape = shape.borrow::<LuaShape>().unwrap();
            shapes.push(shape.shape.clone())
          }
        }
        this.painter.extend(shapes);
        Ok(())
      },
    );
  }
}

impl LuluUiApp {
  fn new(
    _cc: &eframe::CreationContext<'_>,
    lulu: Lulu,
    main: Option<mlua::Value>,
    err: Option<String>,
  ) -> Self {
    egui_extras::install_image_loaders(&_cc.egui_ctx);
    lulu.lua.set_app_data(_cc.egui_ctx.clone());

    lulu
      .lua
      .globals()
      .set(
        "Shape2D",
        lulu
          .lua
          .create_function(|_, tab: mlua::Table| {
            Ok(LuaShape {
              shape: from_lua_table(tab).unwrap(),
            })
          })
          .unwrap(),
      )
      .unwrap();

    lulu
      .lua
      .globals()
      .set(
        "register_font",
        lulu
          .lua
          .create_function(|lua, (name, font_bytes): (String, mlua::String)| {
            let ctx: &egui::Context = &*lua
              .app_data_ref::<egui::Context>()
              .expect("egui context not found");
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
              name.clone(),
              egui::FontData::from_owned(font_bytes.as_bytes().to_vec()),
            );
            fonts
              .families
              .entry(egui::FontFamily::Proportional)
              .or_default()
              .insert(0, name.clone());
            fonts
              .families
              .entry(egui::FontFamily::Monospace)
              .or_default()
              .insert(0, name.clone());
            ctx.set_fonts(fonts);
            Ok(())
          })
          .unwrap(),
      )
      .unwrap();

    let mut init_error = err;

    if let Some(main) = main {
      match main {
        mlua::Value::Function(f) => match f.call::<mlua::Value>(()) {
          Ok(v) => match v {
            mlua::Value::Function(f) => match lulu.lua.globals().set("ui_update", f) {
              _ => {}
            },
            _ => {}
          },
          Err(e) => {
            init_error = Some(e.to_string());
          }
        },
        _ => {
          init_error = Some("Expected main to return init function, recieved nothing".to_string());
        }
      }
    }

    Self { lulu, init_error }
  }
}

impl eframe::App for LuluUiApp {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // Changed frame back to _frame
    self.lulu.lua.set_app_data(ctx.clone());
    match self
      .lulu
      .lua
      .globals()
      .set("cpu_usage", _frame.info().cpu_usage.unwrap_or(0.0))
    {
      Err(err) => {
        self.init_error = Some(err.to_string());
      }
      Ok(_) => {}
    }

    if let Some(err) = &self.init_error {
      egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Lua Initialization Error:");
        ui.label(err.to_string());
      });
    } else if let Ok(render_fn) = self.lulu.lua.globals().get::<mlua::Function>("ui_update") {
      egui::CentralPanel::default().show(ctx, |ui| {
        match self.lulu.lua.scope(|_scope| {
          let lua_ui = _scope.create_userdata(LuaUi { ui }).unwrap();
          let temp_func = _scope
            .create_function(move |_lua, ()| render_fn.call::<()>(lua_ui.clone()))
            .unwrap();
          temp_func.call::<()>(())?;
          Ok(())
        }) {
          Err(err) => {
            ui.heading("Lua Initialization Error:");
            ui.label(err.to_string());
          }
          Ok(_) => {}
        }
      });
    }

    let scheduler: mlua::Function = self
      .lulu
      .lua
      .globals()
      .get::<mlua::Table>("coroutine")
      .unwrap()
      .get("resume")
      .unwrap();
    let sched_co: mlua::Value = self
      .lulu
      .lua
      .globals()
      .get::<mlua::Table>("Future")
      .unwrap()
      .get("scheduler")
      .unwrap();

    scheduler.call::<mlua::Value>(sched_co.clone()).unwrap();
  }
}

async fn load_main(lulu: &mut Lulu) -> Result<mlua::Value, String> {
  lulu.preload_mods().map_err(|e| e.to_string())?;

  let main_name = lulu.find_mod("main").map_err(|e| e.to_string())?;

  let ui_code = std::fs::read_to_string("src/lua/ui.lua").map_err(|e| e.to_string())?;
  // let ui_code = include_str!("lua/ui.lua");
  let f = lulu
    .lua
    .load(lulu.compiler.compile(&ui_code, None, None))
    .set_name("ui.lua")
    .eval()
    .map_err(|e| e.to_string())?;

  lulu.exec_mod(&main_name).map_err(|e| e.to_string())?;
  Ok(f)
}

pub async fn run(lulu: &mut Lulu) -> Result<(), eframe::Error> {
  let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
    ..Default::default()
  };

  let mut err: Option<String> = None;

  let main = match load_main(lulu).await {
    Ok(f) => Some(f),
    Err(e) => {
      err = Some(e);
      None
    }
  };

  let title = if let Ok(modname) = lulu.find_mod("ui-title") {
    match lulu.mods.iter().find(|m| m.name == modname) {
      Some(m) => match m.source.clone() {
        LuluModSource::Bytecode(bytes) => String::from_utf8(bytes).unwrap(),
        LuluModSource::Code(_) => match lulu.exec_mod(&modname) {
          Ok(value) => lulu.lua.from_value::<String>(value).unwrap(),
          Err(_) => "Lulu UI".to_string(),
        },
      },
      _ => "Lulu UI".to_string(),
    }
  } else {
    "Lulu UI".to_string()
  };

  let lulu = lulu.clone();

  eframe::run_native(
    &title,
    options,
    Box::new(|cc| {
      let mut fonts = egui::FontDefinitions::default();
      fonts.font_data.insert(
        "DejaVuSansMono".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/DejaVuSansMono.ttf")),
      );
      fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "DejaVuSansMono".to_owned());
      fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "DejaVuSansMono".to_owned());
      cc.egui_ctx.set_fonts(fonts);

      Box::new(LuluUiApp::new(cc, lulu, main, err))
    }),
  )
}
