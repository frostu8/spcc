//! The tile map that determines grid-locked interactions, such as operators.

use bevy::prelude::*;
use bevy::transform::TransformSystem;

use bevy_mod_picking::prelude::*;

use serde::{Deserialize, Serialize};

use crate::material::TileHighlightMaterial;

//use iyes_progress::prelude::*;

//use crate::AppState;

/// The plugin for grid operations.
pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<GridAssets>()
            .add_event::<TileFocusEvent>()
            .add_systems(Update, color_selected_tiles) // TODO: debug
            .add_systems(PostUpdate, setup_new_tiles.before(TransformSystem::TransformPropagate))
            .add_systems(Startup, load_grid_assets);
            //.add_systems(OnEnter(AppState::StageLoading), load_grid_assets);
    }
}

/// Grid assets.
#[derive(Default, Resource)]
pub struct GridAssets {
    /// A single square mesh. Two triangles whose normals face upward, and with
    /// standard UV.
    pub square_mesh: Handle<Mesh>,
    /// The grid indicator texture.
    pub grid_indicator_texture: Handle<Image>,
    /// Material for hostile (or damage) tiles.
    pub hostile_indicator: Handle<TileHighlightMaterial>,
    /// Material for support (or healing) tiles.
    pub support_indicator: Handle<TileHighlightMaterial>,
}

/// Grid bundle.
#[derive(Bundle, Default)]
pub struct GridBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
    pub grid: Grid,
}

/// The grid component.
#[derive(Clone, Component, Debug, Default)]
pub struct Grid;

/// The coordinates to a tile entity.
#[derive(Clone, Component, Debug, Default, Deserialize, Reflect, Serialize)]
pub struct Coordinates {
    pub x: u32,
    pub y: u32,
}

/// The tile.
/// 
/// Actually contains information about the tile. Along with this, also
/// contains mesh information to render informative data.
#[derive(Clone, Component, Debug, Default, Reflect)]
pub struct Tile {
    kind: TileKind,
    deployable: bool,
}

impl Tile {
    /// Creates a new tile.
    pub fn new(kind: TileKind, deployable: bool) -> Tile {
        Tile { kind, deployable }
    }

    /// The kind of tile.
    pub fn kind(&self) -> TileKind {
        self.kind
    }

    /// Whether the tile is deployable or not.
    pub fn deployable(&self) -> bool {
        self.deployable
    }
}

/// The kind of tile.
///
/// Determines what kind of operators can be deployed, and whether enemies can
/// cross.
#[derive(Clone, Copy, Component, Debug, Default, Deserialize, Reflect, Serialize)]
pub enum TileKind {
    Ground,
    #[default]
    HighGround,
}

/// A tile bundle for setting up a [`Tile`].
///
/// Anything besides [`TileBundle::coordinates`] and [`TileBundle::tile`].
#[derive(Bundle, Clone, Default)]
pub struct TileBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
    pub mesh: Handle<Mesh>,
    pub material: Handle<TileHighlightMaterial>,
    pub coordinates: Coordinates,
    pub tile: Tile,
    pub raycast_pick_target: RaycastPickTarget,
}

/// An event for when the player taps/clicks on a tile.
#[derive(Debug, Event)]
pub struct TileFocusEvent(pub Entity);

impl From<ListenerInput<Pointer<Click>>> for TileFocusEvent {
    fn from(e: ListenerInput<Pointer<Click>>) -> TileFocusEvent {
        TileFocusEvent(e.target())
    }
}

pub fn color_selected_tiles(
    mut query: Query<&mut Handle<TileHighlightMaterial>>,
    grid_assets: Res<GridAssets>,
    mut events: EventReader<TileFocusEvent>,
) {
    for event in events.iter() {
        if let Ok(mut material) = query.get_mut(event.0) {
            *material = grid_assets.support_indicator.clone();
        }
    }
}

pub fn setup_new_tiles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Handle<Mesh>, &mut Handle<TileHighlightMaterial>, &Tile, &Coordinates), Added<Tile>>,
    grid_assets: Res<GridAssets>,
) {
    for (id, mut transform, mut mesh, mut material, tile, coordinates) in query.iter_mut() {
        *mesh = grid_assets.square_mesh.clone();

        let height = match tile.kind {
            TileKind::Ground => 0.0,
            TileKind::HighGround => 0.5,
        };

        // default material
        *material = grid_assets.hostile_indicator.clone();

        *transform = Transform::from_xyz(-(coordinates.x as f32), height, coordinates.y as f32);
        
        // add event callbacks
        commands
            .entity(id)
            .insert(On::<Pointer<Click>>::send_event::<TileFocusEvent>());
    }
}

pub fn load_grid_assets(
    mut grid_assets: ResMut<GridAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut tile_materials: ResMut<Assets<TileHighlightMaterial>>,
    //mut loading: ResMut<AssetsLoading>,
    asset_server: Res<AssetServer>,
) {
    // create square mesh
    grid_assets.square_mesh = meshes.add(Mesh::from(shape::Plane::from_size(1.0)));

    // load grid indicator
    grid_assets.grid_indicator_texture = asset_server.load("system/grid_indicator.png");

    // create materials
    grid_assets.hostile_indicator = tile_materials.add(TileHighlightMaterial {
        color: Color::rgba(1.0, 0.576, 0.180, 0.9), // #ff932e
        color_texture: Some(grid_assets.grid_indicator_texture.clone()),
        animate_speed: 0.25,
    });

    grid_assets.support_indicator = tile_materials.add(TileHighlightMaterial {
        color: Color::rgba(0.184, 0.467, 0.922, 0.9), // #2f77eb
        color_texture: Some(grid_assets.grid_indicator_texture.clone()),
        animate_speed: 0.25,
    });
}

