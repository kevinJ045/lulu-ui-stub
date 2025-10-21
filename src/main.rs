

mod ui;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> mlua::Result<()> {
  let mut lulu =
      lulu::lulu::Lulu::new(Some(std::env::args().skip(1).collect()),
      Some(std::env::current_exe()?.parent().unwrap().to_path_buf()));

  // This is a mlua Lua instance
  lulu.lua = mlua::Lua::new();

  // Read font bytes
  let font_bytes = std::fs::read("assets/fonts/DejaVuSansMono.ttf")
      .map_err(|e| mlua::Error::external(format!("Failed to read font file: {}", e)))?;
  let lua_font_bytes = lulu.lua.create_string(&font_bytes)?;
  lulu.lua.globals().set("DEJAVU_FONT_BYTES", lua_font_bytes)?;


  let lua_code = r#"
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

  ui:horizontal(function(h_ui)
    h_ui:label("Text Input:")
    local changed_text, new_text_value = h_ui:text_edit_singleline("my_text_edit", text_state)
    if changed_text then
      text_state = new_text_value
      print("Text changed to: " .. text_state)
    end
  end)

  ui:vertical(function(v_ui)
    v_ui:label("Multi-line Text Input:")
    local changed_multiline_text, new_multiline_text_value = v_ui:text_edit_multiline("my_multiline_text_edit", multiline_text_state)
    if changed_multiline_text then
      multiline_text_state = new_multiline_text_value
      print("Multi-line text changed to: " .. multiline_text_state)
    end
  end)

  ui:horizontal(function(h_ui)
    local changed_checkbox, new_checked_value = h_ui:checkbox("Enable Feature", checkbox_state)
    if changed_checkbox then
      checkbox_state = new_checked_value
      print("Checkbox changed to: " .. tostring(new_checked_value)) -- Fixed typo here
    end

    local changed_slider, new_slider_value = h_ui:slider("Value", 0.0, 100.0, slider_state)
    if changed_slider then
      slider_state = new_slider_value
      print("Slider changed to: " .. tostring(slider_state))
    end
  end)

  local changed_drag_value, new_drag_value = ui:drag_value("Drag Me:", drag_value_state)
  if changed_drag_value then
    drag_value_state = new_drag_value
    print("Drag value changed to: " .. tostring(drag_value_state))
  end

  ui:hyperlink("https://www.example.com")

  ui:separator()

  ui:horizontal(function(h_ui)
    local changed_radio1, new_radio_value1 = h_ui:radio_button("Option 1", radio_value_state, "Option 1")
    if changed_radio1 then
      radio_value_state = new_radio_value1
      print("Radio button changed to: " .. radio_value_state)
    end
    local changed_radio2, new_radio_value2 = h_ui:radio_button("Option 2", radio_value_state, "Option 2")
    if changed_radio2 then
      radio_value_state = new_radio_value2
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
      print("Button clicked in Lua!")
    end
    if h_ui:button("Click me 2! (Lua)") then
      print("Button 2 clicked in Lua!")
    end
  end)
end

return render_ui
  "#.to_string();

  ui::run(lulu, lua_code).await.map_err(|e| mlua::Error::external(e.to_string()))?;

  Ok(())
}

