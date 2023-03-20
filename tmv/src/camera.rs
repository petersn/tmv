use crate::{math::Vec2, game_maps::GameMap, tile_rendering::TILE_SIZE};


pub struct Boundary {
  pub a: Vec2,
  pub b: Vec2,
}

pub struct CameraBounds {
  pub boundaries: Vec<Boundary>,
}

impl CameraBounds {
  pub fn from_game_map(game_map: &GameMap) -> Self {
    let layer = game_map.map.layers().find(|l| l.name == "CameraBounds").unwrap();
    let mut boundaries = Vec::new();

    match layer.layer_type() {
      tiled::LayerType::ObjectLayer(object_layer) => {
        for object in object_layer.objects() {
          match &object.shape {
            tiled::ObjectShape::Polyline { points } | tiled::ObjectShape::Polygon { points } => {
              let mut points =
                points.iter().map(|p| (p.0 / TILE_SIZE, p.1 / TILE_SIZE)).collect::<Vec<_>>();
              // If the shape is a polygon, we close it.
              if let tiled::ObjectShape::Polygon { .. } = object.shape {
                points.push(points[0]);
              }
              for i in 0..points.len() - 1 {
                boundaries.push(Boundary {
                  a: Vec2(points[i].0, points[i].1),
                  b: Vec2(points[i + 1].0, points[i + 1].1),
                });
              }
            }
            _ => panic!("Unsupported object shape: {:?}", object.shape),
          }
        }
      }
      _ => panic!("Unsupported layer type"),
    }
    Self { boundaries }
  }
}
