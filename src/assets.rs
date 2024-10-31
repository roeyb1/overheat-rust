use bevy::{asset::Handle, prelude::{Image, Resource}, sprite::TextureAtlasLayout};
use bevy_asset_loader::asset_collection::AssetCollection;


#[derive(AssetCollection, Resource)]
pub struct PlayerAssets {
    #[asset(texture_atlas(
        tile_size_x = 48, tile_size_y = 48,
        columns = 23, rows = 4,
        padding_x = 0, padding_y = 0,
        offset_x = 0, offset_y = 0))]
    pub player_atlas: Handle<TextureAtlasLayout>,

    #[asset(path = "players/player.png")]
    pub player_tileset: Handle<Image>,
}