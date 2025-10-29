

use mlua::prelude::LuaError;
mod ui;
mod shape;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> mlua::Result<()> {
  let mut lulu =
      lulu::lulu::Lulu::new(Some(std::env::args().skip(1).collect()),
      Some(std::env::current_exe()?.parent().unwrap().to_path_buf()));

  let font_bytes = include_bytes!("../assets/fonts/DejaVuSansMono.ttf");
  let lua_font_bytes = lulu.lua.create_string(&font_bytes)?;
  lulu.lua.globals().set("DEJAVU_FONT_BYTES", lua_font_bytes)?;

  lulu.compiler.compile(include_str!("./lua/macros.lua"), None, None);

  if let Some(mods) = lulu::bundle::load_embedded_scripts() {
    lulu::bundle::reg_bundle_nods(&mut lulu, mods)?;
  } else {
    let path = std::path::Path::new("main.lua");
    if path.exists() {
      lulu.entry_mod_path(path.to_path_buf())?;
    }
    let path = std::path::Path::new("test/main.lua");
    if path.exists() {
      lulu.entry_mod_path(path.to_path_buf())?;
    }
  }
  lulu::handle_error!(ui::run(&mut lulu).await.map_err(|e| mlua::Error::external(e.to_string())));

  Ok(())
}

