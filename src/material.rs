//! Materials.

use bevy::prelude::*;

use bevy::render::render_resource::{AsBindGroup, ShaderRef, ShaderType, AsBindGroupShaderType};
use bevy::render::render_asset::RenderAssets;
use bevy::reflect::{TypeUuid, TypePath};

/// Custom materials plugin for UI.
pub struct MaterialPlugin;

impl Plugin for MaterialPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(bevy::prelude::MaterialPlugin::<TileHighlightMaterial>::default());
    }
}

/// The material used to highlight areas on the grid.
#[derive(AsBindGroup, TypeUuid, TypePath, Debug, Clone)]
#[uniform(0, TileHighlightMaterialUniform)]
#[uuid = "23dba946-a4a1-43f0-a944-3610c5aee354"]
pub struct TileHighlightMaterial {
    pub color: Color,
    pub animate_speed: f32,
    #[texture(1)]
    #[sampler(2)]
    pub color_texture: Option<Handle<Image>>,
}

impl Material for TileHighlightMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/highlight_shader.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn depth_bias(&self) -> f32 {
        // TODO: this does not work for some reason
        1000.0
    }
}

/// The GPU representation of the uniform data of a [`TileHighlightMaterial`].
#[derive(Clone, Default, ShaderType)]
pub struct TileHighlightMaterialUniform {
    pub color: Vec4,
    pub animate_speed: f32,
}

impl AsBindGroupShaderType<TileHighlightMaterialUniform> for TileHighlightMaterial {
    fn as_bind_group_shader_type(&self, _images: &RenderAssets<Image>) -> TileHighlightMaterialUniform {
        TileHighlightMaterialUniform {
            color: self.color.as_linear_rgba_f32().into(),
            animate_speed: self.animate_speed,
        }
    }
}

