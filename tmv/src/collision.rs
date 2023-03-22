use std::{
  cell::Cell,
  collections::{HashMap, HashSet},
  rc::Rc,
};

use rapier2d::{
  control::{EffectiveCharacterMovement, KinematicCharacterController},
  na::{Isometry2, Vector2},
  prelude::*,
};
use tiled::Chunk;

use crate::{
  game_maps::GameMap, math::Vec2, tile_rendering::TILE_SIZE, CharState, GameObject, GameObjectData,
};

pub enum PhysicsKind {
  Static,
  Dynamic,
  Kinematic,
  Sensor,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhysicsObjectHandle {
  pub rigid_body: Option<RigidBodyHandle>,
  pub collider:   ColliderHandle,
}

pub const BASIC_GROUP: Group = Group::GROUP_1;
pub const WALLS_GROUP: Group = Group::GROUP_2;
pub const PLAYER_GROUP: Group = Group::GROUP_3;
pub const WATER_GROUP: Group = Group::GROUP_4;
pub const LAVA_GROUP: Group = Group::GROUP_5;
pub const PLATFORMS_GROUP: Group = Group::GROUP_6;

pub const BASIC_INT_GROUPS: InteractionGroups = InteractionGroups::new(BASIC_GROUP, Group::ALL);
pub const WALLS_INT_GROUPS: InteractionGroups = InteractionGroups::new(WALLS_GROUP, Group::ALL);

// We make a struct to hold all the physics objects.
pub struct CollisionWorld {
  pub rigid_body_set:         RigidBodySet,
  pub collider_set:           ColliderSet,
  pub gravity:                Vector<f32>,
  pub integration_parameters: IntegrationParameters,
  pub physics_pipeline:       PhysicsPipeline,
  pub query_pipeline:         QueryPipeline,
  pub island_manager:         IslandManager,
  pub broad_phase:            BroadPhase,
  pub narrow_phase:           NarrowPhase,
  pub impulse_joint_set:      ImpulseJointSet,
  pub multibody_joint_set:    MultibodyJointSet,
  pub ccd_solver:             CCDSolver,
  pub physics_hooks:          (),
  pub event_handler:          (), // ChannelEventCollector,
  pub char_controller:        KinematicCharacterController,
  pub spawn_point:            Vec2,
  // pub collision_recv:         crossbeam::channel::Receiver<CollisionEvent>,
  // pub contact_force_recv:     crossbeam::channel::Receiver<ContactForceEvent>,
}

impl CollisionWorld {
  pub fn new() -> Self {
    // let (collision_send, collision_recv) = crossbeam::channel::unbounded();
    // let (contact_force_send, contact_force_recv) = crossbeam::channel::unbounded();
    Self {
      rigid_body_set:         RigidBodySet::new(),
      collider_set:           ColliderSet::new(),
      gravity:                vector![0.0, 0.0],
      integration_parameters: IntegrationParameters::default(),
      physics_pipeline:       PhysicsPipeline::new(),
      query_pipeline:         QueryPipeline::new(),
      island_manager:         IslandManager::new(),
      broad_phase:            BroadPhase::new(),
      narrow_phase:           NarrowPhase::new(),
      impulse_joint_set:      ImpulseJointSet::new(),
      multibody_joint_set:    MultibodyJointSet::new(),
      ccd_solver:             CCDSolver::new(),
      physics_hooks:          (),
      event_handler:          (), //ChannelEventCollector::new(collision_send, contact_force_send),
      char_controller:        KinematicCharacterController::default(),
      spawn_point:            Vec2::default(),
      // collision_recv,
      // contact_force_recv,
    }
  }

  pub fn load_game_map(
    &mut self,
    char_state: &CharState,
    game_map: &GameMap,
    objects: &mut HashMap<ColliderHandle, GameObject>,
  ) {
    let mut all_solid_cells = HashSet::new();

    // The main layer includes some objects, like spikes.
    let main_layer = game_map.map.layers().find(|l| l.name == "Main").unwrap();
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
                let base_tile = tile.get_tile().unwrap();
                let user_type: &str = match &base_tile.user_type {
                  Some(s) => s,
                  _ => "",
                };
                match user_type {
                  "nonsolid" | "marker" => {}
                  "" => {
                    all_solid_cells.insert(tile_pos);
                  }
                  _ => panic!("Unknown user_type: {}", user_type),
                }

                let name: &str = match base_tile.properties.get("name") {
                  Some(tiled::PropertyValue::StringValue(s)) => s,
                  _ => continue,
                };
                let mut make_circle = |radius| {
                  self.new_circle(
                    PhysicsKind::Sensor,
                    Vec2(tile_pos.0 as f32 + 0.5, tile_pos.1 as f32 + 0.5),
                    radius,
                    true,
                    None,
                  )
                };
                let mut orientation = Vec2(1.0, 0.0);
                let mut is_mirrored = false;
                if tile.flip_d {
                  (orientation.0, orientation.1) = (orientation.1, orientation.0);
                  is_mirrored ^= true;
                }
                if tile.flip_v {
                  orientation.1 *= -1.0;
                  is_mirrored ^= true;
                }
                if tile.flip_h {
                  orientation.0 *= -1.0;
                  is_mirrored ^= true;
                }
                let entity_id = 1_000_000 * tile_pos.1 + tile_pos.0;
                match name {
                  "coin" | "rare_coin" | "hp_up" => {
                    // If the player has already picked up this coin, skip it.
                    if char_state.coins.contains(&entity_id)
                      | char_state.rare_coins.contains(&entity_id)
                      | char_state.hp_ups.contains(&entity_id)
                    {
                      continue;
                    }
                  }
                  "powerup" => {
                    let power_up: &str = match base_tile.properties.get("powerup") {
                      Some(tiled::PropertyValue::StringValue(s)) => s,
                      _ => panic!("Powerup without powerup property"),
                    };
                    // If the player has already picked up this powerup, skip it.
                    if char_state.power_ups.contains(power_up) {
                      continue;
                    }
                  }
                  _ => {}
                }
                match name {
                  "water" => {
                    let handle = make_circle(0.45);
                    objects.insert(
                      handle.collider,
                      GameObject {
                        physics_handle: handle,
                        data:           GameObjectData::Water,
                      },
                    );
                  }
                  "lava" => {
                    let handle = make_circle(0.45);
                    objects.insert(
                      handle.collider,
                      GameObject {
                        physics_handle: handle,
                        data:           GameObjectData::Lava,
                      },
                    );
                  }
                  // Coin
                  "coin" => {
                    let handle = make_circle(0.45);
                    objects.insert(
                      handle.collider,
                      GameObject {
                        physics_handle: handle,
                        data:           GameObjectData::Coin { entity_id },
                      },
                    );
                  }
                  // Rare coin
                  "rare_coin" => {
                    let handle = make_circle(0.45);
                    objects.insert(
                      handle.collider,
                      GameObject {
                        physics_handle: handle,
                        data:           GameObjectData::RareCoin { entity_id },
                      },
                    );
                  }
                  "hp_up" => {
                    let handle = make_circle(0.45);
                    objects.insert(
                      handle.collider,
                      GameObject {
                        physics_handle: handle,
                        data:           GameObjectData::HpUp { entity_id },
                      },
                    );
                  }
                  "powerup" => {
                    let power_up: &str = match base_tile.properties.get("powerup") {
                      Some(tiled::PropertyValue::StringValue(s)) => s,
                      _ => panic!("Powerup without powerup property"),
                    };
                    let handle = make_circle(0.45);
                    objects.insert(
                      handle.collider,
                      GameObject {
                        physics_handle: handle,
                        data:           GameObjectData::PowerUp {
                          power_up: power_up.to_string(),
                        },
                      },
                    );
                  }
                  "spike" => {
                    let handle = make_circle(0.2);
                    objects.insert(
                      handle.collider,
                      GameObject {
                        physics_handle: handle,
                        data:           GameObjectData::Spike,
                      },
                    );
                  }
                  "shooter1" => {
                    let handle = make_circle(0.45);
                    objects.insert(
                      handle.collider,
                      GameObject {
                        physics_handle: handle,
                        data:           GameObjectData::Shooter1 {
                          orientation,
                          cooldown: Cell::new(1.25),
                          shoot_period: 1.25,
                        },
                      },
                    );
                  }
                  "beehive" => {
                    let handle = make_circle(0.45);
                    objects.insert(
                      handle.collider,
                      GameObject {
                        physics_handle: handle,
                        data:           GameObjectData::Beehive {
                          cooldown: Cell::new(0.0),
                        },
                      },
                    );
                  }
                  "coin_wall" => {
                    let count: i32 = match base_tile.properties.get("count") {
                      Some(tiled::PropertyValue::IntValue(count)) => *count,
                      Some(_) => panic!("count must be an int"),
                      _ => continue,
                    };
                    let handle = self.new_cuboid(
                      PhysicsKind::Static,
                      Vec2(tile_pos.0 as f32 + 0.5, tile_pos.1 as f32 + 0.5),
                      Vec2(0.6, 0.6),
                      0.05,
                      false,
                      WALLS_INT_GROUPS,
                    );
                    objects.insert(
                      handle.collider,
                      GameObject {
                        physics_handle: handle,
                        data:           GameObjectData::CoinWall { count },
                      },
                    );
                  }
                  "stone" => {
                    let handle = self.new_cuboid(
                      PhysicsKind::Static,
                      Vec2(tile_pos.0 as f32 + 0.5, tile_pos.1 as f32 + 0.5),
                      Vec2(1.0, 1.0),
                      0.05,
                      false,
                      WALLS_INT_GROUPS,
                    );
                    objects.insert(
                      handle.collider,
                      GameObject {
                        physics_handle: handle,
                        data:           GameObjectData::Stone,
                      },
                    );
                  }
                  "save_left" => {
                    let handle = make_circle(0.45);
                    // Because only the left tile in the save point gets an entity, we shift it over half a tile.
                    self.set_position(
                      &handle,
                      Vec2(tile_pos.0 as f32 + 1.0, tile_pos.1 as f32 + 0.5),
                    );
                    objects.insert(
                      handle.collider,
                      GameObject {
                        physics_handle: handle,
                        data:           GameObjectData::SavePoint,
                      },
                    );
                  }
                  "platform" => {
                    let handle = self.new_static_walls(
                      (tile_pos.0 as f32, tile_pos.1 as f32),
                      &[(0.0, 0.3), (1.0, 0.3)],
                      InteractionGroups {
                        memberships: PLATFORMS_GROUP,
                        filter:      Group::ALL,
                      },
                    );
                    objects.insert(
                      handle.collider,
                      GameObject {
                        physics_handle: handle,
                        data:           GameObjectData::Platform {
                          currently_solid: true,
                          y:               tile_pos.1 as f32 + 0.3,
                        },
                      },
                    );
                  }
                  "thwump" | "moving_platform" => {
                    let handle = self.new_cuboid(
                      PhysicsKind::Kinematic,
                      Vec2(tile_pos.0 as f32 + 0.5, tile_pos.1 as f32 + 0.5),
                      Vec2(3.0, 1.0),
                      0.05,
                      false,
                      WALLS_INT_GROUPS,
                    );
                    objects.insert(
                      handle.collider,
                      GameObject {
                        physics_handle: handle,
                        data:           match name {
                          "thwump" => GameObjectData::Thwump {
                            orientation,
                            state: crate::ThwumpState::Idle,
                          },
                          "moving_platform" => GameObjectData::MovingPlatform { orientation },
                          _ => unreachable!(),
                        },
                      },
                    );
                  }
                  "turn_laser" => {
                    let laser_origin = Vec2(tile_pos.0 as f32 + 0.5, tile_pos.1 as f32 + 0.5);
                    let handle = self.new_circle(
                      PhysicsKind::Static,
                      laser_origin,
                      0.45,
                      false,
                      Some(WALLS_INT_GROUPS),
                    );
                    objects.insert(
                      handle.collider,
                      GameObject {
                        physics_handle: handle,
                        data:           GameObjectData::TurnLaser {
                          is_mirrored,
                          angle: orientation.1.atan2(orientation.0),
                          hit_point: laser_origin,
                        },
                      },
                    );
                  }
                  "vanish_block" => {
                    let handle = self.new_cuboid(
                      PhysicsKind::Static,
                      Vec2(tile_pos.0 as f32 + 0.5, tile_pos.1 as f32 + 0.5),
                      Vec2(1.0, 1.0),
                      0.05,
                      false,
                      WALLS_INT_GROUPS,
                    );
                    objects.insert(
                      handle.collider,
                      GameObject {
                        physics_handle: handle,
                        data:           GameObjectData::VanishBlock {
                          vanish_timer: 1.0,
                          is_solid:     true,
                        },
                      },
                    );
                  }
                  "spawn" => self.spawn_point = Vec2(tile_pos.0 as f32, tile_pos.1 as f32),
                  _ => panic!("Unsupported tile name: {}", name),
                }
              }
            }
          }
        }
      }
      _ => panic!("Unsupported layer type: {:?}", main_layer.layer_type()),
    }

    // Add extra collision objects from the collision layer.
    let collision_layer = game_map.map.layers().find(|l| l.name == "Collision").unwrap();
    match collision_layer.layer_type() {
      tiled::LayerType::ObjectLayer(object_layer) => {
        for object in object_layer.objects() {
          match &object.shape {
            tiled::ObjectShape::Rect { width, height } => {
              let name: &str = match object.properties.get("name") {
                Some(tiled::PropertyValue::StringValue(s)) => s,
                _ => panic!("Rects must have a name property that's a string."),
              };
              match name {
                "interact" => {
                  let interaction_number = match object.properties.get("interaction") {
                    Some(tiled::PropertyValue::IntValue(i)) => *i,
                    _ => panic!("interact rects must have an interaction property."),
                  };
                  crate::log(&format!(
                    "Rect: {}x{} @ ({}, {})",
                    width, height, object.x, object.y
                  ));
                  // Create a new cuboid collider for this interaction.
                  let handle = self.new_cuboid(
                    PhysicsKind::Sensor,
                    Vec2(
                      (object.x + width / 2.0) / TILE_SIZE,
                      (object.y + height / 2.0) / TILE_SIZE,
                    ),
                    Vec2(width / TILE_SIZE, height / TILE_SIZE),
                    0.05,
                    false,
                    BASIC_INT_GROUPS,
                  );
                  objects.insert(
                    handle.collider,
                    GameObject {
                      physics_handle: handle,
                      data:           GameObjectData::Interaction { interaction_number },
                    },
                  );
                }
                _ => panic!("Unsupported rect name: {}", name),
              }
            }
            tiled::ObjectShape::Polyline { points } | tiled::ObjectShape::Polygon { points } => {
              //crate::log(&format!("Polygon: {:?} @ ({}, {})", points, object.x, object.y));
              let mut points =
                points.iter().map(|p| (p.0 / TILE_SIZE, p.1 / TILE_SIZE)).collect::<Vec<_>>();
              // If the shape is a polygon, we close it.
              if let tiled::ObjectShape::Polygon { .. } = object.shape {
                points.push(points[0]);
              }
              self.new_static_walls(
                (object.x / TILE_SIZE, object.y / TILE_SIZE),
                &points[..],
                WALLS_INT_GROUPS,
              );
            }
            _ => panic!("Unsupported object shape: {:?}", object.shape),
          }
        }
      }
      _ => panic!("Unsupported layer type"),
    }

    // We now generate walls from our solid cells.
    let min_x = all_solid_cells.iter().map(|c| c.0).min().unwrap();
    let max_x = all_solid_cells.iter().map(|c| c.0).max().unwrap();
    let min_y = all_solid_cells.iter().map(|c| c.1).min().unwrap();
    let max_y = all_solid_cells.iter().map(|c| c.1).max().unwrap();
    let mut walls: Vec<((i32, i32), (i32, i32))> = Vec::new();
    // Horizontal scans.
    for y in min_y..=max_y + 1 {
      let mut row_start: Option<i32> = None;
      for x in min_x..=max_x + 1 {
        let is_boundary = all_solid_cells.contains(&(x, y)) ^ all_solid_cells.contains(&(x, y - 1));
        match (is_boundary, row_start) {
          (true, None) => row_start = Some(x),
          (true, Some(_)) => {}
          (false, Some(start)) => {
            walls.push(((start, y), (x, y)));
            row_start = None;
          }
          (false, None) => {}
        }
      }
    }
    // Vertical scans.
    for x in min_x..=max_x + 1 {
      let mut row_start: Option<i32> = None;
      for y in min_y..=max_y + 1 {
        let is_boundary = all_solid_cells.contains(&(x, y)) ^ all_solid_cells.contains(&(x - 1, y));
        match (is_boundary, row_start) {
          (true, None) => row_start = Some(y),
          (true, Some(_)) => {}
          (false, Some(start)) => {
            walls.push(((x, start), (x, y)));
            row_start = None;
          }
          (false, None) => {}
        }
      }
    }
    crate::log(&format!("Found {} walls", walls.len()));
    // We now insert the walls into the physics world.
    let rigid_body = self.rigid_body_set.insert(
      RigidBodyBuilder::fixed()
        .position(Isometry::new(Vector2::new(0.0, 0.0), nalgebra::zero()))
        .build(),
    );
    let mut indices: Vec<[u32; 2]> = Vec::new();
    let mut idx = 0;
    for _ in 0..walls.len() {
      indices.push([idx, idx + 1]);
      idx += 2;
    }
    let mut vertices = Vec::new();
    for ((x1, y1), (x2, y2)) in walls {
      vertices.push(Point::new(x1 as f32, y1 as f32));
      vertices.push(Point::new(x2 as f32, y2 as f32));
    }
    self.collider_set.insert_with_parent(
      ColliderBuilder::polyline(vertices, Some(indices)).collision_groups(WALLS_INT_GROUPS),
      rigid_body,
      &mut self.rigid_body_set,
    );
  }

  pub fn new_static_walls(
    &mut self,
    xy: (f32, f32),
    segments: &[(f32, f32)],
    int_groups: InteractionGroups,
  ) -> PhysicsObjectHandle {
    println!("New static walls: {:?}", segments);
    let rigid_body = self.rigid_body_set.insert(
      RigidBodyBuilder::fixed()
        .position(Isometry::new(Vector2::new(xy.0, xy.1), nalgebra::zero()))
        .build(),
    );
    let mut indices: Vec<[u32; 2]> = Vec::new();
    let mut idx = 0;
    for _ in 0..segments.len() - 1 {
      indices.push([idx, idx + 1]);
      idx += 1;
    }
    let vertices: Vec<_> = segments.iter().map(|v| Point::new(v.0, v.1)).collect();
    let collider = self.collider_set.insert_with_parent(
      ColliderBuilder::polyline(vertices, Some(indices)).collision_groups(int_groups),
      rigid_body,
      &mut self.rigid_body_set,
    );
    PhysicsObjectHandle {
      rigid_body: None,
      collider,
    }
  }

  pub fn new_circle(
    &mut self,
    kind: PhysicsKind,
    position: Vec2,
    radius: f32,
    is_sensor: bool,
    int_groups: Option<InteractionGroups>,
  ) -> PhysicsObjectHandle {
    let rigid_body = match kind {
      PhysicsKind::Static => RigidBodyBuilder::fixed(),
      PhysicsKind::Dynamic => RigidBodyBuilder::dynamic(),
      PhysicsKind::Kinematic => RigidBodyBuilder::kinematic_velocity_based(),
      PhysicsKind::Sensor => RigidBodyBuilder::kinematic_position_based(),
    }
    .translation(vector![position.0, position.1])
    .build();
    let rigid_body = self.rigid_body_set.insert(rigid_body);
    let mut builder = ColliderBuilder::ball(radius).sensor(is_sensor);
    if let Some(int_groups) = int_groups {
      builder = builder.collision_groups(int_groups);
    }
    let collider =
      self.collider_set.insert_with_parent(builder, rigid_body, &mut self.rigid_body_set);
    PhysicsObjectHandle {
      rigid_body: Some(rigid_body),
      collider,
    }
  }

  // FIXME: Deduplicate with the above.
  pub fn new_cuboid(
    &mut self,
    kind: PhysicsKind,
    position: Vec2,
    size: Vec2,
    rounding: f32,
    is_sensor: bool,
    int_groups: InteractionGroups,
  ) -> PhysicsObjectHandle {
    let rigid_body = match kind {
      PhysicsKind::Static => RigidBodyBuilder::fixed(),
      PhysicsKind::Dynamic => RigidBodyBuilder::dynamic(),
      PhysicsKind::Kinematic => RigidBodyBuilder::kinematic_velocity_based(),
      PhysicsKind::Sensor => RigidBodyBuilder::kinematic_position_based(),
    }
    .translation(vector![position.0, position.1])
    .build();
    let rigid_body = self.rigid_body_set.insert(rigid_body);
    let collider = self.collider_set.insert_with_parent(
      ColliderBuilder::round_cuboid(size.0 / 2.0 - rounding, size.1 / 2.0 - rounding, rounding)
        .sensor(is_sensor)
        .collision_groups(int_groups),
      rigid_body,
      &mut self.rigid_body_set,
    );
    PhysicsObjectHandle {
      rigid_body: Some(rigid_body),
      collider,
    }
  }

  pub fn remove_object(&mut self, handle: PhysicsObjectHandle) {
    if let Some(rigid_body) = handle.rigid_body {
      self.rigid_body_set.remove(
        rigid_body,
        &mut self.island_manager,
        &mut self.collider_set,
        &mut self.impulse_joint_set,
        &mut self.multibody_joint_set,
        false,
      );
    }
    self.collider_set.remove(
      handle.collider,
      &mut self.island_manager,
      &mut self.rigid_body_set,
      false,
    );
  }

  pub fn get_position(&self, handle: &PhysicsObjectHandle) -> Option<Vec2> {
    let rigid_body = self.rigid_body_set.get(handle.rigid_body?)?;
    let position = rigid_body.position().translation.vector;
    Some(Vec2(position.x, position.y))
  }

  pub fn set_position(&mut self, handle: &PhysicsObjectHandle, position: Vec2) {
    let rigid_body = self.rigid_body_set.get_mut(handle.rigid_body.unwrap()).unwrap();
    rigid_body.set_translation(Vector2::new(position.0, position.1), true);
    rigid_body.set_linvel(Vector2::zeros(), true);
  }

  pub fn get_velocity(&self, handle: &PhysicsObjectHandle) -> Option<Vec2> {
    let rigid_body = self.rigid_body_set.get(handle.rigid_body?)?;
    let velocity = rigid_body.linvel();
    Some(Vec2(velocity.x, velocity.y))
  }

  pub fn set_velocity(&mut self, handle: &PhysicsObjectHandle, velocity: Vec2) {
    let rigid_body = self.rigid_body_set.get_mut(handle.rigid_body.unwrap()).unwrap();
    rigid_body.set_linvel(Vector2::new(velocity.0, velocity.1), true);
  }

  pub fn get_shape_and_position(
    &self,
    handle: &PhysicsObjectHandle,
  ) -> Option<(&dyn Shape, &Isometry<Real>)> {
    let rigid_body = self.rigid_body_set.get(handle.rigid_body?)?;
    let collider = self.collider_set.get(handle.collider)?;
    Some((collider.shape(), rigid_body.position()))
  }

  pub fn check_character_controller_movement(
    &self,
    dt: f32,
    handle: &PhysicsObjectHandle,
    shift: Vec2,
    drop_through_platforms: bool,
  ) -> EffectiveCharacterMovement {
    let shape = self.collider_set.get(handle.collider).unwrap().shape();
    let mut hit_groups = WALLS_GROUP;
    if shift.1 > 0.0 && !drop_through_platforms {
      hit_groups |= PLATFORMS_GROUP;
    }
    let corrected_movement = self.char_controller.move_shape(
      dt, // The timestep length (can be set to SimulationSettings::dt).
      &self.rigid_body_set,
      &self.collider_set,
      &self.query_pipeline,
      // We fetch the object's shape.
      shape,
      // We fetch the object's position.
      self.rigid_body_set.get(handle.rigid_body.unwrap()).unwrap().position(),
      //character_shape, // The character’s shape.
      //character_pos,   // The character’s initial position.
      // The character’s movement.
      Vector2::new(shift.0, shift.1),
      QueryFilter::default()
        // Make sure the the character we are trying to move isn’t considered an obstacle.
        .exclude_sensors()
        .groups(InteractionGroups::new(PLAYER_GROUP, hit_groups))
        //.groups(InteractionGroups::new(Group::ALL, Group::GROUP_10))
        .exclude_rigid_body(handle.rigid_body.unwrap()),
      |_| {}, // We don’t care about events in this example.
    );
    corrected_movement
  }

  pub fn move_object_with_character_controller(
    &mut self,
    dt: f32,
    handle: &PhysicsObjectHandle,
    shift: Vec2,
    drop_through_platforms: bool,
  ) -> EffectiveCharacterMovement {
    let corrected_movement = self.check_character_controller_movement(
      dt,
      handle,
      shift,
      drop_through_platforms,
    );
    // Move the object to the new position.
    self.shift_object(
      handle,
      Vec2(
        corrected_movement.translation.x,
        corrected_movement.translation.y,
      ),
    );
    corrected_movement
  }

  pub fn shift_object(&mut self, handle: &PhysicsObjectHandle, shift: Vec2) {
    let rigid_body = self.rigid_body_set.get_mut(handle.rigid_body.unwrap()).unwrap();
    rigid_body.set_translation(
      rigid_body.translation() + Vector2::new(shift.0, shift.1),
      true,
    );
    rigid_body.set_linvel(Vector2::zeros(), true);
  }

  pub fn step(&mut self, dt: f32) {
    self.integration_parameters.dt = dt;
    self.physics_pipeline.step(
      &self.gravity,
      &self.integration_parameters,
      &mut self.island_manager,
      &mut self.broad_phase,
      &mut self.narrow_phase,
      &mut self.rigid_body_set,
      &mut self.collider_set,
      &mut self.impulse_joint_set,
      &mut self.multibody_joint_set,
      &mut self.ccd_solver,
      Some(&mut self.query_pipeline),
      &self.physics_hooks,
      &self.event_handler,
    );
    self.query_pipeline.update(&self.rigid_body_set, &self.collider_set);
  }
}
