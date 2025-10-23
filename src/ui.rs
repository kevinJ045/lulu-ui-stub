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
  get_ui_description_fn: Option<mlua::Function>,
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
    methods.add_method_mut("button", |_lua, this, text: String| {
      let res: Response = this.ui.button(text);
      Ok(LuaUiResponse { res, value: None })
    });
    methods.add_method_mut("label", |_lua, this, text: String| {
      Ok(LuaUiResponse {
        res: this.ui.label(text),
        value: None,
      })
    });
    methods.add_method_mut(
      "text_edit_singleline",
      |lua, this, (_id, text): (String, String)| {
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
      "text_edit_multiline",
      |lua, this, (_id, text): (String, String)| {
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
          value: Some(mlua::Value::Number(val as f64))
        })
      },
    );
    methods.add_method_mut("drag_value", |_lua, this, (text, value): (String, f64)| {
      let mut val = value;
      let response = this.ui.add(egui::DragValue::new(&mut val).prefix(text));
      Ok(LuaUiResponse {
        res: response,
        value: Some(mlua::Value::Number(val))
      })
    });
    methods.add_method_mut("hyperlink", |_lua, this, url: String| {
      this.ui.hyperlink(url);
      Ok(())
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
    methods.add_method_mut("separator", |_lua, this, ()| {
      this.ui.separator();
      Ok(())
    });
    methods.add_method_mut("spinner", |_lua, this, ()| {
      this.ui.spinner();
      Ok(())
    });
    methods.add_method_mut(
      "progress_bar",
      |_lua, this, (fraction, text): (f32, String)| {
        this.ui.add(egui::ProgressBar::new(fraction).text(text));
        Ok(())
      },
    );
    methods.add_method_mut("vertical", |lua, this, func: mlua::Function| {
      this.ui.vertical(|ui| {
        lua
          .scope(|_scope| {
            let lua_ui = _scope.create_userdata(LuaUi { ui }).unwrap();
            // Create a temporary Lua function that calls the original func
            let temp_func = _scope
              .create_function(move |_lua, ()| {
                func.call::<()>(lua_ui.clone()) // Call the original func with lua_ui
              })
              .unwrap();
            temp_func.call::<()>(()).unwrap();
            Ok(())
          })
          .unwrap();
      });
      Ok(())
    });
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
    methods.add_method_mut("horizontal", |lua, this, func: mlua::Function| {
      this.ui.horizontal(|ui| {
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

    methods.add_method_mut(
      "color_picker",
      |_, this, (r, g, b, a): (f32, f32, f32, f32)| {
        let mut color = egui::Color32::from_rgba_unmultiplied(
          (r * 255.0) as u8,
          (g * 255.0) as u8,
          (b * 255.0) as u8,
          (a * 255.0) as u8,
        );
        egui::widgets::color_picker::color_edit_button_srgba(
          this.ui,
          &mut color,
          egui::color_picker::Alpha::Opaque,
        );
        Ok(())
      },
    );

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

    methods.add_method_mut("align", |_, this, align: String| {
      let alignment = match align.as_str() {
        "left" => egui::Align::Min,
        "center" => egui::Align::Center,
        "right" => egui::Align::Max,
        _ => egui::Align::Min,
      };
      this
        .ui
        .with_layout(egui::Layout::left_to_right(alignment), |_| {});
      Ok(())
    });

    methods.add_method_mut("set_attribs", |_, this, style_table: mlua::Table| {
      set_attrib!(("height", String), style_table, |val: String| {
        this.ui.set_height(get_size_attrib!(this.ui, val))
      });
      set_attrib!(("width", String), style_table, |val: String| {
        this.ui.set_width(get_size_attrib!(this.ui, val))
      });

      // set_attrib(1)

      set_attrib!(("visible", bool), style_table, |val: bool| {
        this.ui.set_visible(val);
      });

      Ok(())
    });

    methods.add_method_mut("set_style", |_, this, style_table: mlua::Table| {
      let ctx = this.ui.ctx();
      let mut style = (*ctx.style()).clone();

      if let Ok(spacing) = style_table.get::<f32>("spacing") {
        style.spacing.item_spacing = egui::vec2(spacing, spacing);
      }

      if let Ok(rounding) = style_table.get::<f32>("rounding") {
        style.visuals.widgets.noninteractive.rounding = egui::Rounding::same(rounding);
        style.visuals.widgets.inactive.rounding = egui::Rounding::same(rounding);
        style.visuals.widgets.hovered.rounding = egui::Rounding::same(rounding);
        style.visuals.widgets.active.rounding = egui::Rounding::same(rounding);
      }

      if let Ok(fill_table) = style_table.get::<mlua::Table>("frame_fill") {
        if let Some(color) = color_from_lua_table(fill_table) {
          style.visuals.extreme_bg_color = color;
          style.visuals.window_fill = color;
        }
      }

      set_attrib!(
        ("extreme_bg_color", mlua::Table),
        style_table,
        |val: mlua::Table| {
          is_color!(val, style.visuals.extreme_bg_color);
        }
      );

      set_attrib!(
        ("window_fill", mlua::Table),
        style_table,
        |val: mlua::Table| {
          is_color!(val, style.visuals.window_fill);
        }
      );

      set_attrib!(
        ("window_fill", mlua::Table),
        style_table,
        |val: mlua::Table| {
          is_color!(val, style.visuals.window_fill);
        }
      );

      if let Ok(text_table) = style_table.get::<mlua::Table>("text_color") {
        if let Some(color) = color_from_lua_table(text_table) {
          style.visuals.override_text_color = Some(color);
        }
      }

      if let Ok(br) = style_table.get::<f32>("button_rounding") {
        style.visuals.widgets.inactive.rounding = egui::Rounding::same(br);
      }

      this.ui.set_style(style);

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

        // Frame color
        if let Ok(color_tbl) = style.get::<mlua::Table>("color") {
          frame = frame.fill(color_from_lua_table(color_tbl).unwrap());
        }

        // Stroke
        if let Ok(stroke_tbl) = style.get::<mlua::Table>("stroke") {
          let width: f32 = stroke_tbl.get(5).unwrap_or(1.0);
          frame = frame.stroke(Stroke::new(
            width,
            color_from_lua_table(stroke_tbl).unwrap(),
          ));
        }

        // Rounding
        if let Ok(rounding) = style.get::<f32>("rounding") {
          frame = frame.rounding(Rounding::same(rounding));
        }

        // Padding
        if let Ok(padding) = style.get::<f32>("padding") {
          frame = frame.inner_margin(Margin::same(padding));
        }

        // Min size
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
            // Width
            if let Ok(w) = style.get::<String>("width") {
              ui.set_min_width(get_size_attrib!(ui, w));
            }

            // Height
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
      "with_layout",
      |lua, this, (layout_name, align, func): (String, String, mlua::Function)| {
        let alignment = match align.as_str() {
          "left" => egui::Align::Min,
          "center" => egui::Align::Center,
          "right" => egui::Align::Max,
          _ => egui::Align::Min,
        };
        let layout = match layout_name.as_str() {
          "left_to_right" => Layout::left_to_right(alignment),
          "right_to_left" => Layout::right_to_left(alignment),
          "top_down" => Layout::top_down(alignment),
          "bottom_up" => Layout::bottom_up(alignment),
          _ => Layout::default(),
        };
        this.ui.with_layout(layout, |ui| {
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
  fn new(_cc: &eframe::CreationContext<'_>, lulu: Lulu, lua_code: String) -> Self {
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

    let mut init_error: Option<String> = None;
    let get_ui_description_fn = match lulu.lua.load(&lua_code).eval::<mlua::Function>() {
      Ok(func) => Some(func),
      Err(e) => {
        init_error = Some(e.to_string());
        None
      }
    };

    Self {
      lulu,
      get_ui_description_fn,
      init_error,
    }
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
    } else if let Some(render_fn) = &self.get_ui_description_fn {
      // Renamed for clarity
      egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical(|ui| {
          // Add a vertical layout
          self
            .lulu
            .lua
            .scope(|_scope| {
              let lua_ui = _scope.create_userdata(LuaUi { ui }).unwrap();
              // Create a temporary Lua function that calls the original func
              let temp_func = _scope
                .create_function(move |_lua, ()| {
                  render_fn.call::<()>(lua_ui.clone()) // Call the original func with lua_ui
                })
                .unwrap();
              temp_func.call::<()>(()).unwrap(); // Call the temporary function
              Ok(()) // The scope closure needs to return a Result
            })
            .unwrap();
        });
      });
    }
  }
}

pub async fn run(lulu: Lulu, lua_code: String) -> Result<(), eframe::Error> {
  let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
    ..Default::default()
  };

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

      Box::new(LuluUiApp::new(cc, lulu, lua_code))
    }),
  )
}
