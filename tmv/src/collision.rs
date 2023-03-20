use rapier2d::{prelude::*, na::Vector2};

use crate::math::Vec2;

pub enum PhysicsKind {
  Static,
  Dynamic,
  Kinematic,
  Sensor,
}

#[derive(Debug, Clone)]
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
  pub island_manager:         IslandManager,
  pub broad_phase:            BroadPhase,
  pub narrow_phase:           NarrowPhase,
  pub impulse_joint_set:      ImpulseJointSet,
  pub multibody_joint_set:    MultibodyJointSet,
  pub ccd_solver:             CCDSolver,
  pub physics_hooks:          (),
  pub event_handler:          (),
}

impl CollisionWorld {
  pub fn new() -> Self {
    Self {
      rigid_body_set:         RigidBodySet::new(),
      collider_set:           ColliderSet::new(),
      gravity:                vector![0.0, 0.0],
      integration_parameters: IntegrationParameters::default(),
      physics_pipeline:       PhysicsPipeline::new(),
      island_manager:         IslandManager::new(),
      broad_phase:            BroadPhase::new(),
      narrow_phase:           NarrowPhase::new(),
      impulse_joint_set:      ImpulseJointSet::new(),
      multibody_joint_set:    MultibodyJointSet::new(),
      ccd_solver:             CCDSolver::new(),
      physics_hooks:          (),
      event_handler:          (),
    }
  }

  pub fn new_static_walls(&mut self, segments: &[Vec<Vec2>]) -> PhysicsObjectHandle {
    let rigid_body = self.rigid_body_set.insert(RigidBodyBuilder::fixed().build());
    let vertices: Vec<_> =
      segments.iter().flat_map(|segment| segment.iter().map(|v| Point::new(v.0, v.1))).collect();
    let mut indices: Vec<[u32; 2]> = Vec::new();
    let mut idx = 0;
    for segment in segments {
      for _ in 0..segment.len() - 1 {
        indices.push([idx, idx + 1]);
        idx += 1;
      }
      idx += 1;
    }
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
      ColliderBuilder::round_cuboid(size.0 - rounding, size.1 - rounding, rounding),
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
      None,
      &self.physics_hooks,
      &self.event_handler,
    );
  }
}
