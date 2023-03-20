use tiled::Chunk;
use crate::{game_maps::GameMap, math::{Vec2, Rect}};

pub struct Collision {
  pub size: (i32, i32),
  pub offset: (i32, i32),
  pub collision_layer: Vec<bool>,
}

impl Collision {
  pub fn from_game_map(game_map: &GameMap) -> Self {
    let main_layer = game_map.get_main_layer();

    let mut max_xy = (i32::MIN, i32::MIN);
    let mut min_xy = (i32::MAX, i32::MAX);

    match main_layer.layer_type() {
      tiled::LayerType::TileLayer(tiled::TileLayer::Infinite(data)) => {
        for (chunk_pos, chunk) in data.chunks() {
          for x in 0..Chunk::WIDTH as i32 {
            for y in 0..Chunk::HEIGHT as i32 {
              if let Some(tile) = chunk.get_tile(x, y) {
                let tile_pos = (
                  chunk_pos.0 * Chunk::WIDTH as i32 + x,
                  chunk_pos.1 * Chunk::HEIGHT as i32 + y,
                );
                crate::log(&format!("At ({}, {}): {:?}", tile_pos.0, tile_pos.1, tile));
                max_xy.0 = max_xy.0.max(tile_pos.0);
                max_xy.1 = max_xy.1.max(tile_pos.1);
                min_xy.0 = min_xy.0.min(tile_pos.0);
                min_xy.1 = min_xy.1.min(tile_pos.1);
              }
            }
          }
        }
      }
      _ => panic!("Unsupported layer type"),
    }
    crate::log(&format!("Max: {:?}", max_xy));
    crate::log(&format!("Min: {:?}", min_xy));

    let size = (
      max_xy.0 - min_xy.0 + 1,
      max_xy.1 - min_xy.1 + 1,
    );
    let offset = (
      min_xy.0,
      min_xy.1,
    );
    let mut collision_layer = vec![false; (size.0 * size.1) as usize];
    match main_layer.layer_type() {
      tiled::LayerType::TileLayer(tiled::TileLayer::Infinite(data)) => {
        for (chunk_pos, chunk) in data.chunks() {
          for x in 0..Chunk::WIDTH as i32 {
            for y in 0..Chunk::HEIGHT as i32 {
              if let Some(_) = chunk.get_tile(x, y) {
                let tile_pos = (
                  chunk_pos.0 * Chunk::WIDTH as i32 + x - offset.0,
                  chunk_pos.1 * Chunk::HEIGHT as i32 + y - offset.1,
                );
                collision_layer[(tile_pos.1 * size.0 + tile_pos.0) as usize] = true;
              }
            }
          }
        }
      }
      _ => panic!("Unsupported layer type"),
    }

    Self {
      size,
      offset,
      collision_layer,
    }
  }

  fn check_collision(&self, x: i32, y: i32) -> bool {
    let x = x - self.offset.0;
    let y = y - self.offset.1;
    if x < 0 || y < 0 || x >= self.size.0 || y >= self.size.1 {
      return false;
    }
    self.collision_layer[(y * self.size.0 + x) as usize]
  }

  /// Checks if the rect overlaps any of the collision tiles.
  pub fn check_rect_collision(&self, r: Rect) -> bool {
    let lowest_x = r.pos.0.floor() as i32;
    let lowest_y = r.pos.1.floor() as i32;
    let highest_x = (r.pos.0 + r.size.0).ceil() as i32;
    let highest_y = (r.pos.1 + r.size.1).ceil() as i32;
    for x in lowest_x..highest_x {
      for y in lowest_y..highest_y {
        if self.check_collision(x, y) {
          return true;
        }
      }
    }
    false
  }

  pub fn try_move_rect(&self, r: Rect, delta: Vec2) -> (Vec2, bool) {
    let mut new_pos = r.pos;
    let mut has_collision = false;
    for i in 0..41 {
      let test_pos = r.pos + delta * (i as f32 / 40.0);
      if self.check_rect_collision(Rect {
        pos: test_pos,
        size: r.size,
      }) {
        has_collision = true;
        break;
      }
      new_pos = test_pos;
    }
    (new_pos, has_collision)
  }
}
