@group(1) @binding(0)
var puzzle_texture: texture_2d<f32>;

@group(1) @binding(1)
var puzzle_sampler: sampler;

@group(1) @binding(2)
var mask_texture: texture_2d<f32>;

@group(1) @binding(3)
var mask_sampler: sampler;

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    // uv_rect is encoded in vertex color: xy = offset, zw = scale
    let image_uv = uv * color.zw + color.xy;
    let col = textureSample(puzzle_texture, puzzle_sampler, image_uv);
    let mask = textureSample(mask_texture, mask_sampler, uv);
    return vec4(col.rgb, col.a * mask.a);
}
