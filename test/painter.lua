
() @namespace(ui) =>


  local (self) @AutoRender @Component() AppRoot =>
    return VBox {
      children = {
        Painter {
          pos = { x = 10, y = 10 },
          siz = { x = 100, y = 50 },
          render = function(painter, ui)

            if ui:keydown('w') then
              pos.y = pos.y - 10
            elseif ui:keydown('s') then
              pos.y = pos.y + 10
            end

            if ui:keydown('a') then
              pos.x = pos.x - 10
            elseif ui:keydown('d') then
              pos.x = pos.x + 10
            end
            
            if ui:keydown('⏶') then
              siz.y = siz.y - 1
            end
            if ui:keydown('⏷') then
              siz.y = siz.y + 1
            end

            if ui:keydown('⏴') then
              siz.x = siz.x - 1
            end
            if ui:keydown('⏵') then
              siz.x = siz.x + 1
            end

            painter:rect_filled(pos.x, pos.y, siz.x, siz.y, {0, 255, 0, 1.0})

            painter:circle_filled(150, 35, 25, {255, 0, 0, 1.0}) -- red

            ui:image("/home/makano/Pictures/coffeescript_waifu2x_art_noise3_scale.png", {
              at = { 200, 200 },
              width = 100,
              height = 100
            })
          end,
        }
      }
    }
  end


end