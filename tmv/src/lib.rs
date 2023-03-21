use std::{
  cell::Cell,
  collections::{HashMap, HashSet},
  rc::Rc,
};

use collision::{
  CollisionWorld, PhysicsKind, PhysicsObjectHandle, BASIC_GROUP, PLAYER_GROUP, WALLS_GROUP,
};
use game_maps::GameMap;
use js_sys::Array;
use math::{Rect, Vec2};
use rapier2d::{prelude::{ColliderHandle, InteractionGroups, QueryFilter, Shape, Isometry, Cuboid}, na::Vector2};
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

pub type EntityId = i32;

#[derive(Debug, Clone, Serialize)]
pub struct CharState {
  pub save_point:  Vec2,
  pub hp:          i32,
  pub double_jump: bool,
  pub coins:       HashSet<EntityId>,
  pub rare_coins:  HashSet<EntityId>,
}

impl CharState {
  pub fn reset_hp(&mut self) {
    self.hp = 3;
  }
}

impl Default for CharState {
  fn default() -> Self {
    Self {
      save_point:  Vec2::default(),
      hp:          3,
      double_jump: false,
      coins:       HashSet::new(),
      rare_coins:  HashSet::new(),
    }
  }
}

pub enum GameObjectData {
  Coin {
    entity_id: EntityId,
  },
  RareCoin {
    entity_id: EntityId,
  },
  Spike,
  SavePoint,
  Shooter1 {
    orientation:  Vec2,
    cooldown:     Cell<f32>,
    shoot_period: f32,
  },
  Bullet {
    velocity: Vec2,
  },
  Water,
  Lava,
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
  game_map:            Rc<GameMap>,
  collision:           CollisionWorld,
  player_physics:      PhysicsObjectHandle,
  player_vel:          Vec2,
  grounded_last_frame: bool,
  grounded_recently:   f32,
  touching_water:      bool,
  submerged_in_water:  bool,
  char_state:          CharState,
  saved_char_state:    CharState,
  objects:             HashMap<ColliderHandle, GameObject>,
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

    let game_map =
      Rc::new(GameMap::from_resources(&resources, "/assets/map1.tmx").expect("Failed to load map"));

    let mut objects = HashMap::new();

    //let collision = Collision::from_game_map(&game_map);
    let mut collision = collision::CollisionWorld::new();

    let mut char_state = CharState::default();

    collision.load_game_map(&char_state, &game_map, &mut objects);
    let player_physics = collision.new_cuboid(
      PhysicsKind::Sensor,
      collision.spawn_point,
      PLAYER_SIZE,
      0.25,
    );
    char_state.save_point = collision.spawn_point;

    let draw_context = DrawContext {
      canvases: canvases.try_into().unwrap(),
      contexts: contexts.try_into().unwrap(),
      images,
      // FIXME: Don't hard-code this.
      tile_renderer: TileRenderer::new(game_map.clone(), Vec2(2048.0, 1536.0)),
    };

    Ok(Self {
      resources,
      draw_context,
      keys_held: HashSet::new(),
      jump_hit: false,
      camera_pos: Vec2::default(),
      game_map,
      collision,
      player_physics,
      player_vel: Vec2::default(),
      touching_water: false,
      submerged_in_water: false,
      grounded_last_frame: false,
      grounded_recently: 0.0,
      char_state: char_state.clone(),
      saved_char_state: char_state,
      objects,
      death_animation: 0.0,
    })
  }

  pub fn get_char_state(&self) -> JsValue {
    serde_wasm_bindgen::to_value(&self.char_state).unwrap()
  }

  pub fn get_info_line(&self) -> String {
    format!(
      "Coins: {:3}   Rare Coins: {:3}",
      self.char_state.coins.len(),
      self.char_state.rare_coins.len(),
    )
  }

  pub fn apply_input_event(&mut self, event: &str) -> Result<(), JsValue> {
    let event: InputEvent = serde_json::from_str(event).to_js_error()?;
    match event {
      InputEvent::KeyDown { key } => {
        if key == "ArrowUp" {
          self.jump_hit = true;
        }
        if key == " " && self.char_state.hp <= 0 {
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
    self.char_state = self.saved_char_state.clone();
    self.death_animation = 0.0;
    self.player_vel = Vec2::default();

    self.objects = HashMap::new();
    //let collision = Collision::from_game_map(&game_map);
    self.collision = collision::CollisionWorld::new();
    self.collision.load_game_map(&self.char_state, &self.game_map, &mut self.objects);
    self.player_physics = self.collision.new_cuboid(
      PhysicsKind::Sensor,
      self.char_state.save_point,
      PLAYER_SIZE,
      0.25,
    );
  }

  fn create_bullet(&mut self, location: Vec2, velocity: Vec2) {
    let physics_handle = self.collision.new_circle(
      collision::PhysicsKind::Dynamic,
      location,
      0.5,
      false,
      Some(InteractionGroups::new(
        BASIC_GROUP,
        WALLS_GROUP | PLAYER_GROUP,
      )),
    );
    // Set the interaction group.

    // Set the velocity.
    self.collision.set_velocity(&physics_handle, velocity);
    self.objects.insert(
      physics_handle.collider,
      GameObject {
        physics_handle,
        data: GameObjectData::Bullet { velocity },
      },
    );
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

    let filter = QueryFilter::default();

    self.touching_water = false;
    self.submerged_in_water = false;
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
              GameObjectData::Coin { entity_id } => {
                object.data = GameObjectData::DeleteMe;
                self.char_state.coins.insert(entity_id);
              }
              GameObjectData::RareCoin { entity_id } => {
                object.data = GameObjectData::DeleteMe;
                self.char_state.rare_coins.insert(entity_id);
              }
              GameObjectData::Spike => self.char_state.hp -= 100,
              GameObjectData::Bullet { .. } => {
                if self.char_state.hp > 0 {
                  self.char_state.hp -= 1;
                  object.data = GameObjectData::DeleteMe;
                }
              }
              GameObjectData::Water => {
                self.touching_water = true;
              }
              GameObjectData::Lava { .. } => {
                // FIXME: Properly handle lava.
                self.char_state.hp = 0;
              }
              GameObjectData::SavePoint => {
                let save_point = &self.objects[&handle].physics_handle;
                self.char_state.save_point =
                  self.collision.get_position(save_point).unwrap() + Vec2(0.0, -1.0);
                self.char_state.reset_hp();
                self.saved_char_state = self.char_state.clone();
              }
              GameObjectData::Shooter1 { .. } | GameObjectData::DeleteMe => {}
            }
          }
          true // Return `false` instead if we want to stop searching for other colliders that contain this point.
        },
      );
      // If we're touching water, check if we're submerged.
      let head_pos = Isometry::new(pos.translation.vector - Vector2::new(0.0, 1.0), 0.0);
      let head_shape = Cuboid::new(Vector2::new(PLAYER_SIZE.0 / 2.0, 0.5));
      self.collision.query_pipeline.intersections_with_shape(
        &self.collision.rigid_body_set,
        &self.collision.collider_set,
        &head_pos,
        &head_shape,
        filter,
        |handle| {
          if let Some(object) = self.objects.get_mut(&handle) {
            match object.data {
              GameObjectData::Water => {
                self.submerged_in_water = true;
                false
              }
              _ => true,
            }
          } else {
            true
          }
        },
      );
    }

    // Remove deleted objects.
    self.objects.retain(|_, v| match v.data {
      GameObjectData::DeleteMe => {
        self.collision.remove_object(v.physics_handle.clone());
        false
      }
      _ => true,
    });

    let mut calls = Vec::new();
    for object in self.objects.values_mut() {
      match &object.data {
        GameObjectData::Shooter1 {
          orientation,
          cooldown,
          shoot_period,
        } => {
          cooldown.set(cooldown.get() - dt);
          if cooldown.get() <= 0.0 {
            cooldown.set(*shoot_period);
            let velocity = 10.0 * *orientation;
            let physics_handle = object.physics_handle.clone();
            calls.push(move |this: &mut Self| {
              this.create_bullet(
                this.collision.get_position(&physics_handle).unwrap(),
                velocity,
              )
            });
          }
        }
        GameObjectData::Bullet { velocity } => {
          // If the object's velocity has changed, delete it.
          let vel = self.collision.get_velocity(&object.physics_handle).unwrap();
          if (vel - *velocity).length() > 0.01 {
            object.data = GameObjectData::DeleteMe;
          }
        }
        _ => {}
      }
    }
    for f in calls {
      f(self);
    }

    // Don't do anything else if we're dead.
    if self.char_state.hp <= 0 {
      self.death_animation += dt;
      return Ok(());
    }

    //self.char_state.hp = 3;
    //self.char_state = CharState::default();
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
        GameObjectData::Coin { .. }
        | GameObjectData::RareCoin { .. }
        | GameObjectData::Bullet { .. } => {
          let pos = self.collision.get_position(&object.physics_handle).unwrap_or(Vec2(0.0, 0.0));
          // Draw a circle, with a different color outside.
          match object.data {
            GameObjectData::Coin { .. } => {
              contexts[MAIN_LAYER].set_fill_style(&JsValue::from_str("#ff0"));
              contexts[MAIN_LAYER].set_stroke_style(&JsValue::from_str("#aa0"));
            }
            GameObjectData::RareCoin { .. } => {
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
        GameObjectData::SavePoint
        | GameObjectData::Water
        | GameObjectData::Lava
        | GameObjectData::Shooter1 { .. }
        | GameObjectData::Spike
        | GameObjectData::DeleteMe => {}
      }
    }

    // If we're under water, draw a blue rectangle over the screen.
    if self.submerged_in_water {
      contexts[MAIN_LAYER].set_fill_style(&JsValue::from_str("rgba(0, 0, 255, 0.5)"));
      contexts[MAIN_LAYER].fill_rect(0.0, 0.0, 800.0, 600.0);
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
