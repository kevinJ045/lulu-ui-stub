
() @namespace(ui) =>

  local LIGHT_COLORS, DARK_COLORS = {c"#ffffff", c"#fdfdfd"}, { c"#1e1e2e", c"#181825" }

  local (self) @AutoRender @StatedComponent({
    colors = { c"#1e1e2e", c"#181825" },
    accent = c"#cba6f7",
    selected_accent = "mauve",
    is_dark = true
  }) @ComponentValues({
    accents = { mauve = c"#cba6f7", red = c"#f38ba8", sky = c"#89dceb", rosewater = c"#f5e0dc" },
    selectables = { mauve = "Mauve", red = "Red", sky = "Sky", rosewater = "Rosewater" }
  }) @UIOverride('rebuild') @Component AppRoot =>
    return Style {
      style = self.colors:map((col) => return {
        spacing = {
          button_padding = { 15, 5 },
        },
        font = {
          heading_size = 32.0
        },
        visuals = {
          panel_fill = col[1],
          noninteractive = {
            bg_fill = col[2],
            weak_bg_fill = col[2],
          },
          inactive = {
            bg_fill = col[2],
            weak_bg_fill = col[2],
            rounding = { 20, 20, 20, 20 }
          },
          active = {
            bg_fill = self.selected_accent:inside(self.accents):get(),
            weak_bg_fill = self.selected_accent:inside(self.accents):get(),
            rounding = { 20, 20, 20, 20 }
          },
          open = {
            bg_fill = col[1],
            weak_bg_fill = col[1],
            fg_stroke = {
              width = 1.0,
              color = c"#cdd6f4"
            },
            rounding = { 20, 20, 20, 20 }
          },
          hovered = {
            bg_fill = self.selected_accent:inside(self.accents):get(),
            weak_bg_fill = self.selected_accent:inside(self.accents):get(),
            bg_stroke = {
              width = 1.0,
              color = col[2]
            },
            fg_stroke = {
              width = 1.0,
              color = col[2]
            },
            rounding = { 20, 20, 20, 20 }
          }
        }
      } end),
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
                  text = self.is_dark:bool("Light Theme", "Dark Theme"),
                  on_clicked = function()
                    self.colors:toggle_as(
                      self.is_dark,
                      LIGHT_COLORS,
                      DARK_COLORS
                    )
                    self.is_dark:toggle()
                  end
                },
                Combobox {
                  selected = self.selected_accent,
                  values = self.selectables,
                  label = "Select Accent Color",
                  on_changed = function()
                    self.colors:reemit()
                  end
                }
              }
            }
          }
        }
      }
    }
  end


end