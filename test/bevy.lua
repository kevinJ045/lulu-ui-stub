
() @namespace(ui) =>

  local (self, props) @Component AddButton =>
    return Button {
      text = f"Add",
      on_clicked = function()
        props.clicked:add(1)
      end
    }
  end

  local (self, props) @Component SubButton =>
    return Button {
      text = f"Subtract",
      on_clicked = function()
        props.clicked:sub(1)
      end
    }
  end

  local (self) @AutoRender @StatedComponent({
    clicked = 0
  }) @Component AppRoot =>
    return VBox {
      children = {
        {
          match! self.clicked:get(), {
            (val > 10) {
              return ColoredLabel {
                text = "Too much",
                color = { 255, 255, 255, 255 }
              }
            }
          }
        },
        Label {
          text = f"Clicked: {self.clicked:get()}"
        },
        HBox {
          children = {
            AddButton { clicked = self.clicked },
            SubButton { clicked = self.clicked },
          }
        }
      }
    }
  end

end