//! Structs that help load maps from files.
//!
//! Until a bevy editor exists (which it won't by the time I finish this
//! project), here is a ridiculously crude scene loading system. Most of this
//! code is "hack quality" at best.

use serde::Deserialize;
use serde::de::{self, Deserializer, Visitor};

use std::fmt::{self, Formatter};

use bevy::reflect::{TypeUuid, TypePath};
use bevy::prelude::*;

use iyes_progress::prelude::*;

use bevy_common_assets::ron::RonAssetPlugin;

use crate::AppState;
use crate::tile_map::{self, GridBundle, TileBundle, Coordinates, TileKind};

/// A map.
///
/// Can be loaded from RON.
#[derive(Debug, Clone, Deserialize, TypeUuid, TypePath)]
#[uuid = "909141d1-0a85-4833-8b94-7164332c2bf4"]
pub struct Map {
    /// The name of the map.
    ///
    /// This will display in the loading screen.
    pub name: String,
    /// Environment settings.
    pub environment: Environment,
    /// Tile settings.
    pub tile_map: TileMap,
    /// Static models.
    pub models: Vec<Model>,
}

/// Environmental display settings for maps.
#[derive(Debug, Clone, Deserialize)]
pub struct Environment {
    /// The color of the directional light.
    #[serde(deserialize_with = "from_hex")]
    pub color: Color,
    /// The luminance (lux) of the directional light.
    ///
    /// See [`DirectionalLight`][1] for more information on this.
    ///
    /// [1]: https://docs.rs/bevy/latest/bevy/prelude/struct.DirectionalLight.html
    pub luminance: f32,
}

/// Tile map settings.
#[derive(Debug, Clone, Deserialize)]
pub struct TileMap {
    /// The offset of the tilemap.
    pub offset: Vec3,
    /// The tiles that make up the tile map.
    pub tiles: Vec<Tile>,
}

/// Definition for a single tile.
#[derive(Debug, Clone, Deserialize)]
pub struct Tile {
    /// Position of a tile.
    pub pos: Coordinates,
    /// The type of tile.
    #[serde(default)]
    pub kind: TileKind,
    /// Whether the tile is deployable or not.
    #[serde(default)]
    pub deployable: bool,
}

/// A static model for a map.
#[derive(Debug, Clone, Deserialize)]
pub struct Model {
    pub path: String,
    #[serde(default)]
    pub position: (f32, f32, f32),
}

fn from_hex<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: Deserializer<'de>,
{
    struct HexVisitor;

    impl<'a> Visitor<'a> for HexVisitor {
        type Value = Color;

        fn expecting(&self, f: &mut Formatter) -> fmt::Result {
            f.write_str("valid hex color, with or without '#'")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error
        {
            Color::hex(v)
                .map_err(|e| de::Error::custom(e.to_string()))
        }
    }

    deserializer.deserialize_string(HexVisitor)
}

/// Stage plugin.
pub struct StagePlugin;

impl Plugin for StagePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(StageLoader::default())
            .add_plugins(ProgressPlugin::new(AppState::StageLoading)
                .continue_to(AppState::InGame)
                .track_assets())
            .add_systems(OnEnter(AppState::StageLoading), load_stage)
            .add_systems(Update, load_map.run_if(in_state(AppState::StageLoading)))
            .add_plugins(RonAssetPlugin::<Map>::new(&["ron"]));
    }
}

/// A stage builder.
#[derive(Default)]
pub struct StageBuilder {
    /// The map as a path.
    map: String,
}

impl StageBuilder {
    /// Creates a new stage builder, to the specified map.
    pub fn new(map_path: impl Into<String>) -> StageBuilder {
        StageBuilder {
            map: map_path.into(),
        }
    }
}

/// The stage loader resource.
///
/// Loads the map, which includes static models, enemies and environmental
/// conditions, and also loads operator resources. Also loads contingencies.
#[derive(Default, Resource)]
pub struct StageLoader {
    builder: StageBuilder,
    map: Handle<Map>,
    map_entity: Option<Entity>,
}

impl StageLoader {
    /// Queues setup for a stage on the stage loader.
    ///
    /// To trigger the loading, the [`AppState`] **must** be set to
    /// [`AppState::StageLoading`].
    pub fn load(&mut self, builder: StageBuilder) {
        self.builder = builder;
    }

    /// Gets a handle to the map definition.
    pub fn map(&self) -> Handle<Map> {
        self.map.clone()
    }
}

/// Loads the map.
fn load_map(
    mut commands: Commands,
    mut stage_loader: ResMut<StageLoader>,
    maps: Res<Assets<Map>>,
    asset_server: Res<AssetServer>,
    mut loading: ResMut<AssetsLoading>,
) {
    // stop loading if it has already been loaded
    if stage_loader.map_entity.is_some() {
        return;
    }

    // check if the map is loaded
    let map = match maps.get(&stage_loader.map) {
        Some(map) => map,
        None => return,
    };

    // begin loading static models with root entity.
    let map_entity = commands
        .spawn(SpatialBundle::default())
        .id();

    // load directional light
    commands
        .spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: map.environment.color,
                illuminance: map.environment.luminance,
                ..Default::default()
            },
            transform: Transform::default().looking_to(-Vec3::Y, Vec3::Y),
            ..Default::default()
        })
        .set_parent(map_entity);

    // spawn tile map
    commands
        .spawn(GridBundle {
            transform: Transform::from_translation(map.tile_map.offset),
            ..default()
        })
        .with_children(|parent| {
            // loop through tiles
            for tile in map.tile_map.tiles.iter() {
                parent
                    .spawn(TileBundle {
                        coordinates: tile.pos.clone(),
                        tile: tile_map::Tile::new(tile.kind, tile.deployable),
                        ..default()
                    });
            }
        });

    // load models
    for model in map.models.iter() {
        let gltf = asset_server.load(&model.path);

        loading.add(&gltf);

        commands
            .spawn(SceneBundle {
                scene: gltf,
                transform: Transform::from_xyz(model.position.0, model.position.1, model.position.2),
                ..Default::default()
            })
            .set_parent(map_entity);
    }

    stage_loader.map_entity = Some(map_entity);
}

/// Begins loading the stage. Map-specific loading will occur when the Map file
/// is loaded.
fn load_stage(
    //mut commands: Commands,
    mut stage_loader: ResMut<StageLoader>,
    asset_server: Res<AssetServer>,
    mut loading: ResMut<AssetsLoading>,
) {
    // start loading the map
    let StageLoader {
        builder,
        map,
        ..
        //map_entity,
    } = &mut *(stage_loader);

    *map = asset_server.load(&builder.map);
    loading.add(&*map);

    // load operators, contingencies:
    // lol
}

