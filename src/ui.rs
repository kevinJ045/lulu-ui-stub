use eframe::egui;
use eframe::egui::*;
use lulu::lulu::Lulu;
use mlua::{UserData, UserDataMethods};

fn color_from_lua_table(table: mlua::Table) -> Option<Color32> {
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
      mlua::Value::Number(n) => {
        Margin::same(n as f32)
      }
      _ => {
        Margin::same(1.0)
      }
    }
  };
}
macro_rules! table_into_rounding {
  ($val:expr) => {
    match $val {
      mlua::Value::Table(t) => {
        Rounding {
          ne: t.get(1).unwrap(),
          nw: t.get(2).unwrap(),
          se: t.get(3).unwrap(),
          sw: t.get(4).unwrap(),
        }
      }
      mlua::Value::Number(n) => {
        Rounding::same(n as f32)
      }
      _ => {
        Rounding::same(1.0)
      }
    }
  };
}

macro_rules! widget_style {
  ($state:ident, $table:expr, $style:expr) => {
    set_attrib!( (stringify!($state), mlua::Table), $table, |style_table: mlua::Table| {
      set_attrib!( ("bg_fill", mlua::Table), style_table, |val: mlua::Table| {
        is_color!(val, $style.visuals.widgets.$state.bg_fill);
      });
      set_attrib!( ("weak_bg_fill", mlua::Table), style_table, |val: mlua::Table| {
        is_color!(val, $style.visuals.widgets.$state.weak_bg_fill);
      });
      // set_attrib!( ("bg_stroke", mlua::Table), style_table, |val: mlua::Table| {
        // $style.visuals.widgets.$state.bg_stroke
      // });
      set_attrib!( ("rounding", mlua::Value), style_table, |val: mlua::Value| {
        $style.visuals.widgets.$state.rounding = table_into_rounding!(val);
      });
    });
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
    fields.add_field_method_get("clicked_elsewhere", |_, this| Ok(this.res.clicked_elsewhere()));
    fields.add_field_method_get("lost_focus", |_, this| Ok(this.res.lost_focus()));
    fields.add_field_method_get("gained_focus", |_, this| Ok(this.res.gained_focus()));
    fields.add_field_method_get("has_focus", |_, this| Ok(this.res.has_focus()));
    fields.add_field_method_get("drag_delta", |_, this| Ok(vec![this.res.drag_delta().x, this.res.drag_delta().y]));
    fields.add_field_method_get("hovered", |_, this| Ok(this.res.hovered()));
    fields.add_field_method_get("changed", |_, this| Ok(this.res.changed()));
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
  fn add_methods<'lua, M: UserDataMethods<Self>>(methods: &mut M) {
    methods.add_method_mut("button", |_lua, this, (text, style) : (String, Option<mlua::Table>)| {
      let mut button = Button::new(text);

      if let Some(style_table) = style {
        stylize_element!(button, style_table);
      }

      let res: Response = this.ui.add(button);
      
      Ok(LuaUiResponse { res, value: None })
    });
    methods.add_method_mut("label", |_lua, this, text: String| {
      Ok(LuaUiResponse {
        res: this.ui.label(text),
        value: None,
      })
    });
    methods.add_method_mut("text_edit_singleline", |lua, this, text: String| {
      let mut value = text;
      let res = this.ui.text_edit_singleline(&mut value);

      let lua_value = lua.create_string(&value)?;

      let lua_response = lua.create_userdata(LuaUiResponse {
        res,
        value: Some(mlua::Value::String(lua_value)),
      })?;

      Ok(lua_response)
    });

    methods.add_method_mut("text_edit_multiline", |lua, this, text: String| {
      let mut value = text;
      let res = this.ui.text_edit_multiline(&mut value);

      let lua_value = lua.create_string(&value)?;

      let lua_response = lua.create_userdata(LuaUiResponse {
        res,
        value: Some(mlua::Value::String(lua_value)),
      })?;

      Ok(lua_response)
    });
    methods.add_method_mut("checkbox", |_lua, this, (text, checked): (String, bool)| {
      let mut value = checked;
      let response = this.ui.checkbox(&mut value, text);
      Ok(LuaUiResponse {
        res: response,
        value: Some(mlua::Value::Boolean(value)),
      })
    });
    methods.add_method_mut(
      "slider",
      |_lua, this, (text, min, max, value): (String, f32, f32, f32)| {
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
    methods.add_method_mut("drag_value", |_lua, this, (text, value): (String, f64)| {
      let mut val = value;
      let response = this.ui.add(egui::DragValue::new(&mut val).prefix(text));
      Ok(LuaUiResponse {
        res: response,
        value: Some(mlua::Value::Number(val)),
      })
    });
    methods.add_method_mut("hyperlink", |_lua, this, url: String| {
      ui_resp!(this.ui.hyperlink(url))
    });
    methods.add_method_mut(
      "radio_button",
      |_lua, this, (text, current_value, my_value): (String, String, String)| {
        let mut current = current_value;
        let response = this.ui.radio_value(&mut current, my_value.clone(), text);
        Ok(LuaUiResponse {
          res: response,
          value: Some(mlua::Value::String(_lua.create_string(current)?)),
        })
      },
    );
    methods.add_method_mut("separator", |_lua, this, ()| ui_resp!(this.ui.separator()));
    methods.add_method_mut("spinner", |_lua, this, ()| ui_resp!(this.ui.spinner()));
    methods.add_method_mut(
      "progress_bar",
      |_lua, this, (fraction, text): (f32, String)| {
        ui_resp!(this.ui.add(egui::ProgressBar::new(fraction).text(text)))
      },
    );
    methods.add_method_mut("horizontal", |lua, this, func: mlua::Function| {
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
    });

    methods.add_method_mut("vertical", |lua, this, func: mlua::Function| {
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

    methods.add_method_mut("horizontal_wrapped", |lua, this, func: mlua::Function| {
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
    });

    methods.add_method_mut("vertical_centered", |lua, this, func: mlua::Function| {
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
    });

    methods.add_method_mut(
      "vertical_centered_justified",
      |lua, this, func: mlua::Function| {
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

    methods.add_method_mut("group", |lua, this, func: mlua::Function| {
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

    methods.add_method_mut("scroll_area", |lua, this, func: mlua::Function| {
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
    });

    methods.add_method_mut("set_width", |_, this, width: f32| {
      this.ui.set_width(width);
      Ok(())
    });

    methods.add_method_mut("set_height", |_, this, height: f32| {
      this.ui.set_height(height);
      Ok(())
    });

    methods.add_method_mut("scope", |lua, this, func: mlua::Function| {
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
    //   |lua, this, (id, func): (String, mlua::Function)| {
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
      |lua, this, (title, func): (String, mlua::Function)| {
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

    methods.add_method_mut("color_picker", |_lua, this, color_table: mlua::Table| {
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
    });

    methods.add_method_mut(
      "color_edit_button",
      |_, this, (r, g, b): (f32, f32, f32)| {
        let mut color =
          egui::Color32::from_rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8);
        this.ui.color_edit_button_srgba(&mut color);
        Ok(())
      },
    );

    methods.add_method_mut("grid", |lua, this, (id, func): (String, mlua::Function)| {
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
    });

    methods.add_method_mut(
      "collapsing_header",
      |lua, this, (label, func): (String, mlua::Function)| {
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

    // methods.add_method_mut("table", |lua, this, (id, func): (String, LuaFunction)| {
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

    methods.add_method_mut("set_attribs", |_, this, style_table: mlua::Table| {
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
    });

    methods.add_method_mut("set_style", |_, this, (style_table, context): (mlua::Table, bool)| {
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
            |val: mlua::Value| {
              style.spacing.menu_margin = table_into_margin!(val)
            }
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
    });

    methods.add_method_mut("set_spacing", |_, this, spacing: f32| {
      let ctx = this.ui.ctx();
      let mut style = (*ctx.style()).clone();

      style.spacing.item_spacing = egui::vec2(spacing, spacing);
      this.ui.set_style(style);
      Ok(())
    });

    methods.add_method_mut(
      "columns",
      |lua, this, (n, func): (usize, mlua::Function)| {
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
      |lua, this, (layout_name, func): (String, mlua::Function)| {
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

    methods.add_method_mut("visuals_mut", |_, this, func: mlua::Function| {
      let visuals = LuaVisuals(this.ui.visuals_mut().clone());
      func.call::<()>(visuals)?;
      Ok(())
    });

    // menu_button
    methods.add_method_mut(
      "menu_button",
      |lua, this, (label, func): (String, mlua::Function)| {
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
      |lua, this, (style, func): (mlua::Table, mlua::Function)| {
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
      |lua, this, (layout_name, align, func): (String, String, mlua::Function)| {
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
    // methods.add_method_mut("context_menu", |lua, this, func: mlua::Function| {
    //   this.ui.context_menu(|ui| {
    //     let lua_ui = LuaUi { ui };
    //     func.call::<()>(lua_ui).unwrap();
    //   });
    //   Ok(())
    // });

    methods.add_method_mut(
      "place_ui_at",
      |lua, this, (x, y, w, h, func): (f32, f32, f32, f32, mlua::Function)| {
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
    methods.add_method_mut("clip_rect", |_, this, ()| {
      let rect = this.ui.clip_rect();
      Ok((rect.min.x, rect.min.y, rect.max.x, rect.max.y))
    });
  }
}

impl LuluUiApp {
  fn new(
    _cc: &eframe::CreationContext<'_>,
    lulu: Lulu,
    main: Option<mlua::Value>,
    err: Option<String>,
  ) -> Self {
    lulu.lua.set_app_data(_cc.egui_ctx.clone());

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
      .get::<mlua::Table>("coroutine").unwrap()
      .get("resume").unwrap();
    let sched_co: mlua::Value = self
      .lulu
      .lua
      .globals()
      .get::<mlua::Table>("Future").unwrap()
      .get("scheduler").unwrap();

    scheduler.call::<mlua::Value>(sched_co.clone()).unwrap();
  }
}

async fn load_main(lulu: &mut Lulu) -> Result<mlua::Value, String> {
  lulu.preload_mods().map_err(|e| e.to_string())?;

  let main_name = lulu.find_mod("main").map_err(|e| e.to_string())?;

  let ui_code = std::fs::read_to_string("src/lua/ui.lua").map_err(|e| e.to_string())?;
  let f = lulu
    .lua
    .load(lulu.compiler.compile(&ui_code, None, None))
    .set_name("ui.lua")
    .eval()
    .map_err(|e| e.to_string())?;

  lulu
    .exec_mod(&main_name)
    .map_err(|e| e.to_string())?;
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

  let lulu = lulu.clone();

  eframe::run_native(
    "Lulu UI",
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
