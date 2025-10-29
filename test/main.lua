
() @namespace(ui) =>


  local (self) @AutoRender @Component AppRoot =>
    return VBox {
      children = {
        Painter {
          pos = { x = 10, y = 10 },
          render = function(painter)
            painter:rect_filled(pos.x, pos.y, 100, 50, {0, 1, 0, 1}) -- green
            painter:circle_filled(150, 35, 25, {1, 0, 0, 1}) -- red
          end,
        }
      }
    }
  end


end