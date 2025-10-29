use eframe::{
  egui::*,
  epaint::{CircleShape, RectShape},
};
use mlua::{self, UserData};

#[derive(Clone)]
pub struct LuaShape {
  pub shape: Shape,
}

impl UserData for LuaShape {}

pub fn from_lua_table(table: mlua::Table) -> Option<Shape> {
  if let Ok(shape_type) = table.get::<String>("type") {
    match shape_type.as_str() {
      "rect" => {
        let x: f32 = table.get("x").ok()?;
        let y: f32 = table.get("y").ok()?;
        let w: f32 = table.get("w").ok()?;
        let h: f32 = table.get("h").ok()?;
        let rect = Rect::from_min_size(pos2(x, y), vec2(w, h));

        let fill: mlua::Table = table.get("fill").ok()?;
        let fill_color = crate::ui::color_from_lua_table(fill).unwrap();

        let stroke: mlua::Table = table.get("stroke").ok()?;
        let stroke_color = crate::ui::color_from_lua_table(stroke.clone()).unwrap();
        let stroke_width: f32 = stroke.get("width").ok()?;

        Some(Shape::Rect(RectShape::new(
          rect,
          0.0,
          fill_color,
          Stroke::new(stroke_width, stroke_color),
        )))
      }
      "circle" => {
        let x: f32 = table.get("x").ok()?;
        let y: f32 = table.get("y").ok()?;
        let radius: f32 = table.get("radius").ok()?;
        let center = pos2(x, y);

        let fill: mlua::Table = table.get("fill").ok()?;
        let fill_color = crate::ui::color_from_lua_table(fill).unwrap();

        let stroke: mlua::Table = table.get("stroke").ok()?;
        let stroke_color = crate::ui::color_from_lua_table(stroke.clone()).unwrap();
        let stroke_width: f32 = stroke.get("width").ok()?;

        Some(Shape::Circle(CircleShape{
          center,
          radius,
          fill: fill_color,
          stroke: Stroke::new(stroke_width, stroke_color),
      }))
      }
      "line" => {
        let x1: f32 = table.get("x1").ok()?;
        let y1: f32 = table.get("y1").ok()?;
        let x2: f32 = table.get("x2").ok()?;
        let y2: f32 = table.get("y2").ok()?;
        let points = [pos2(x1, y1), pos2(x2, y2)];

        let color: mlua::Table = table.get("color").ok()?;
        let color = crate::ui::color_from_lua_table(color).unwrap();
        let width: f32 = table.get("width").ok()?;

        Some(Shape::LineSegment {
          points,
          stroke: Stroke::new(width, color),
        })
      }
      _ => None,
    }
  } else {
    None
  }
}
