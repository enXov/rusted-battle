use rapier2d::prelude::*;
use std::collections::HashMap;

use super::collision::{CollisionEvent as GameCollisionEvent, CollisionEventQueue};

/// Handle to identify rigid bodies
pub type RigidBodyHandle = rapier2d::prelude::RigidBodyHandle;

/// Handle to identify colliders
pub type ColliderHandle = rapier2d::prelude::ColliderHandle;

/// Physics world that manages all physics simulation
pub struct PhysicsWorld {
    /// Gravity vector (default: -9.81 m/s² in y-axis)
    gravity: Vector<Real>,

    /// Integration parameters for the physics simulation
    integration_parameters: IntegrationParameters,

    /// Physics pipeline handles collision detection and solving
    physics_pipeline: PhysicsPipeline,

    /// Island manager for sleeping bodies
    island_manager: IslandManager,

    /// Broad phase collision detection
    broad_phase: DefaultBroadPhase,

    /// Narrow phase collision detection
    narrow_phase: NarrowPhase,

    /// Impulse joint set
    impulse_joint_set: ImpulseJointSet,

    /// Multibody joint set
    multibody_joint_set: MultibodyJointSet,

    /// CCD solver for fast-moving objects
    ccd_solver: CCDSolver,

    /// Query pipeline for raycasts and shape casts
    query_pipeline: QueryPipeline,

    /// Rigid body set
    rigid_body_set: RigidBodySet,

    /// Collider set
    collider_set: ColliderSet,

    /// Collision event handler
    collision_event_queue: CollisionEventQueue,

    /// User data mapping from handles to game entity IDs
    body_to_entity: HashMap<RigidBodyHandle, u64>,
}

impl PhysicsWorld {
    /// Create a new physics world with default settings
    pub fn new() -> Self {
        Self::with_gravity(vector![0.0, -9.81])
    }

    /// Create a new physics world with custom gravity
    pub fn with_gravity(gravity: Vector<Real>) -> Self {
        let mut integration_parameters = IntegrationParameters::default();
        // Fixed timestep of 1/60 seconds (60 FPS)
        integration_parameters.dt = 1.0 / 60.0;

        Self {
            gravity,
            integration_parameters,
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            collision_event_queue: CollisionEventQueue::new(),
            body_to_entity: HashMap::new(),
        }
    }

    /// Step the physics simulation forward by one timestep
    pub fn step(&mut self) {
        // Clear previous frame's collision events
        self.collision_event_queue.clear();

        // Create event handler
        let event_handler = &self.collision_event_queue;

        // Step the physics simulation
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
            &(),
            event_handler,
        );
    }

    /// Add a rigid body to the physics world
    pub fn add_rigid_body(&mut self, body: RigidBody) -> RigidBodyHandle {
        self.rigid_body_set.insert(body)
    }

    /// Add a collider attached to a rigid body
    pub fn add_collider(
        &mut self,
        collider: Collider,
        parent_handle: RigidBodyHandle,
    ) -> ColliderHandle {
        self.collider_set
            .insert_with_parent(collider, parent_handle, &mut self.rigid_body_set)
    }

    /// Remove a rigid body and all its attached colliders
    pub fn remove_rigid_body(&mut self, handle: RigidBodyHandle) {
        self.rigid_body_set.remove(
            handle,
            &mut self.island_manager,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            true, // remove attached colliders
        );
        self.body_to_entity.remove(&handle);
    }

    /// Remove a collider from the physics world
    pub fn remove_collider(&mut self, handle: ColliderHandle) {
        self.collider_set.remove(
            handle,
            &mut self.island_manager,
            &mut self.rigid_body_set,
            true, // wake up attached body
        );
    }

    /// Get a reference to a rigid body
    pub fn get_rigid_body(&self, handle: RigidBodyHandle) -> Option<&RigidBody> {
        self.rigid_body_set.get(handle)
    }

    /// Get a mutable reference to a rigid body
    pub fn get_rigid_body_mut(&mut self, handle: RigidBodyHandle) -> Option<&mut RigidBody> {
        self.rigid_body_set.get_mut(handle)
    }

    /// Get a reference to a collider
    pub fn get_collider(&self, handle: ColliderHandle) -> Option<&Collider> {
        self.collider_set.get(handle)
    }

    /// Get a mutable reference to a collider
    pub fn get_collider_mut(&mut self, handle: ColliderHandle) -> Option<&mut Collider> {
        self.collider_set.get_mut(handle)
    }

    /// Associate a game entity ID with a rigid body
    pub fn set_entity_mapping(&mut self, body_handle: RigidBodyHandle, entity_id: u64) {
        self.body_to_entity.insert(body_handle, entity_id);
    }

    /// Get the entity ID associated with a rigid body
    pub fn get_entity_id(&self, body_handle: RigidBodyHandle) -> Option<u64> {
        self.body_to_entity.get(&body_handle).copied()
    }

    /// Cast a ray and return the first hit
    pub fn raycast(
        &self,
        ray_origin: Vector<Real>,
        ray_dir: Vector<Real>,
        max_toi: Real,
        solid: bool,
        filter: QueryFilter,
    ) -> Option<(ColliderHandle, Real)> {
        let ray = Ray::new(point![ray_origin.x, ray_origin.y], ray_dir);
        self.query_pipeline.cast_ray(
            &self.rigid_body_set,
            &self.collider_set,
            &ray,
            max_toi,
            solid,
            filter,
        )
    }

    // Note: Shape casting will be added in a future version
    // The API changed in rapier 0.19 and needs further investigation

    /// Get all collision events from this frame
    pub fn get_collision_events(&self) -> Vec<GameCollisionEvent> {
        self.collision_event_queue.events()
    }

    /// Set gravity for the physics world
    pub fn set_gravity(&mut self, gravity: Vector<Real>) {
        self.gravity = gravity;
    }

    /// Get current gravity
    pub fn gravity(&self) -> Vector<Real> {
        self.gravity
    }

    /// Set the timestep for physics simulation
    pub fn set_timestep(&mut self, dt: Real) {
        self.integration_parameters.dt = dt;
    }

    /// Get the current timestep
    pub fn timestep(&self) -> Real {
        self.integration_parameters.dt
    }

    /// Get references to internal components for debug rendering
    pub fn debug_data(&self) -> DebugData<'_> {
        DebugData {
            rigid_bodies: &self.rigid_body_set,
            colliders: &self.collider_set,
        }
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}

/// Data structure for debug rendering
pub struct DebugData<'a> {
    pub rigid_bodies: &'a RigidBodySet,
    pub colliders: &'a ColliderSet,
}
