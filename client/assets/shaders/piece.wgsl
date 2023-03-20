#import bevy_sprite::mesh2d_view_bindings

@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var texture_sampler: sampler;

@group(1) @binding(2)
var<uniform> sprite_origin: vec2<f32>;

/* sides to blur
 * bitmask:
 * 0b0001 = north
 * 0b0010 = south
 * 0b0100 = east
 * 0b1000 = west
 */
/*@group(1) @binding(3)*/
/*var sides: u32;*/
const sides: u32 = 15u;

const directions: f32 = 8.0;
const quality: f32 = 4.0;
const size: f32 = 1.0;

const pi2 = 6.28318530718;

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {

    let dim = vec2<f32>(textureDimensions(texture));
    let radius = size / dim;
    let color = textureSample(texture, texture_sampler, uv);
    var summed_color = color;

    for (var d = 0.0; d < pi2; d += pi2 / directions) {
        for (var i = 1.0 / quality; i <= 1.0; i += 1.0 / quality) {
            summed_color += textureSample(texture, texture_sampler, uv + vec2(cos(d), sin(d)) * radius * i);
        }
    }

    // only blur near edges
    if summed_color.w >= 1.0 + directions * quality || summed_color.w == 0.0 {
        return color;
    }

    let blurred_color = summed_color / (directions * quality + (directions / 2.0 - 1.0));

    let rot45 = mat2x2(0.70710678118, -0.70710678118, 0.70710678118, 0.70710678118);
    let min_dim = min(dim.x, dim.y);
    let uv = uv - sprite_origin;
    let hadamard = vec2(uv.x * dim.x, uv.y * dim.y);
    let uv_prime = rot45 * (hadamard / min_dim);

    // uncomment to debug edges

    if uv_prime.x < 0.0 {
     if uv_prime.y > 0.0 {
         // west
         return vec4(1.0, 0.0, 0.0, 1.0);
     } else if uv_prime.y < 0.0 {
         // north
         return vec4(0.0, 1.0, 0.0, 1.0);
     }
    } else {
     if uv_prime.y > 0.0 {
         // south
         return vec4(0.0, 0.0, 1.0, 1.0);
     } else if uv_prime.y < 0.0 {
         // east
         return vec4(1.0, 0.0, 1.0, 1.0);
     }
    }

    if uv_prime.x < 0.0 {
        if uv_prime.y > 0.0 && (sides & 8u) == 8u {
            // west
            return blurred_color;
        } else if uv_prime.y < 0.0 && (sides & 1u) == 1u {
            // north
            return blurred_color;
        }
    } else {
        if uv_prime.y > 0.0 && (sides & 2u) == 2u {
            // south
            return blurred_color;
        } else if uv_prime.y < 0.0 && (sides & 4u) == 4u {
            // east
            return blurred_color;
        }
    }

    return color;
}
