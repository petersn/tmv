use std::{collections::{HashMap, HashSet}, rc::Rc};

use collision::CollisionWorld;
use js_sys::Array;
use math::{Vec2, Rect};
use physics::Collision;
use serde::Deserialize;
use tile_rendering::TileRenderer;
use strum::IntoEnumIterator;
use wasm_bindgen::prelude::*;

use game_maps::GameMap;

pub mod game_maps;
pub mod tile_rendering;
pub mod math;
pub mod physics;
pub mod collision;

use tile_rendering::TILE_SIZE;

const UI_LAYER: usize = 0;
const MAIN_LAYER: usize = 1;
const BACKGROUND_LAYER: usize = 2;
const SCRATCH_LAYER: usize = 3;
//const PLAYER_SIZE: Vec2 = Vec2(1.25, 2.5);
const PLAYER_SIZE: Vec2 = Vec2(3.0, 3.0);

pub trait IntoJsError {
  type Ok;
  fn to_js_error(self) -> Result<Self::Ok, JsValue>;
}

impl<T, E: ToString> IntoJsError for Result<T, E> {
  type Ok = T;

  fn to_js_error(self) -> Result<T, JsValue> {
    self.map_err(|e| JsValue::from_str(&e.to_string()))
  }
}

impl<T> IntoJsError for Option<T> {
  type Ok = T;

  fn to_js_error(self) -> Result<T, JsValue> {
    self.ok_or_else(|| JsValue::from_str("Unwrapped a None"))
  }
}

#[derive(Debug, Clone, strum_macros::EnumIter, PartialEq, Eq, Hash)]
pub enum ImageResource {
  WorldProperties,
}

impl ImageResource {
  pub fn get_path(&self) -> &'static str {
    match self {
      ImageResource::WorldProperties => "/assets/images/colors_tileset.png",
    }
  }

  pub fn from_path(path: &str) -> Option<Self> {
    use strum::IntoEnumIterator;
    for image_resource in Self::iter() {
      if image_resource.get_path() == path {
        return Some(image_resource);
      }
    }
    None
  }
}

#[wasm_bindgen]
pub fn get_all_image_paths() -> Array {
  let mut array = Array::new();
  for image_resource in ImageResource::iter() {
    array.push(&JsValue::from_str(image_resource.get_path()));
  }
  array
}

#[derive(Debug, Clone, strum_macros::EnumIter, PartialEq, Eq, Hash)]
pub enum BinaryResource {
  Map1,
  WorldProperties,
}

impl BinaryResource {
  pub fn get_path(&self) -> &'static str {
    match self {
      BinaryResource::Map1 => "/assets/map1.tmx",
      BinaryResource::WorldProperties => "/assets/world_properties.tsx",
    }
  }
}

#[wasm_bindgen]
pub fn get_all_resource_names() -> Array {
  let mut array = Array::new();
  for binary_resource in BinaryResource::iter() {
    array.push(&JsValue::from_str(binary_resource.get_path()));
  }
  array
}

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = console)]
  pub fn log(s: &str);
}

#[wasm_bindgen]
pub fn get_wasm_version() -> String {
  #[cfg(debug_assertions)]
  return format!("v{}-debug", env!("CARGO_PKG_VERSION"));

  #[cfg(not(debug_assertions))]
  return format!("v{}", env!("CARGO_PKG_VERSION"));
}

struct DrawContext {
  canvases:      [web_sys::HtmlCanvasElement; 4],
  contexts:      [web_sys::CanvasRenderingContext2d; 4],
  images:        HashMap<ImageResource, web_sys::HtmlImageElement>,
  tile_renderer: TileRenderer,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum InputEvent {
  KeyDown {
    key: String,
  },
  KeyUp {
    key: String,
  },
}

#[wasm_bindgen]
pub struct GameState {
  resources:    HashMap<String, Vec<u8>>,
  draw_context: DrawContext,
  keys_held:    HashSet<String>,
  camera_pos:   Vec2,
  collision:    Collision,
  coll_world:   CollisionWorld,
  player_pos:   Vec2,
  player_vel:   Vec2,
}

#[wasm_bindgen]
impl GameState {
  #[wasm_bindgen(constructor)]
  pub fn new(resources: JsValue) -> Result<GameState, JsValue> {
    console_error_panic_hook::set_once();
    let resources = serde_wasm_bindgen::from_value(resources).unwrap();

    crate::log("Setting up game state");
    let document = web_sys::window().unwrap().document().to_js_error()?;
    let mut images = HashMap::new();
    for image_resource in ImageResource::iter() {
      let image = document.get_element_by_id(image_resource.get_path()).to_js_error()?;
      let image = image.dyn_into::<web_sys::HtmlImageElement>()?;
      images.insert(image_resource, image);
    }
    crate::log("... 0");

    let mut canvases = Vec::new();
    let mut contexts = Vec::new();
    for (i, path) in [
      "uiCanvas",
      "mainCanvas",
      "backgroundCanvas",
      "scratchCanvas",
    ]
    .iter()
    .enumerate()
    {
      let canvas = document.get_element_by_id(path).to_js_error()?;
      let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
      let context2d =
        canvas.get_context("2d")?.to_js_error()?.dyn_into::<web_sys::CanvasRenderingContext2d>()?;
      canvases.push(canvas);
      contexts.push(context2d);
    }
    crate::log("... 1");

    let game_map = Rc::new(
      GameMap::from_resources(&resources, "/assets/map1.tmx").expect("Failed to load map"),
    );

    let collision = Collision::from_game_map(&game_map);
    let mut physics = collision::CollisionWorld::new();
    physics.load_game_map(&game_map);


    crate::log("... 2");

    let draw_context = DrawContext {
      canvases: canvases.try_into().unwrap(),
      contexts: contexts.try_into().unwrap(),
      images,
      // FIXME: Don't hard-code this.
      tile_renderer: TileRenderer::new(game_map, Vec2(2048.0, 1536.0)),
    };
    crate::log("... 3");

    Ok(Self {
      resources,
      draw_context,
      keys_held: HashSet::new(),
      camera_pos: Vec2::default(),
      player_pos: Vec2::default(),
      player_vel: Vec2::default(),
      collision,
      coll_world: CollisionWorld::new(),
    })
  }

  pub fn apply_input_event(&mut self, event: &str) -> Result<(), JsValue> {
    let event: InputEvent = serde_json::from_str(event).to_js_error()?;
    match event {
      InputEvent::KeyDown { key } => {
        self.keys_held.insert(key);
      }
      InputEvent::KeyUp { key } => {
        self.keys_held.remove(&key);
      }
    }
    Ok(())
  }

  pub fn step(&mut self, dt: f32) -> Result<(), JsValue> {
    self.player_vel.1 += 1.0 * dt;
    // let (new_player_pos, collision_happened) = self.collision.try_move_rect(Rect {
    //   pos: self.player_pos,
    //   size: PLAYER_SIZE,
    // }, self.player_vel);
    // if collision_happened {
    //   self.player_vel.1 = 0.0;
    // }
    let (new_pos, has_collision) = self.collision.try_move_rect(
      Rect {
        pos: self.player_pos,
        size: PLAYER_SIZE,
      },
      self.player_vel,
    );
    self.player_pos = new_pos;
    if has_collision {
      self.player_vel.1 = 0.0;
    }
    if self.keys_held.contains("ArrowLeft") {
      self.player_vel.0 -= 1.0 * dt;
    }
    if self.keys_held.contains("ArrowRight") {
      self.player_vel.0 += 1.0 * dt;
    }
    if self.keys_held.contains("ArrowUp") {
      self.player_vel.1 -= 10.0;
    }
    Ok(())
  }

  pub fn draw_frame(&mut self) -> Result<bool, JsValue> {
    let DrawContext {
      canvases,
      contexts,
      images,
      tile_renderer,
    } = &mut self.draw_context;
    // contexts[BACKGROUND_LAYER].begin_path();
    // contexts[BACKGROUND_LAYER].move_to(10.0, 10.0);
    // contexts[BACKGROUND_LAYER].line_to(100.0 * rand::random::<f64>(), 100.0);
    // contexts[BACKGROUND_LAYER].stroke();

    crate::log("Drawing frame");

    // Recenter the gamera.
    self.camera_pos = Vec2(self.player_pos.0 - 400.0 / TILE_SIZE, self.player_pos.1 - 300.0 / TILE_SIZE);

    // Draw the game background.
    let draw_rect = Rect {
      pos:  TILE_SIZE * self.camera_pos,
      size: Vec2(800.0, 600.0),
    };
    tile_renderer.draw(
      draw_rect,
      &contexts[BACKGROUND_LAYER],
      images,
      &canvases[SCRATCH_LAYER],
      &contexts[SCRATCH_LAYER],
    );

    // Clear the main layer.
    contexts[MAIN_LAYER].clear_rect(0.0, 0.0, 800.0, 600.0);

    // Draw a red rectangle for the player.
    contexts[MAIN_LAYER].set_fill_style(&JsValue::from_str("red"));
    contexts[MAIN_LAYER].fill_rect(
      (TILE_SIZE * self.player_pos.0 - TILE_SIZE * self.camera_pos.0) as f64,
      (TILE_SIZE * self.player_pos.1 - TILE_SIZE * self.camera_pos.1) as f64,
      (TILE_SIZE * PLAYER_SIZE.0) as f64,
      (TILE_SIZE * PLAYER_SIZE.1) as f64,
    );

    // // Draw all of the game objects.
    // for game_object in self.game_world.game_objects.values() {
    //   let draw_info = match &game_object.draw_info {
    //     Some(draw_info) => draw_info,
    //     None => continue,
    //   };
    //   let pos = game_object.get_position(&self.game_world);
    //   contexts[MAIN_LAYER]
    //     .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
    //       &images.get(&draw_info.image).expect("Image not found"),
    //       draw_info.x as f64,
    //       draw_info.y as f64,
    //       draw_info.width as f64,
    //       draw_info.height as f64,
    //       (TILE_SIZE * (pos.0 - self.camera_pos.0)) as f64,
    //       (-TILE_SIZE * (pos.1 - self.camera_pos.1)) as f64,
    //       50.0 * game_object.draw_half_dims.0 as f64,
    //       50.0 * game_object.draw_half_dims.1 as f64,
    //     )?;
    // }

    // Copy a bit of the sprite sheet to the canvas
    // for i in 0..1_000 {
    //   contexts[MAIN_LAYER].draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
    //     &images[&ImageResource::MainSpriteSheet],
    //     200.0 * rand::random::<f64>(),
    //     200.0 * rand::random::<f64>(),
    //     32.0,
    //     32.0,
    //     200.0 * rand::random::<f64>(),
    //     200.0 * rand::random::<f64>(),
    //     32.0,
    //     32.0,
    //   )?;
    // }
    Ok(true)
  }
}
