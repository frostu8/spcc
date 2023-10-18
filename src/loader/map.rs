//! Types for saving and loading maps.
//!
//! The map is uniquely defined as all of the constants of a stage: the static
//! models, the environment and lighting, the enemies that spawn, etc. Although
//! these technically can be modified by contracts and other nonsense, these
//! aren't typically things that can be directly varied by the player, unlike
//! selected contracts, operators and event-specific tools.

use serde::Deserialize;
use serde::de::{self, Deserializer, Visitor};

use std::fmt::{self, Formatter};

use bevy::reflect::{TypeUuid, TypePath};
use bevy::prelude::*;

use iyes_progress::prelude::*;

use crate::tile_map::{self, GridBundle, TileBundle, TileKind};

use super::StageAssets;

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
    pub pos: IVec2,
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

/// The entity that contains all static models and the grid of a map.
///
/// It is only valid if there is one or zero `MapInstance`.
#[derive(Clone, Component, Debug, Default)]
pub struct MapInstance;

/// Loads the map.
pub fn load_map(
    mut commands: Commands,
    stage_assets: Res<StageAssets>,
    maps: Res<Assets<Map>>,
    asset_server: Res<AssetServer>,
    mut loading: ResMut<AssetsLoading>,
    map_instance_query: Query<Entity, With<MapInstance>>,
) {
    // stop loading if it has already been loaded
    if let Ok(_) = map_instance_query.get_single() {
        return;
    }

    // check if the map is loaded
    let map = match maps.get(&stage_assets.map) {
        Some(map) => map,
        None => return,
    };

    // begin loading static models with root entity.
    let map_entity = commands
        .spawn((
            SpatialBundle::default(),
            MapInstance,
        ))
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
        .set_parent(map_entity)
        .with_children(|parent| {
            // loop through tiles
            for tile in map.tile_map.tiles.iter() {
                parent
                    .spawn(TileBundle {
                        coordinates: tile.pos.clone().into(),
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
}

