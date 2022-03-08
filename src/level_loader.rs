use crate::{
    map::{Level, LevelList, LevelSerde},
    CurrentLevel, GameState,
};
use bevy::prelude::*;
use bevy_asset::{AssetLoader, AssetServer};

#[derive(Default)]
pub struct LevelListLoader;

impl AssetLoader for LevelListLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy_asset::LoadContext,
    ) -> bevy_asset::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let level_list: Vec<LevelSerde> = serde_json::de::from_slice(&bytes).unwrap();
            let beaten = vec![false; level_list.len()];
            let mut levels = vec![];
            for level in level_list.into_iter() {
                levels.push(Level::from_level_serde(level).await);
            }
            load_context.set_default_asset(bevy_asset::LoadedAsset::new(crate::map::LevelList {
                levels,
                beaten,
            }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

pub fn start_load_level_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load::<LevelList, _>("./levels.json");
    commands.insert_resource(handle);
}

pub fn setup_level_resources(
    mut commands: Commands,
    mut state: ResMut<State<GameState>>,
    handle: Res<Handle<LevelList>>,
    assets: Res<Assets<LevelList>>,
) {
    if let Some(levels) = assets.get(&*handle) {
        state.set(GameState::StartScreen).unwrap();
        commands.insert_resource(levels.clone());
        commands.insert_resource(levels.levels[0].clone());
        commands.insert_resource(CurrentLevel(0));
    }
}
