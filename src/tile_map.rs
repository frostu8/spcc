//! The tile map that determines grid-locked interactions, such as operators.

use bevy::prelude::*;
use bevy::render::{mesh::Indices, render_resource::PrimitiveTopology};
use bevy::transform::TransformSystem;

//use iyes_progress::prelude::*;

//use crate::AppState;

/// The plugin for grid operations.
pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<GridAssets>()
            .add_systems(Update, scroll_square_mesh)
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
    pub hostile_indicator: Handle<StandardMaterial>,
    /// Material for support (or healing) tiles.
    pub support_indicator: Handle<StandardMaterial>,
}

/// The grid component.
pub struct Grid;

/// The coordinates to a tile.
#[derive(Clone, Component, Debug, Default)]
pub struct Coordinates {
    pub x: u32,
    pub y: u32,
}

/// The tile.
/// 
/// Actually contains information about the tile. Along with this, also
/// contains mesh information to render informative data.
#[derive(Clone, Component, Debug, Default)]
pub struct Tile {
    kind: TileKind,
    deployable: bool,
}

impl Tile {
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
#[derive(Clone, Copy, Component, Debug, Default)]
pub enum TileKind {
    #[default]
    Ground,
    HighGround,
}

/// A tile bundle for setting up a [`Tile`].
///
/// Anything besides [`TileBundle::coordinates`] and [`TileBundle::tile`].
#[derive(Clone, Default, Debug)]
pub struct TileBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub coordinates: Coordinates,
    pub tile: Tile,
}

// TODO: Why is this CPU bound but I don't care anymore, it works(TM)
pub fn scroll_square_mesh(
    grid_assets: ResMut<GridAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    time: Res<Time>,
) {
    if let Some(square_mesh) = meshes.get_mut(&grid_assets.square_mesh) {
        let zero = -time.delta_seconds();
        let one = -time.delta_seconds() + 1.0;

        square_mesh.insert_attribute(
            Mesh::ATTRIBUTE_UV_0,
            vec![[zero, zero], [zero, one], [one, one], [one, zero]]
        );
    }
}

pub fn setup_new_tiles(
    mut query: Query<(&mut Transform, &mut Handle<Mesh>, &Tile, &Coordinates), Added<Tile>>,
    grid_assets: Res<GridAssets>,
) {
    for (mut transform, mut mesh, tile, coordinates) in query.iter_mut() {
        *mesh = grid_assets.square_mesh.clone();

        let height = match tile.kind {
            TileKind::Ground => 0.0,
            TileKind::HighGround => 0.5,
        };

        *transform = Transform::from_xyz(coordinates.x as f32 + 0.5, height, coordinates.y as f32 + 0.5);
    }
}

pub fn load_grid_assets(
    mut grid_assets: ResMut<GridAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    //mut loading: ResMut<AssetsLoading>,
    asset_server: Res<AssetServer>,
) {
    // create square mesh
    let mut square_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    square_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[-0.5, 0.0, -0.5], [-0.5, 0.0, 0.5], [0.5, 0.0, 0.5], [0.5, 0.0, -0.5]]
    );
    // Assign a UV coordinate to each vertex.
    square_mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]]
    );
    // Assign normals (everything points upwards)
    square_mesh.insert_attribute(
       Mesh::ATTRIBUTE_NORMAL,
       vec![[0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0]]
    );
    square_mesh.set_indices(Some(Indices::U32(vec![
        // First triangle
        0, 3, 1,
        // Second triangle
        0, 3, 2
    ])));

    // add square mesh to resources
    grid_assets.square_mesh = meshes.add(square_mesh);

    // load grid indicator
    grid_assets.grid_indicator_texture = asset_server.load("system/grid_indicator.png");

    // create materials
    grid_assets.hostile_indicator = materials.add(StandardMaterial {
        base_color: Color::rgba(1.0, 0.576, 0.180, 0.9), // #ff932e
        base_color_texture: Some(grid_assets.grid_indicator_texture.clone()),
        alpha_mode: AlphaMode::Blend,
        depth_bias: 0.05,
        unlit: true,
        ..default()
    });

    grid_assets.support_indicator = materials.add(StandardMaterial {
        base_color: Color::rgba(0.184, 0.467, 0.922, 0.9), // #2f77eb
        base_color_texture: Some(grid_assets.grid_indicator_texture.clone()),
        alpha_mode: AlphaMode::Blend,
        depth_bias: 0.05,
        unlit: true,
        ..default()
    });
}

