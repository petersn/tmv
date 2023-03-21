use std::{
  collections::{HashMap, HashSet},
  rc::Rc, cell::Cell,
};

use collision::{CollisionWorld, PhysicsObjectHandle, PhysicsKind};
use game_maps::GameMap;
use js_sys::Array;
use math::{Rect, Vec2};
use rapier2d::prelude::{ColliderHandle, QueryFilter, Shape};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use tile_rendering::TileRenderer;
use tiled::ObjectLayerData;
use wasm_bindgen::prelude::*;

pub mod game_maps;
pub mod math;
pub mod tile_rendering;
//pub mod physics;
pub mod camera;
pub mod collision;

use tile_rendering::TILE_SIZE;

const UI_LAYER: usize = 0;
const MAIN_LAYER: usize = 1;
const BACKGROUND_LAYER: usize = 2;
const SCRATCH_LAYER: usize = 3;
const PLAYER_SIZE: Vec2 = Vec2(1.25, 2.5);
const JUMP_GRACE_PERIOD: f32 = 0.1;
//const PLAYER_SIZE: Vec2 = Vec2(3.0, 3.0);

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
  MainTiles,
}

impl ImageResource {
  pub fn get_path(&self) -> &'static str {
    match self {
      ImageResource::WorldProperties => "/assets/images/colors_tileset.png",
      ImageResource::MainTiles => "/assets/images/main_tiles.png",
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
  MainTiles,
}

impl BinaryResource {
  pub fn get_path(&self) -> &'static str {
    match self {
      BinaryResource::Map1 => "/assets/map1.tmx",
      BinaryResource::WorldProperties => "/assets/world_properties.tsx",
      BinaryResource::MainTiles => "/assets/main_tiles.tsx",
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
  KeyDown { key: String },
  KeyUp { key: String },
}

#[derive(Serialize)]
struct PowerUpState {
  pub double_jump: bool,
  pub coins:       u32,
  pub rare_coins:  u32,
}

impl Default for PowerUpState {
  fn default() -> Self {
    Self {
      double_jump: false,
      coins:       0,
      rare_coins:  0,
    }
  }
}

pub enum GameObjectData {
  Coin,
  RareCoin,
  Spike,
  Shooter1 { orientation: Vec2, cooldown: Cell<f32>, shoot_period: f32 },
  Bullet { velocity: Vec2 },
  DeleteMe,
}

pub struct GameObject {
  pub physics_handle: PhysicsObjectHandle,
  pub data:           GameObjectData,
}

#[wasm_bindgen]
pub struct GameState {
  resources:           HashMap<String, Vec<u8>>,
  draw_context:        DrawContext,
  keys_held:           HashSet<String>,
  jump_hit:            bool,
  camera_pos:          Vec2,
  collision:           CollisionWorld,
  player_physics:      PhysicsObjectHandle,
  player_vel:          Vec2,
  grounded_last_frame: bool,
  grounded_recently:   f32,
  powerup_state:       PowerUpState,
  objects:             HashMap<ColliderHandle, GameObject>,
  hp:                  i32,
  death_animation:     f32,
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

    let game_map =
      Rc::new(GameMap::from_resources(&resources, "/assets/map1.tmx").expect("Failed to load map"));

    let mut objects = HashMap::new();

    //let collision = Collision::from_game_map(&game_map);
    let mut collision = collision::CollisionWorld::new();
    collision.load_game_map(&game_map, &mut objects);
    let player_physics = collision.new_cuboid(
      PhysicsKind::Sensor,
      collision.spawn_point,
      PLAYER_SIZE,
      0.25,
    );

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
      jump_hit: false,
      camera_pos: Vec2::default(),
      collision,
      player_physics,
      player_vel: Vec2::default(),
      grounded_last_frame: false,
      grounded_recently: 0.0,
      powerup_state: PowerUpState::default(),
      objects,
      hp: 3,
      death_animation: 0.0,
    })
  }

  pub fn get_powerup_state(&self) -> JsValue {
    serde_wasm_bindgen::to_value(&self.powerup_state).unwrap()
  }

  pub fn get_info_line(&self) -> String {
    format!(
      "Coins: {:3}   Rare Coins: {:3}",
      self.powerup_state.coins, self.powerup_state.rare_coins,
    )
  }

  pub fn apply_input_event(&mut self, event: &str) -> Result<(), JsValue> {
    let event: InputEvent = serde_json::from_str(event).to_js_error()?;
    match event {
      InputEvent::KeyDown { key } => {
        if key == "ArrowUp" {
          self.jump_hit = true;
        }
        if key == " " && self.hp <= 0 {
          self.respawn();
        }
        self.keys_held.insert(key);
      }
      InputEvent::KeyUp { key } => {
        self.keys_held.remove(&key);
      }
    }
    Ok(())
  }

  pub fn respawn(&mut self) {
    self.hp = 3;
    self.death_animation = 0.0;
    self.collision.set_position(&self.player_physics, self.collision.spawn_point);
    self.player_vel = Vec2::default();
  }

  fn create_bullet(&mut self, location: Vec2, velocity: Vec2) -> (ColliderHandle, GameObject) {
    let physics_handle = self.collision.new_sensor_circle(
      collision::PhysicsKind::Sensor,
      location,
      0.5,
    );
    (
      physics_handle.collider,
      GameObject {
        physics_handle,
        data: GameObjectData::Bullet { velocity },
      },
    )
  }

  pub fn step(&mut self, dt: f32) -> Result<(), JsValue> {
    //self.player_vel.1 += 1.0 * dt;
    // let (new_player_pos, collision_happened) = self.collision.try_move_rect(Rect {
    //   pos: self.player_pos,
    //   size: PLAYER_SIZE,
    // }, self.player_vel);
    // if collision_happened {
    //   self.player_vel.1 = 0.0;
    // }
    // let (new_pos, has_collision) = self.collision.try_move_rect(
    //   Rect {
    //     pos: self.player_pos,
    //     size: PLAYER_SIZE,
    //   },
    //   self.player_vel,
    // );
    self.collision.step(dt);
    // while let Ok(collision_event) = self.collision.collision_recv.try_recv() {
    //   // Handle the collision event.
    //   crate::log(&format!("Received collision event: {:?}", collision_event));
    // }
    // while let Ok(contact_force_event) = self.collision.contact_force_recv.try_recv() {
    //   // Handle the trigger event.
    //   crate::log(&format!("Received trigger event: {:?}", contact_force_event));
    // }

    let mut new_objects = Vec::new();
    for (handle, object) in self.objects {
      match &object.data {
        GameObjectData::Shooter1 { orientation: _, cooldown, shoot_period } => {
          cooldown.set(cooldown.get() - dt);
          if cooldown.get() <= 0.0 {
            cooldown.set(*shoot_period);
            new_objects.push(self.create_bullet(
              self.collision.get_position(&object.physics_handle).unwrap(),
              Vec2(0.0, 1.0),
            ));
          }
        }
        _ => {}
      }
    }
    for (handle, object) in new_objects {
      self.objects.insert(handle, object);
    }

    // Don't do anything else if we're dead.
    if self.hp <= 0 {
      self.death_animation += dt;
      return Ok(());
    }

    let filter = QueryFilter::default();

    // Get the shape and pos of the player collider.
    if let Some((shape, pos)) = self.collision.get_shape_and_position(&self.player_physics) {
      self.collision.query_pipeline.intersections_with_shape(
        &self.collision.rigid_body_set,
        &self.collision.collider_set,
        pos,
        shape,
        filter,
        |handle| {
          if let Some(object) = self.objects.get_mut(&handle) {
            match object.data {
              GameObjectData::Coin => {
                object.data = GameObjectData::DeleteMe;
                self.powerup_state.coins += 1;
              }
              GameObjectData::RareCoin => {
                object.data = GameObjectData::DeleteMe;
                self.powerup_state.rare_coins += 1;
              }
              GameObjectData::Spike => self.hp -= 100,
              GameObjectData::Bullet { .. } => {
                self.hp -= 1;
                object.data = GameObjectData::DeleteMe;
              }
              GameObjectData::Shooter1 { .. } | GameObjectData::DeleteMe => {}
            }
          }
          true // Return `false` instead if we want to stop searching for other colliders that contain this point.
        },
      );
    }
    // Remove deleted objects.
    self.objects.retain(|_, v| match v.data {
      GameObjectData::DeleteMe => false,
      _ => true,
    });

    //self.hp = 3;
    //self.powerup_state = PowerUpState::default();
    //self.collision.set_position(&self.player_physics, self.collision.spawn_point);

    // self.player_pos = new_pos;
    // if has_collision {
    //   self.player_vel.1 = 0.0;
    // }
    // if self.keys_held.contains("ArrowLeft") {
    //   self.player_vel.0 -= 1.0 * dt;
    // }
    // if self.keys_held.contains("ArrowRight") {
    //   self.player_vel.0 += 1.0 * dt;
    // }
    // if self.keys_held.contains("ArrowUp") {
    //   self.player_vel.1 -= 10.0;
    // }
    let horizontal_decay_factor = match self.grounded_last_frame {
      true => 0.5f32.powf(60.0 * dt),
      false => 0.5f32.powf(5.0 * dt),
    };
    let horizontal_dv = match self.grounded_last_frame {
      true => 150.0,
      false => 50.0,
    };
    if self.keys_held.contains("ArrowLeft") {
      self.player_vel.0 -= horizontal_dv * dt;
    } else if self.player_vel.0 < 0.0 {
      self.player_vel.0 *= horizontal_decay_factor;
    }
    if self.keys_held.contains("ArrowRight") {
      self.player_vel.0 += horizontal_dv * dt;
    } else if self.player_vel.0 > 0.0 {
      self.player_vel.0 *= horizontal_decay_factor;
    }

    if self.player_vel.1 < 0.0 && !self.keys_held.contains("ArrowUp") {
      self.player_vel.1 *= 0.01f32.powf(dt);
    }

    self.player_vel.0 = self.player_vel.0.max(-15.0).min(15.0);
    self.player_vel.1 = (self.player_vel.1 + 60.0 * dt).min(30.0);
    let effective_motion = self.collision.move_object_with_character_controller(
      dt,
      &self.player_physics,
      dt * self.player_vel,
    );
    // For some reason effective_motion.grounded seems to always be false,
    // so we instead consider ourselves grounded if we didn't move the full requested amount in y.
    let grounded =
      self.player_vel.1 > 0.0 && effective_motion.translation.y < dt * self.player_vel.1 * 0.95;
    if grounded {
      self.player_vel.1 = self.player_vel.1.min(0.0);
    }
    let blocked_to_left =
      self.player_vel.0 < 0.0 && effective_motion.translation.x > dt * self.player_vel.0 * 0.95;
    let blocked_to_right =
      self.player_vel.0 > 0.0 && effective_motion.translation.x < dt * self.player_vel.0 * 0.95;
    let blocked_to_top =
      self.player_vel.1 < 0.0 && effective_motion.translation.y > dt * self.player_vel.1 * 0.95;
    if blocked_to_left {
      self.player_vel.0 = self.player_vel.0.max(0.0);
    }
    if blocked_to_right {
      self.player_vel.0 = self.player_vel.0.min(0.0);
    }
    if blocked_to_top {
      self.player_vel.1 = self.player_vel.1.max(0.0);
    }
    if grounded {
      self.grounded_recently = JUMP_GRACE_PERIOD;
    }
    if self.jump_hit && self.grounded_recently > 0.0 {
      let abs_horizontal = self.player_vel.0.abs();
      self.player_vel.1 = -22.0 - 0.1 * abs_horizontal;
      self.grounded_recently = 0.0;
    }
    self.jump_hit = false;
    self.grounded_last_frame = grounded;
    self.grounded_recently = (self.grounded_recently - dt).max(0.0);
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

    let player_pos = self.collision.get_position(&self.player_physics).unwrap_or(Vec2(0.0, 0.0));

    // Recenter the gamera.
    self.camera_pos = Vec2(
      player_pos.0 - 400.0 / TILE_SIZE,
      player_pos.1 - 400.0 / TILE_SIZE,
    );

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
      (TILE_SIZE * (player_pos.0 - self.camera_pos.0 - PLAYER_SIZE.0 / 2.0)) as f64,
      (TILE_SIZE
        * (player_pos.1 - self.camera_pos.1 - PLAYER_SIZE.1 / 2.0 + 10.0 * self.death_animation))
        as f64,
      (TILE_SIZE * PLAYER_SIZE.0) as f64,
      (TILE_SIZE * (PLAYER_SIZE.1 - 10.0 * self.death_animation).max(0.0)) as f64,
    );

    // Draw all of the objects.
    for (handle, object) in &self.objects {
      match object.data {
        GameObjectData::Coin | GameObjectData::RareCoin | GameObjectData::Bullet { .. } => {
          let pos = self.collision.get_position(&object.physics_handle).unwrap_or(Vec2(0.0, 0.0));
          // Draw a circle, with a different color outside.
          match object.data {
            GameObjectData::Coin => {
              contexts[MAIN_LAYER].set_fill_style(&JsValue::from_str("#ff0"));
              contexts[MAIN_LAYER].set_stroke_style(&JsValue::from_str("#aa0"));
            }
            GameObjectData::RareCoin => {
              contexts[MAIN_LAYER].set_fill_style(&JsValue::from_str("#04a"));
              contexts[MAIN_LAYER].set_stroke_style(&JsValue::from_str("#026"));
            }
            GameObjectData::Bullet { .. } => {
              contexts[MAIN_LAYER].set_fill_style(&JsValue::from_str("#f00"));
              contexts[MAIN_LAYER].set_stroke_style(&JsValue::from_str("#a00"));
            }
            _ => unreachable!(),
          }
          contexts[MAIN_LAYER].set_line_width(5.0);
          contexts[MAIN_LAYER].begin_path();
          contexts[MAIN_LAYER].arc(
            (TILE_SIZE * (pos.0 - self.camera_pos.0)) as f64,
            (TILE_SIZE * (pos.1 - self.camera_pos.1)) as f64,
            (TILE_SIZE / 2.0) as f64,
            0.0,
            2.0 * std::f64::consts::PI,
          );
          contexts[MAIN_LAYER].fill();
          contexts[MAIN_LAYER].stroke();
        }
        GameObjectData::Shooter1 { .. } | GameObjectData::Spike | GameObjectData::DeleteMe => {}
      }
    }

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
