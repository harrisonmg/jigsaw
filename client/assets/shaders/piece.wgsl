@group(1) @binding(0)
var texture: texture_2d<f32>;
@group(1) @binding(1)
var texture_sampler: sampler;

const sample_dist: f32 = 0.001;
const transparency_weight: f32 = 10.0;

fn scaled_alpha(x: f32, y: f32) -> f32 {
    let alpha = textureSample(texture, texture_sampler, vec2<f32>(x, y)).w;
    return alpha * transparency_weight - (transparency_weight - 1.0);
}

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let color = textureSample(texture, texture_sampler, uv);
    var alpha = color.w;

    let left_alpha = scaled_alpha(uv.x - sample_dist, uv.y);
    let right_alpha = scaled_alpha(uv.x + sample_dist, uv.y);
    let up_alpha = scaled_alpha(uv.x, uv.y + sample_dist);
    let down_alpha = scaled_alpha(uv.x, uv.y - sample_dist);

    alpha += left_alpha + right_alpha + up_alpha + down_alpha;
    alpha = min(max(alpha / 5.0, 0.0), 1.0);

    return vec4<f32>(color.x, color.y, color.z, alpha);
}
