@group(1) @binding(0)
var puzzle_texture: texture_2d<f32>;

@group(1) @binding(1)
var puzzle_sampler: sampler;

@group(1) @binding(2)
var mask_texture: texture_2d<f32>;

@group(1) @binding(3)
var mask_sampler: sampler;

@group(1) @binding(4)
var<uniform> uv_rect: vec4<f32>;

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let image_uv = uv * uv_rect.zw + uv_rect.xy;
    let color = textureSample(puzzle_texture, puzzle_sampler, image_uv);
    let mask = textureSample(mask_texture, mask_sampler, uv);
    return vec4(color.rgb, color.a * mask.a);
}
