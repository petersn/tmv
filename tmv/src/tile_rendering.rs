use std::collections::HashMap;
use std::rc::Rc;

use wasm_bindgen::prelude::*;

use crate::ImageResource;
// use crate::game::ImageResource;
use crate::game_maps::GameMap;
use crate::math::{Rect, Vec2};
// use crate::web::IntoJsError;

const TILE_SIZE: f32 = 32.0;
const CHUNK_SIZE_IN_PIXELS: f32 = TILE_SIZE * tiled::Chunk::WIDTH as f32;

// Statically assert that tiled::Chunk::WIDTH == tiled::Chunk::HEIGHT.
const _: () = [()][(tiled::Chunk::WIDTH != tiled::Chunk::HEIGHT) as usize];

pub struct TileRenderer {
  pub current_rect: Rect,
  pub game_map:     Rc<GameMap>,
}

impl TileRenderer {
  pub fn new(game_map: Rc<GameMap>, scratch_dims: Vec2) -> Self {
    Self {
      // Our starting rect is far away, forcing a rerender on the first .draw().
      current_rect: Rect::new(Vec2(-f32::MAX, -f32::MAX), scratch_dims),
      game_map,
    }
  }

  fn redraw(
    &mut self,
    (chunk_x, chunk_y): (i32, i32),
    images: &HashMap<ImageResource, web_sys::HtmlImageElement>,
    scratch_ctx: &web_sys::CanvasRenderingContext2d,
  ) {
    // Fill the scratch canvas with pink.
    scratch_ctx.set_fill_style(&JsValue::from_str("black"));
    scratch_ctx.fill_rect(
      0.0,
      0.0,
      self.current_rect.size.0 as f64,
      self.current_rect.size.1 as f64,
    );
    // FIXME: It's possible to reuse much of the existing image, by shifting it.
    let main_layer = self.game_map.get_main_layer();
    let chunk_count_x = (self.current_rect.size.0 / CHUNK_SIZE_IN_PIXELS).floor() as i32;
    let chunk_count_y = (self.current_rect.size.1 / CHUNK_SIZE_IN_PIXELS).floor() as i32;
    self.current_rect = Rect::new(
      Vec2(
        chunk_x as f32 * CHUNK_SIZE_IN_PIXELS,
        chunk_y as f32 * CHUNK_SIZE_IN_PIXELS,
      ),
      self.current_rect.size,
    );
    let mut tileset_index_to_imag_resource = HashMap::new();
    //let mut tileset_index_and_id_to_pos = HashMap::new();
    for (tileset_index, tileset) in self.game_map.map.tilesets().iter().enumerate() {
      if let Some(image) = &tileset.image {
        let image_resource = ImageResource::from_path(image.source.to_str().unwrap()).expect(
          &format!("Failed to find image resource for path: {:?}", image.source),
        );
        tileset_index_to_imag_resource.insert(tileset_index, image_resource);
      }
      // crate::log(&format!("Tileset {} has {} tiles in {} columns", tileset_index, tileset.tiles().len(), tileset.columns));
      // for (tile_index, (tile_id, _)) in tileset.tiles().enumerate() {
      //   crate::log(&format!("Index: {}, ID: {}", tile_index, tile_id));
      //   let ts_x = tile_index as u32 % tileset.columns;
      //   let ts_y = tile_index as u32 / tileset.columns;
      //   tileset_index_and_id_to_pos.insert((tileset_index, tile_id), (ts_x, ts_y));
      // }
    }

    match main_layer.layer_type() {
      tiled::LayerType::TileLayer(tiled::TileLayer::Infinite(data)) => {
        //println!("Infinite tile layer");
        // We iterate over the chunks in the desired rect.
        for y in 0..chunk_count_y {
          for x in 0..chunk_count_x {
            if let Some(chunk) = data.get_chunk(chunk_x + x, chunk_y + y) {
              // Draw the chunk.
              for tile_y in 0..tiled::Chunk::HEIGHT as i32 {
                for tile_x in 0..tiled::Chunk::WIDTH as i32 {
                  if let Some(tile) = chunk.get_tile(tile_x, tile_y) {
                    let tileset_index = tile.tileset_index();
                    //let (ts_x, ts_y) = tileset_index_and_id_to_pos[&(tileset_index, tile.id())];
                    let ts = tile.get_tileset();
                    // //let ts_index = tile.tileset_index() as u32;
                    let ts_index = tile.id() as u32;
                    let ts_x = ts_index % ts.columns;
                    let ts_y = ts_index / ts.columns;
                    let ts_pos = Vec2(ts_x as f32 * TILE_SIZE, ts_y as f32 * TILE_SIZE);
                    let chunk_pos = Vec2(
                      x as f32 * CHUNK_SIZE_IN_PIXELS,
                      y as f32 * CHUNK_SIZE_IN_PIXELS,
                    );
                    let tile_pos = Vec2(tile_x as f32 * TILE_SIZE, tile_y as f32 * TILE_SIZE);
                    let dest_pos = chunk_pos + tile_pos;
                    // let image_resource = tileset_index_to_imag_resource
                    //   .entry(tile.tileset_index())
                    //   .or_insert_with(|| {
                    //     let image_resource = ImageResource::Tileset(ts.name.clone());
                    //     images
                    //       .get(&image_resource)
                    //       .expect("Missing image resource")
                    //       .clone()
                    //   });
                    let image_resource = tileset_index_to_imag_resource
                      .get(&tileset_index)
                      .expect("Missing image resource");
                    scratch_ctx.translate(
                      (dest_pos.0 + TILE_SIZE / 2.0) as f64,
                      (dest_pos.1 + TILE_SIZE / 2.0) as f64,
                    );
                    if tile.flip_h {
                      // Mirror around dest_pos.0 + TILE_SIZE / 2
                      scratch_ctx.scale(-1.0, 1.0);
                    }
                    if tile.flip_v {
                      scratch_ctx.scale(1.0, -1.0);
                      //scratch_ctx.translate(0.0, TILE_SIZE as f64);
                    }
                    // Flip diagonally
                    if tile.flip_d {
                      scratch_ctx.rotate(std::f64::consts::FRAC_PI_2);
                      scratch_ctx.scale(1.0, -1.0);
                      // scratch_ctx.rotate(std::f64::consts::FRAC_PI_2);
                      // scratch_ctx.translate(0.0, -TILE_SIZE as f64);
                    }
                    scratch_ctx
                      .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                        &images[&image_resource],
                        ts_pos.0 as f64,
                        ts_pos.1 as f64,
                        TILE_SIZE as f64,
                        TILE_SIZE as f64,
                        -TILE_SIZE as f64 / 2.0, //dest_pos.0 as f64,
                        -TILE_SIZE as f64 / 2.0, //dest_pos.1 as f64,
                        TILE_SIZE as f64,
                        TILE_SIZE as f64,
                      );
                    // Reset the transform.
                    scratch_ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
                  }
                }
              }
            }
          }
        }
      }
      _ => panic!("Unexpected layer type"),
    }
  }

  pub fn draw(
    &mut self,
    draw_rect: Rect,
    dest: &web_sys::CanvasRenderingContext2d,
    images: &HashMap<ImageResource, web_sys::HtmlImageElement>,
    scratch_canvas: &web_sys::HtmlCanvasElement,
    scratch_ctx: &web_sys::CanvasRenderingContext2d,
  ) {
    // Clear the destination canvas.
    dest.clear_rect(0.0, 0.0, draw_rect.size.0 as f64, draw_rect.size.1 as f64);
    //crate::log(&format!("Starting rect: {:?} -- Request rect: {:?}", self.current_rect, draw_rect));
    // Determine if the desired rect is contained entirely within the current rect.
    if !self.current_rect.contains_rect(draw_rect) {
      crate::log(&format!(
        "Redrawing from rect {:?} to contain request rect: {:?}",
        self.current_rect, draw_rect
      ));
      // Recenter the current rect on the desired rect.
      let excess_size = self.current_rect.size - draw_rect.size;
      let top_left = draw_rect.pos - excess_size / 2.0;
      let chunk_x = (top_left.0 / CHUNK_SIZE_IN_PIXELS).round() as i32;
      let chunk_y = (top_left.1 / CHUNK_SIZE_IN_PIXELS).round() as i32;
      //self.current_rect = Rect::new(
      //  Vec2(
      //    tile_floor(),
      //    tile_floor(draw_rect.pos.1 - excess_size.1 / 2.0),
      //  ),
      //  self.current_rect.size,
      //);
      // Redraw ourself.
      self.redraw((chunk_x, chunk_y), images, scratch_ctx);
    }
    //crate::log(&format!("New rect: {:?} -- Request rect: {:?}", self.current_rect, draw_rect));
    assert!(self.current_rect.contains_rect(draw_rect));
    // Draw the scratch canvas to the destination canvas.
    dest.draw_image_with_html_canvas_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
      // &scratch_canvas,
      // (draw_rect.pos.0 - self.current_rect.pos.0) as f64,
      // (draw_rect.pos.1 - self.current_rect.pos.1) as f64,
      // draw_rect.size.0 as f64,
      // draw_rect.size.1 as f64,
      &scratch_canvas,
      (draw_rect.pos.0 - self.current_rect.pos.0) as f64,
      (draw_rect.pos.1 - self.current_rect.pos.1) as f64,
      draw_rect.size.0 as f64,
      draw_rect.size.1 as f64,
      0.0,
      0.0,
      draw_rect.size.0 as f64,
      draw_rect.size.1 as f64,
    );
  }
}
