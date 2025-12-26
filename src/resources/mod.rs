pub mod combat;
pub mod faction;
pub mod map_data;
pub mod fog_of_war;
pub mod wind;
pub mod ui_assets;
pub mod world_clock;

pub use combat::*;
pub use faction::*;
pub use map_data::*;
pub use fog_of_war::*;
pub use wind::*;
pub use world_clock::*;

pub mod route_cache;
pub use route_cache::*;

pub mod fleet;
pub use fleet::*;

pub mod meta_profile;
pub use meta_profile::*;

pub mod landmass;
pub use landmass::*;

// Legacy navmesh module - deprecated, use landmass instead
pub mod navmesh;
pub use navmesh::NavMeshResource;

pub mod stippling_material;
pub use stippling_material::*;

pub mod cli;
pub use cli::*;
