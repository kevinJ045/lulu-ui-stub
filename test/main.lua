

() @namespace(ui) =>

  local BLOCKS = State(Vec())

  (self, props) @Component Block =>
    return Frame {
        style = {
          color = c"#181825",
          min_width = "100%"
        },
        child = Label {
          text = props.text
        }
      }
  end 

  (self) @AutoRender @StatedComponent({
    currentCommand = ""
  }) @Component =>

    return Style {
      style = {
        spacing = {
          button_padding = { 15, 5 },
        },
        visuals = {
          panel_fill = c"#1e1e2e",
          noninteractive = {
            bg_fill = c"#181825",
            weak_bg_fill = c"#181825",
          },
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
            bg_fill = c"#1e1e2e",
            weak_bg_fill = c"#1e1e2e",
            fg_stroke = {
              width = 1.0,
              color = c"#cdd6f4"
            },
            rounding = { 20, 20, 20, 20 }
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
      child = ScrollArea {
        stick_to_bottom = true,
        id = "ddd",
        children = {
          Each {
            items = BLOCKS,
            render = function(e)
              return Block {
                text = e
              }
            end
          },

          HBox {
            children = {
              Input {
                text = self.currentCommand,
                id = "mytextedit",
                frame = false,
                multiline = true,
                placeholder = "hello",
                width = "100%",
                on_changed = function(s, e)
                  if e.keypressed('Enter') then
                    BLOCKS:push(e.value)
                    self.currentCommand:set("")
                    s:focus()
                  end
                end
              }
            }
          }
        }
      }
    }

  end

end
