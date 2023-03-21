use std::{collections::HashMap, rc::Rc, cell::Cell};

use rapier2d::{
  control::{EffectiveCharacterMovement, KinematicCharacterController},
  na::{Isometry2, Vector2},
  prelude::*,
};
use tiled::Chunk;

use crate::{
  game_maps::GameMap, math::Vec2, tile_rendering::TILE_SIZE, GameObject, GameObjectData,
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
  pub event_handler:          (), //ChannelEventCollector,
  pub char_controller:        KinematicCharacterController,
  pub spawn_point:            Vec2,
  //pub collision_recv:         crossbeam::channel::Receiver<CollisionEvent>,
  //pub contact_force_recv:     crossbeam::channel::Receiver<ContactForceEvent>,
}

impl CollisionWorld {
  pub fn new() -> Self {
    //let (collision_send, collision_recv) = crossbeam::channel::unbounded();
    //let (contact_force_send, contact_force_recv) = crossbeam::channel::unbounded();
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
      //collision_recv,
      //contact_force_recv,
    }
  }

  pub fn load_game_map(
    &mut self,
    game_map: &GameMap,
    objects: &mut HashMap<ColliderHandle, GameObject>,
  ) {
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
                let name: &str = match base_tile.properties.get("name") {
                  Some(tiled::PropertyValue::StringValue(s)) => s,
                  _ => continue,
                };
                let handle = self.new_sensor_circle(
                  PhysicsKind::Sensor,
                  Vec2(tile_pos.0 as f32 + 0.5, tile_pos.1 as f32 + 0.5),
                  0.48,
                );
                let mut orientation = Vec2(1.0, 0.0);
                if tile.flip_h {
                  orientation.0 *= -1.0;
                }
                if tile.flip_v {
                  orientation.1 *= -1.0;
                }
                if tile.flip_d {
                  (orientation.0, orientation.1) = (orientation.1, orientation.0);
                }
                match name {
                  // Coin
                  "coin" => {
                    objects.insert(
                      handle.collider,
                      GameObject {
                        physics_handle: handle,
                        data:           GameObjectData::Coin,
                      },
                    );
                  }
                  // Rare coin
                  "rare_coin" => {
                    objects.insert(
                      handle.collider,
                      GameObject {
                        physics_handle: handle,
                        data:           GameObjectData::RareCoin,
                      },
                    );
                  }
                  "spike" => {
                    objects.insert(
                      handle.collider,
                      GameObject {
                        physics_handle: handle,
                        data:           GameObjectData::Spike,
                      },
                    );
                  }
                  "shooter1" => {
                    objects.insert(
                      handle.collider,
                      GameObject {
                        physics_handle: handle,
                        data:           GameObjectData::Shooter1 {
                          orientation,
                          cooldown: Cell::new(2.0),
                          shoot_period: 2.0,
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

    let collision_layer = game_map.map.layers().find(|l| l.name == "Collision").unwrap();
    match collision_layer.layer_type() {
      tiled::LayerType::ObjectLayer(object_layer) => {
        for object in object_layer.objects() {
          match &object.shape {
            tiled::ObjectShape::Polyline { points } | tiled::ObjectShape::Polygon { points } => {
              //crate::log(&format!("Polygon: {:?} @ ({}, {})", points, object.x, object.y));
              let mut points =
                points.iter().map(|p| (p.0 / TILE_SIZE, p.1 / TILE_SIZE)).collect::<Vec<_>>();
              // If the shape is a polygon, we close it.
              if let tiled::ObjectShape::Polygon { .. } = object.shape {
                points.push(points[0]);
              }
              self.new_static_walls((object.x / TILE_SIZE, object.y / TILE_SIZE), &points[..]);
            }
            _ => panic!("Unsupported object shape: {:?}", object.shape),
          }
          //println!("Object: {:?}", object);
          crate::log(&format!("Object: {:?}", object));
          // let pos = object.properties;
          // let size = object.size();
          // let pos = Vec2(pos.x as f32, pos.y as f32);
          // let size = Vec2(size.width as f32, size.height as f32);
          // let rect = Rect::new(pos, size);
          // self.add_rect(rect, PhysicsKind::Static);
        }
      }
      _ => panic!("Unsupported layer type"),
    }
  }

  pub fn new_static_walls(
    &mut self,
    xy: (f32, f32),
    segments: &[(f32, f32)],
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
      ColliderBuilder::polyline(vertices, Some(indices)),
      rigid_body,
      &mut self.rigid_body_set,
    );
    PhysicsObjectHandle {
      rigid_body: None,
      collider,
    }
  }

  pub fn new_sensor_circle(
    &mut self,
    kind: PhysicsKind,
    position: Vec2,
    radius: f32,
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
      ColliderBuilder::ball(radius).sensor(true),
      rigid_body,
      &mut self.rigid_body_set,
    );
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
      ColliderBuilder::round_cuboid(size.0 / 2.0 - rounding, size.1 / 2.0 - rounding, rounding),
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

  pub fn get_shape_and_position(
    &self,
    handle: &PhysicsObjectHandle,
  ) -> Option<(&dyn Shape, &Isometry<Real>)> {
    let rigid_body = self.rigid_body_set.get(handle.rigid_body?)?;
    let collider = self.collider_set.get(handle.collider)?;
    Some((collider.shape(), rigid_body.position()))
  }

  pub fn move_object_with_character_controller(
    &mut self,
    dt: f32,
    handle: &PhysicsObjectHandle,
    shift: Vec2,
  ) -> EffectiveCharacterMovement {
    let shape = self.collider_set.get(handle.collider).unwrap().shape();
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
        .exclude_rigid_body(handle.rigid_body.unwrap()),
      |_| {}, // We don’t care about events in this example.
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
