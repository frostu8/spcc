#import bevy_pbr::mesh_view_bindings globals
#import bevy_pbr::mesh_vertex_output MeshVertexOutput

struct TileHighlightMaterial {
    color: vec4<f32>,
    animate_speed: f32,
};

@group(1) @binding(0)
var<uniform> material: TileHighlightMaterial;
@group(1) @binding(1)
var color_texture: texture_2d<f32>;
@group(1) @binding(2)
var color_sampler: sampler;

@fragment
fn fragment(mesh: MeshVertexOutput) -> @location(0) vec4<f32> {
    let scroll = globals.time * material.animate_speed;

    let uv_u = mesh.uv.x + scroll - trunc(mesh.uv.x + scroll);
    let uv_v = mesh.uv.y + scroll - trunc(mesh.uv.y + scroll);

    return material.color * textureSample(color_texture, color_sampler, vec2<f32>(uv_u, uv_v));
    //return material.color * textureSample(color_texture, color_sampler, mesh.uv);
}
