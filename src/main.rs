

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


  let lua_code = std::fs::read_to_string("src/lua/test.lua")?;

  ui::run(lulu, lua_code).await.map_err(|e| mlua::Error::external(e.to_string()))?;

  Ok(())
}

