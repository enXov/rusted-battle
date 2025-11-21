// Physics system using rapier2d

pub mod body;
mod collision;
mod debug;
mod world;

pub use body::RigidBodyHandle;
pub use collision::CollisionEvent;
pub use debug::DebugRenderer;
pub use world::PhysicsWorld;

// Re-export commonly used rapier types for convenience
#[allow(unused_imports)]
pub use rapier2d::prelude::{
    nalgebra, ActiveEvents, ColliderBuilder, Isometry, Real, RigidBodyType, Vector,
};

// Re-export for internal use and future expansion
#[allow(unused_imports)]
pub use body::{BodyBuilder, ColliderHandle};
#[allow(unused_imports)]
pub use collision::CollisionGroups;
