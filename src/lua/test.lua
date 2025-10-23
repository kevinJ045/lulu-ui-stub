

-- -- Global state variables
-- local text_state = "Initial Text"
-- local multiline_text_state = "Multi-line\nText"
-- local checkbox_state = true
-- local slider_state = 50.0
-- local drag_value_state = 123.45
-- local radio_value_state = "Option 1"
-- local progress_bar_state = 0.5
-- local color_val = { 255, 255, 0, 255 }

-- ui.add(function(ui)
--   ui:label("Hello from Lua!")
--   ui:separator()
--   ui:set_style({
--     frame_fill = {0, 0, 255, 255}
--   })

--   ui:horizontal(function(h_ui)
--     h_ui:set_style({
--       rounding = 25,
--       spacing = 30
--     })
--     h_ui:label("Text Input:")
--     local changed_text = h_ui:text_edit_singleline("my_text_edit", text_state)
--     if changed_text.changed then
--       text_state = changed_text.value
--       print("Text changed to: " .. text_state)
--     end
--   end)

--   ui:vertical(function(v_ui)
--     v_ui:label("Multi-line Text Input:")
--     local changed_multiline_text = v_ui:text_edit_multiline("my_multiline_text_edit", multiline_text_state)
--     if changed_multiline_text.changed then
--       multiline_text_state = changed_multiline_text.value
--       print("Multi-line text changed to: " .. multiline_text_state)
--     end
--   end)

--   ui:horizontal(function(h_ui)
--     local changed_checkbox = h_ui:checkbox("Enable Feature", checkbox_state)
--     if changed_checkbox.changed then
--       checkbox_state = changed_checkbox.value
--       print("Checkbox changed to: " .. tostring(changed_checkbox.value)) -- Fixed typo here
--     end

--     local changed_slider = h_ui:slider("Value", 0.0, 100.0, slider_state)
--     if changed_slider.changed then
--       slider_state = changed_slider.value
--       print("Slider changed to: " .. tostring(slider_state))
--     end
--   end)

--   local changed_drag_value = ui:drag_value("Drag Me:", drag_value_state)
--   if changed_drag_value.changed then
--     drag_value_state = changed_drag_value.value
--     print("Drag value changed to: " .. tostring(drag_value_state))
--   end

--   ui:hyperlink("https://www.example.com")

--   ui:separator()

--   ui:horizontal(function(h_ui)
--     local changed_radio1 = h_ui:radio_button("Option 1", radio_value_state, "Option 1")
--     if changed_radio1.changed then
--       radio_value_state = changed_radio1.value
--       print("Radio button changed to: " .. radio_value_state)
--     end
--     local changed_radio2 = h_ui:radio_button("Option 2", radio_value_state, "Option 2")
--     if changed_radio2.changed then
--       radio_value_state = changed_radio2.value
--       print("Radio button changed to: " .. radio_value_state)
--     end
--   end)

--   ui:separator()

--   ui:spinner()
--   ui:progress_bar(progress_bar_state, "Progress")
--   progress_bar_state = progress_bar_state + 0.01
--   if progress_bar_state > 1.0 then
--     progress_bar_state = 0.0
--   end

--   ui:horizontal(function(h_ui)
--     if h_ui:button("Click me! (Lua)").clicked then
--       print("Button clicked in Lua!")
--     end
--     if h_ui:button("Click me 2! (Lua)").clicked then
--       print("Button 2 clicked in Lua!")
--     end
--   end)

--   ui:collapsing_header("shshs", function(c_ui)
--     if c_ui:label("Some text").clicked then
--       print("Label clicked in Lua!")
--     end
--   end)

  
--   ui:frame_block({
--     color = {100, 100, 0, 255},
--     padding = 10,
--     width = "fill"
--   }, function(f_ui)
--     f_ui:label("Some text")
--   end)

--   local color_picker = ui:color_picker(color_val)
--   if color_picker.changed then
--     fprint(color_picker.value)
--     color_val = color_picker.value
--   end
-- end)


local clicked_state = State(0)
local text_state = State("sksk")
local inactive = State(false)
local loading = State(true)
local fetched_text = State("")

async(function()
  local dd = net.http.request("https://api.github.com")
  loading:set(false)
  fetched_text:set(dd.text())
end)

local t = lml! {
  <style style={{
      visuals = {
        hyperlink_color = { 255, 0, 0, 255 },
        faint_bg_color = { 255, 0, 255, 255 },
        extreme_bg_color = { 255, 255, 0, 255 },
        panel_fill = { 0, 255, 0, 255 },
        inactive = {
          bg_fill = { 255, 0, 0, 255 }
        }
      }
    }}>
    <frame inactive={loading:inverse()} style={{
      color = { 255, 255, 0, 100 },
      width = "fill",
      height = "70%",
    }}>
      <align layout="center_both">
        <spinner />
      </align>
    </frame>
    <vbox inactive={loading}>
      <hbox>
        <label text="Some button:" />
        <button style={{
          color = { 255, 0, 0, 255 }
        }} text={clicked_state:format("Clicked: {}")} on_clicked={function() clicked_state:add(1) end} />
      </hbox>

      <hyperlink text="https://example.com" />
      <input value={text_state} on_changed={function() print(text_state:get()) end} />

      <checkbox text="Hide Separator" checked={inactive} />
      
      <handle inactive={inactive} render={function(ui) ui:separator() end} />

      <progress_bar value={0.6} text="Progress" />
      <textbox value={fetched_text} />

    </vbox>
  </style>
}


t:into_root()