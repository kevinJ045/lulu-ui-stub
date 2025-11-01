
() @namespace(ui) =>

  local (self, props) @Component() AddButton =>
    return Button {
      text = f"Add",
      on_clicked = function()
        props.clicked:add(1)
      end
    }
  end

  local (self, props) @Component() SubButton =>
    return Button {
      text = f"Subtract",
      on_clicked = function()
        props.clicked:sub(1)
      end
    }
  end

  local (self, props) @Component() Buttons =>
    return HBox {
      AddButton { clicked = props.clicked },
      SubButton { clicked = props.clicked },
      props.children
    }
  end

  local (self) @AutoRender @StatedComponent({
    clicked = 0
  }) @Component() AppRoot =>
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
        Buttons {
          clicked = self.clicked,
          children = {
            Button { text = "Normal button" }
          }
        }
      }
    }
  end

end