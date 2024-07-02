use bevy_asset_loader::prelude::*;

use crate::prelude::*;

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(crate::MainState::Loading)
                .continue_to_state(crate::MainState::Playing)
                .load_collection::<Player>()
                .load_collection::<Bullet>()
                .load_collection::<Misc>(),
        );
    }
}

#[derive(Resource, AssetCollection)]
pub struct Misc {
    #[asset(path = "background.png")]
    pub background: Handle<Image>,
}

#[derive(Resource, AssetCollection)]
pub struct Player {
    #[asset(texture_atlas_layout(tile_size_x = 32., tile_size_y = 64., columns = 8, rows = 2,))]
    pub layout: Handle<TextureAtlasLayout>,
    #[asset(path = "player.png")]
    pub sprite: Handle<Image>,
    #[asset(path = "cursor.png")]
    pub cursor: Handle<Image>,
}

#[derive(Resource, AssetCollection)]
pub struct Bullet {
    #[asset(texture_atlas_layout(tile_size_x = 32., tile_size_y = 32., columns = 4, rows = 1,))]
    pub layout: Handle<TextureAtlasLayout>,
    #[asset(path = "bullet.png")]
    pub sprite: Handle<Image>,
}
