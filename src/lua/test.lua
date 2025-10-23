register_font("DejaVuSansMono", DEJAVU_FONT_BYTES)

-- Global state variables
local text_state = "Initial Text"
local multiline_text_state = "Multi-line\nText"
local checkbox_state = true
local slider_state = 50.0
local drag_value_state = 123.45
local radio_value_state = "Option 1"
local progress_bar_state = 0.5

local function render_ui(ui)
  ui:label("Hello from Lua!")
  ui:separator()
  ui:set_style({
    frame_fill = {0, 0, 255, 255}
  })

  ui:horizontal(function(h_ui)
    h_ui:set_style({
      rounding = 25,
      spacing = 30
    })
    h_ui:label("Text Input:")
    local changed_text = h_ui:text_edit_singleline("my_text_edit", text_state)
    if changed_text.changed then
      text_state = changed_text.value
      print("Text changed to: " .. text_state)
    end
  end)

  ui:vertical(function(v_ui)
    v_ui:label("Multi-line Text Input:")
    local changed_multiline_text = v_ui:text_edit_multiline("my_multiline_text_edit", multiline_text_state)
    if changed_multiline_text.changed then
      multiline_text_state = changed_multiline_text.value
      print("Multi-line text changed to: " .. multiline_text_state)
    end
  end)

  ui:horizontal(function(h_ui)
    local changed_checkbox = h_ui:checkbox("Enable Feature", checkbox_state)
    if changed_checkbox.changed then
      checkbox_state = changed_checkbox.value
      print("Checkbox changed to: " .. tostring(changed_checkbox.value)) -- Fixed typo here
    end

    local changed_slider = h_ui:slider("Value", 0.0, 100.0, slider_state)
    if changed_slider.changed then
      slider_state = changed_slider.value
      print("Slider changed to: " .. tostring(slider_state))
    end
  end)

  local changed_drag_value = ui:drag_value("Drag Me:", drag_value_state)
  if changed_drag_value.changed then
    drag_value_state = changed_drag_value.value
    print("Drag value changed to: " .. tostring(drag_value_state))
  end

  ui:hyperlink("https://www.example.com")

  ui:separator()

  ui:horizontal(function(h_ui)
    local changed_radio1 = h_ui:radio_button("Option 1", radio_value_state, "Option 1")
    if changed_radio1.changed then
      radio_value_state = changed_radio1.value
      print("Radio button changed to: " .. radio_value_state)
    end
    local changed_radio2 = h_ui:radio_button("Option 2", radio_value_state, "Option 2")
    if changed_radio2.changed then
      radio_value_state = changed_radio2.value
      print("Radio button changed to: " .. radio_value_state)
    end
  end)

  ui:separator()

  ui:spinner()
  ui:progress_bar(progress_bar_state, "Progress")
  progress_bar_state = progress_bar_state + 0.01
  if progress_bar_state > 1.0 then
    progress_bar_state = 0.0
  end

  ui:horizontal(function(h_ui)
    if h_ui:button("Click me! (Lua)") then
      -- print("Button clicked in Lua!")
    end
    if h_ui:button("Click me 2! (Lua)") then
      -- print("Button 2 clicked in Lua!")
    end
  end)

  ui:collapsing_header("shshs", function(c_ui)
    c_ui:label("Some text")
  end)

  
  ui:frame_block({
    color = {100, 100, 0, 255},
    padding = 10,
    width = "fill"
  }, function(f_ui)
    f_ui:label("Some text")
  end)
end

return render_ui