
() @namespace(ui) =>


  local (self) @AutoRender @Component AppRoot =>
    return Style {
      style = {
        spacing = {
          button_padding = { 15, 5 },
        },
        font = {
          heading_size = 32.0
        },
        visuals = {
          panel_fill = c"#1e1e2e",
          inactive = {
            bg_fill = c"#181825",
            weak_bg_fill = c"#181825",
            rounding = { 20, 20, 20, 20 }
          },
          active = {
            bg_fill = c"#cba6f7",
            weak_bg_fill = c"#cba6f7",
            rounding = { 20, 20, 20, 20 }
          },
          open = {
            fg_stroke = {
              width = 1.0,
              color = c"#cdd6f4"
            },
          },
          hovered = {
            bg_fill = c"#cba6f7",
            weak_bg_fill = c"#cba6f7",
            bg_stroke = {
              width = 1.0,
              color = c"#181825"
            },
            fg_stroke = {
              width = 1.0,
              color = c"#181825"
            },
            rounding = { 20, 20, 20, 20 }
          }
        }
      },
      child = Align {
        layout = "center_both",
        child = Frame {
          style = {
            width = 100,
            height = 100,
          },
          child = Style {
            style = {
              spacing = {
                item_spacing = { 0, 10 }
              }
            },
            child = VBoxCentered {
              children = {
                Heading {
                  -- Converts to RichText with color 
                  text = txt_col! "#cdd6f4", "Custom styled";,
                },
                Label {
                  text = txt_col! "#cdd6f4", "A custom styled full component example";,
                },
                Button {
                  text = "hello"
                },
              }
            }
          }
        }
      }
    }
  end


end