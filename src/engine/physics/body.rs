use super::collision::CollisionGroups;
use rapier2d::prelude::*;

pub use rapier2d::prelude::{ColliderHandle, RigidBodyHandle};

/// Builder for creating rigid bodies with common configurations
pub struct BodyBuilder {
    body_type: RigidBodyType,
    position: Isometry<Real>,
    linvel: Vector<Real>,
    angvel: Real,
    gravity_scale: Real,
    can_sleep: bool,
    locked_axes: LockedAxes,
}

impl BodyBuilder {
    /// Create a new dynamic body (affected by forces and collisions)
    pub fn new_dynamic() -> Self {
        Self {
            body_type: RigidBodyType::Dynamic,
            position: Isometry::identity(),
            linvel: Vector::zeros(),
            angvel: 0.0,
            gravity_scale: 1.0,
            can_sleep: true,
            locked_axes: LockedAxes::empty(),
        }
    }

    /// Create a new kinematic position-based body (not affected by forces)
    pub fn new_kinematic_position_based() -> Self {
        Self {
            body_type: RigidBodyType::KinematicPositionBased,
            position: Isometry::identity(),
            linvel: Vector::zeros(),
            angvel: 0.0,
            gravity_scale: 0.0,
            can_sleep: false,
            locked_axes: LockedAxes::empty(),
        }
    }

    /// Create a new kinematic velocity-based body
    pub fn new_kinematic_velocity_based() -> Self {
        Self {
            body_type: RigidBodyType::KinematicVelocityBased,
            position: Isometry::identity(),
            linvel: Vector::zeros(),
            angvel: 0.0,
            gravity_scale: 0.0,
            can_sleep: false,
            locked_axes: LockedAxes::empty(),
        }
    }

    /// Create a new fixed (static) body (completely immovable)
    pub fn new_fixed() -> Self {
        Self {
            body_type: RigidBodyType::Fixed,
            position: Isometry::identity(),
            linvel: Vector::zeros(),
            angvel: 0.0,
            gravity_scale: 0.0,
            can_sleep: false,
            locked_axes: LockedAxes::empty(),
        }
    }

    /// Set the initial position of the body
    pub fn position(mut self, x: Real, y: Real) -> Self {
        self.position = Isometry::translation(x, y);
        self
    }

    /// Set the initial position and rotation
    pub fn position_rotation(mut self, x: Real, y: Real, angle: Real) -> Self {
        self.position = Isometry::new(vector![x, y], angle);
        self
    }

    /// Set the initial linear velocity
    pub fn linvel(mut self, x: Real, y: Real) -> Self {
        self.linvel = vector![x, y];
        self
    }

    /// Set the initial angular velocity (radians per second)
    pub fn angvel(mut self, angvel: Real) -> Self {
        self.angvel = angvel;
        self
    }

    /// Set the gravity scale (1.0 = normal gravity, 0.0 = no gravity)
    pub fn gravity_scale(mut self, scale: Real) -> Self {
        self.gravity_scale = scale;
        self
    }

    /// Set whether the body can sleep when inactive
    pub fn can_sleep(mut self, can_sleep: bool) -> Self {
        self.can_sleep = can_sleep;
        self
    }

    /// Lock rotation (useful for player characters)
    pub fn lock_rotation(mut self) -> Self {
        self.locked_axes = LockedAxes::ROTATION_LOCKED;
        self
    }

    /// Lock translation in X axis
    pub fn lock_translation_x(mut self) -> Self {
        self.locked_axes |= LockedAxes::TRANSLATION_LOCKED_X;
        self
    }

    /// Lock translation in Y axis
    pub fn lock_translation_y(mut self) -> Self {
        self.locked_axes |= LockedAxes::TRANSLATION_LOCKED_Y;
        self
    }

    /// Build the rigid body
    pub fn build(self) -> RigidBody {
        let mut body = RigidBodyBuilder::new(self.body_type)
            .position(self.position)
            .linvel(self.linvel)
            .angvel(self.angvel)
            .gravity_scale(self.gravity_scale)
            .can_sleep(self.can_sleep)
            .locked_axes(self.locked_axes)
            .build();

        // Set additional properties
        if self.body_type == RigidBodyType::Dynamic {
            // Set reasonable defaults for dynamic bodies
            body.set_linear_damping(0.5); // Some air resistance
            body.set_angular_damping(1.0); // More rotational damping
        }

        body
    }
}

/// Builder for creating colliders with common configurations
pub struct ColliderBuilder2D {
    shape: SharedShape,
    collision_groups: CollisionGroups,
    is_sensor: bool,
    friction: Real,
    restitution: Real,
    density: Option<Real>,
    mass: Option<Real>,
    active_events: ActiveEvents,
}

impl ColliderBuilder2D {
    /// Create a box-shaped collider
    pub fn box_shape(half_width: Real, half_height: Real) -> Self {
        Self {
            shape: SharedShape::cuboid(half_width, half_height),
            collision_groups: CollisionGroups::Default,
            is_sensor: false,
            friction: 0.5,
            restitution: 0.0,
            density: Some(1.0),
            mass: None,
            active_events: ActiveEvents::COLLISION_EVENTS,
        }
    }

    /// Create a circle-shaped collider
    pub fn circle(radius: Real) -> Self {
        Self {
            shape: SharedShape::ball(radius),
            collision_groups: CollisionGroups::Default,
            is_sensor: false,
            friction: 0.5,
            restitution: 0.0,
            density: Some(1.0),
            mass: None,
            active_events: ActiveEvents::COLLISION_EVENTS,
        }
    }

    /// Create a capsule-shaped collider (good for characters)
    pub fn capsule(half_height: Real, radius: Real) -> Self {
        let a = point![0.0, -half_height];
        let b = point![0.0, half_height];
        Self {
            shape: SharedShape::capsule(a, b, radius),
            collision_groups: CollisionGroups::Default,
            is_sensor: false,
            friction: 0.5,
            restitution: 0.0,
            density: Some(1.0),
            mass: None,
            active_events: ActiveEvents::COLLISION_EVENTS,
        }
    }

    /// Create a collider from a convex polygon
    pub fn convex_hull(points: &[[Real; 2]]) -> Option<Self> {
        let points: Vec<_> = points.iter().map(|p| point![p[0], p[1]]).collect();

        SharedShape::convex_hull(&points).map(|shape| Self {
            shape,
            collision_groups: CollisionGroups::Default,
            is_sensor: false,
            friction: 0.5,
            restitution: 0.0,
            density: Some(1.0),
            mass: None,
            active_events: ActiveEvents::COLLISION_EVENTS,
        })
    }

    /// Set the collision groups for filtering
    pub fn collision_groups(mut self, groups: CollisionGroups) -> Self {
        self.collision_groups = groups;
        self
    }

    /// Make this a sensor (detects collisions but doesn't cause physical response)
    pub fn sensor(mut self, is_sensor: bool) -> Self {
        self.is_sensor = is_sensor;
        self
    }

    /// Set friction coefficient (0.0 = no friction, 1.0 = high friction)
    pub fn friction(mut self, friction: Real) -> Self {
        self.friction = friction;
        self
    }

    /// Set restitution/bounciness (0.0 = no bounce, 1.0 = perfect bounce)
    pub fn restitution(mut self, restitution: Real) -> Self {
        self.restitution = restitution;
        self
    }

    /// Set density (mass will be calculated from shape volume)
    pub fn density(mut self, density: Real) -> Self {
        self.density = Some(density);
        self.mass = None;
        self
    }

    /// Set mass directly (overrides density)
    pub fn mass(mut self, mass: Real) -> Self {
        self.mass = Some(mass);
        self.density = None;
        self
    }

    /// Enable contact force events
    pub fn contact_force_events(mut self) -> Self {
        self.active_events |= ActiveEvents::CONTACT_FORCE_EVENTS;
        self
    }

    /// Build the collider
    pub fn build(self) -> Collider {
        let mut builder = rapier2d::prelude::ColliderBuilder::new(self.shape)
            .collision_groups(self.collision_groups.to_interaction_groups())
            .sensor(self.is_sensor)
            .friction(self.friction)
            .restitution(self.restitution)
            .active_events(self.active_events);

        // Set mass or density
        if let Some(mass) = self.mass {
            builder = builder.mass(mass);
        } else if let Some(density) = self.density {
            builder = builder.density(density);
        }

        builder.build()
    }
}

/// Common rigid body configurations for game objects
pub mod presets {
    use super::*;

    /// Create a player character body (dynamic, rotation locked)
    pub fn player_body(x: Real, y: Real) -> RigidBody {
        BodyBuilder::new_dynamic()
            .position(x, y)
            .lock_rotation()
            .gravity_scale(1.0)
            .can_sleep(false) // Players should never sleep
            .build()
    }

    /// Create a player character collider (capsule shape)
    pub fn player_collider(width: Real, height: Real) -> Collider {
        let radius = width / 2.0;
        let half_height = (height / 2.0) - radius; // Subtract radius to get capsule half-height

        ColliderBuilder2D::capsule(half_height, radius)
            .collision_groups(CollisionGroups::Player)
            .friction(0.0) // No friction for smooth movement
            .restitution(0.0) // No bounce
            .density(1.0)
            .build()
    }

    /// Create a platform body (fixed/static)
    pub fn platform_body(x: Real, y: Real) -> RigidBody {
        BodyBuilder::new_fixed().position(x, y).build()
    }

    /// Create a platform collider (box shape)
    pub fn platform_collider(width: Real, height: Real) -> Collider {
        ColliderBuilder2D::box_shape(width / 2.0, height / 2.0)
            .collision_groups(CollisionGroups::Platform)
            .friction(0.3)
            .restitution(0.0)
            .build()
    }

    /// Create a projectile body (dynamic)
    pub fn projectile_body(x: Real, y: Real, vel_x: Real, vel_y: Real) -> RigidBody {
        BodyBuilder::new_dynamic()
            .position(x, y)
            .linvel(vel_x, vel_y)
            .gravity_scale(0.0) // Projectiles often ignore gravity
            .can_sleep(false)
            .build()
    }

    /// Create a projectile collider (circle shape)
    pub fn projectile_collider(radius: Real) -> Collider {
        ColliderBuilder2D::circle(radius)
            .collision_groups(CollisionGroups::Projectile)
            .friction(0.0)
            .restitution(0.8) // Slightly bouncy
            .density(0.1) // Light
            .build()
    }

    /// Create a sensor collider (detects but doesn't block)
    pub fn sensor_collider(width: Real, height: Real) -> Collider {
        ColliderBuilder2D::box_shape(width / 2.0, height / 2.0)
            .collision_groups(CollisionGroups::Sensor)
            .sensor(true)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_body_builder_dynamic() {
        let body = BodyBuilder::new_dynamic()
            .position(10.0, 20.0)
            .linvel(5.0, 0.0)
            .build();

        assert_eq!(body.body_type(), RigidBodyType::Dynamic);
        assert_eq!(body.translation().x, 10.0);
        assert_eq!(body.translation().y, 20.0);
    }

    #[test]
    fn test_collider_builder_box() {
        let collider = ColliderBuilder2D::box_shape(1.0, 2.0).friction(0.3).build();

        assert!(!collider.is_sensor());
        assert_eq!(collider.friction(), 0.3);
    }

    #[test]
    fn test_player_preset() {
        let body = presets::player_body(0.0, 0.0);
        let collider = presets::player_collider(1.0, 2.0);

        assert_eq!(body.body_type(), RigidBodyType::Dynamic);
        assert!(body.is_rotation_locked());
        assert!(!collider.is_sensor());
    }
}
