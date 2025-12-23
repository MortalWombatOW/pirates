use bevy::prelude::*;

#[derive(Resource)]
pub struct UiAssets {
    pub parchment_texture: Handle<Image>,
}

impl FromWorld for UiAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        UiAssets {
            parchment_texture: asset_server.load("sprites/ui/parchment.png"),
        }
    }
}
