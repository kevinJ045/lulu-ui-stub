return function(ui)
  local painter = ui:painter()

  painter:rect_filled(10, 10, 100, 50, {0, 1, 0, 1}) -- green
  painter:circle_filled(150, 35, 25, {1, 0, 0, 1}) -- red
end
