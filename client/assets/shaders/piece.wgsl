#import bevy_sprite::mesh2d_view_bindings

@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var texture_sampler: sampler;

struct Params {
   params.sprite_origin: vec2<f32>,

   // sides to blur
   // bitmask:
   // 0b0001 = north
   // 0b0010 = south
   // 0b0100 = east
   // 0b1000 = west
   sides: u32,

   _padding_1: u32,
   _padding_2: u32,
   _padding_3: u32,
   _padding_4: u32,
   _padding_5: u32,
   _padding_6: u32,
   _padding_7: u32,
   _padding_8: u32,
   _padding_9: u32,
   _padding_10: u32,
   _padding_11: u32,
   _padding_12: u32,
   _padding_13: u32,
   _padding_14: u32,
   _padding_15: u32,
   _padding_16: u32,
   _padding_17: u32,
   _padding_18: u32,
   _padding_19: u32,
   _padding_20: u32,
   _padding_21: u32,
   _padding_22: u32,
   _padding_23: u32,
   _padding_24: u32,
   _padding_25: u32,
   _padding_26: u32,
   _padding_27: u32,
   _padding_28: u32,
   _padding_29: u32,
   _padding_30: u32,
   _padding_31: u32,
   _padding_32: u32,
   _padding_33: u32,
   _padding_34: u32,
   _padding_35: u32,
   _padding_36: u32,
   _padding_37: u32,
   _padding_38: u32,
   _padding_39: u32,
   _padding_40: u32,
   _padding_41: u32,
   _padding_42: u32,
   _padding_43: u32,
   _padding_44: u32,
   _padding_45: u32,
}

@group(1) @binding(2)
var<uniform> params: Param;


const directions: f32 = 8.0;
const quality: f32 = 4.0;
const size: f32 = 1.0;

const pi2 = 6.28318530718;

@fragment
fn fragment(
   #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    return vec4(1.0, 0.0, 0.0, 0.0);
}

/*@fragment*/
/*fn fragment(*/
/*    #import bevy_pbr::mesh_vertex_output*/
/*) -> @location(0) vec4<f32> {*/

/*    let dim = vec2<f32>(textureDimensions(texture));*/
/*    let radius = size / dim;*/
/*    let color = textureSample(texture, texture_sampler, uv);*/
/*    var summed_color = color;*/

/*    for (var d = 0.0; d < pi2; d += pi2 / directions) {*/
/*        for (var i = 1.0 / quality; i <= 1.0; i += 1.0 / quality) {*/
/*            summed_color += textureSample(texture, texture_sampler, uv + vec2(cos(d), sin(d)) * radius * i);*/
/*        }*/
/*    }*/

/*    // uncomment to view uv coord (0.0, 0.0)*/

/*    // if abs(uv.x - 0.5) < 0.005 || abs(uv.y - 0.5) < 0.005 {*/
/*    //    return vec4(1.0, 0.0, 0.0, 1.0);*/
/*    // }*/

/*    // only blur near edges*/
/*    if summed_color.w >= 1.0 + directions * quality || summed_color.w == 0.0 {*/
/*        return color;*/
/*    }*/

/*    let blurred_color = summed_color / (directions * quality + (directions / 2.0 - 1.0));*/

/*    let rot45 = mat2x2(0.70710678118, -0.70710678118, 0.70710678118, 0.70710678118);*/
/*    let min_dim = min(dim.x, dim.y);*/
/*    let uv = uv - params.sprite_origin;*/
/*    let hadamard = vec2(uv.x * dim.x, uv.y * dim.y);*/
/*    let uv_prime = rot45 * (hadamard / min_dim);*/

/*    // uncomment to debug edges*/

/*    // if uv_prime.x < 0.0 {*/
/*    //    if uv_prime.y > 0.0 {*/
/*    //        // west*/
/*    //        return vec4(1.0, 0.0, 0.0, 1.0);*/
/*    //    } else if uv_prime.y < 0.0 {*/
/*    //        // north*/
/*    //        return vec4(0.0, 1.0, 0.0, 1.0);*/
/*    //    }*/
/*    // } else {*/
/*    //    if uv_prime.y > 0.0 {*/
/*    //        // south*/
/*    //        return vec4(0.0, 0.0, 1.0, 1.0);*/
/*    //    } else if uv_prime.y < 0.0 {*/
/*    //        // east*/
/*    //        return vec4(1.0, 0.0, 1.0, 1.0);*/
/*    //    }*/
/*    // }*/

/*    if uv_prime.x < 0.0 {*/
/*        if uv_prime.y > 0.0 && (params.sides & 8u) == 8u {*/
/*            // west*/
/*            return blurred_color;*/
/*        } else if uv_prime.y < 0.0 && (params.sides & 1u) == 1u {*/
/*            // north*/
/*            return blurred_color;*/
/*        }*/
/*    } else {*/
/*        if uv_prime.y > 0.0 && (params.sides & 2u) == 2u {*/
/*            // south*/
/*            return blurred_color;*/
/*        } else if uv_prime.y < 0.0 && (params.sides & 4u) == 4u {*/
/*            // east*/
/*            return blurred_color;*/
/*        }*/
/*    }*/

/*    return color;*/
/*}*/
