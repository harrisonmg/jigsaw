@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var texture_sampler: sampler;

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let color = textureSample(texture, texture_sampler, uv);

    // uncomment to view uv origin (0.0, 0.0)
    //if abs(uv.x - 0.5) < 0.005 || abs(uv.y - 0.5) < 0.005 {
    //    return vec4(1.0, 0.0, 0.0, 1.0);
    //}

    return color;
}
