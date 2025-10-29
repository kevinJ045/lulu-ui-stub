local bevy = BevyInstance()

bevy:load_asset("MODEL_SOMETHING", "assets/something.glb")

bevy:render(function(commands)
  ...
end)

return function(ui)
  ui:bevy_container(bevy)
end