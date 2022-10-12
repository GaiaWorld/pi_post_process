#version 450

#define SHADER_NAME fragment:DualBlur

layout(location = 0) in vec2 postiion_cs;
layout(location = 1) in float vAlpha;

layout(location = 0) out vec4 gl_FragColor;

layout(set = 0, binding = 1) uniform Param {
    vec2 center;
    float offset;
    float iteration;
    float start;
    float fade;
    vec2 _wasm_0;
};

layout(set = 0, binding = 2) uniform TextureMatrix {
    vec4 diffuseMat;
};

layout(set = 1, binding = 0) uniform sampler sampler_diffuseTex;
layout(set = 1, binding = 1) uniform texture2D diffuseTex;

// vec4 loop_f(int i, texture2D diffuseTex, sampler sampler_diffuseTex, vec2 uv, vec2 diff, float time) {
//     vec4 c = vec4(0., 0., 0., 0.);
//     if (i + 0.001 < time) {
//         c = texture(sampler2D(diffuseTex, sampler_diffuseTex), uv + i * diff);
//     }

//     return c;
// }

vec4 loop_n(texture2D diffuseTex, sampler sampler_diffuseTex, vec2 uv, vec2 diff, float time) {
    vec4 c = vec4(0., 0., 0., 0.);
    float count = 0.0;

    // for (int i = 0; i < 16; i++) {
    //     c += loop_f(i, diffuseTex, sampler_diffuseTex, uv, diff, time);

    //     // if (i + 0.001 < time) {
    //     //     count += 1.0;
    //     //     c += texture(sampler2D(diffuseTex, sampler_diffuseTex), uv + i * diff);
    //     // }

    //     // count += step(i + 0.001, time);
    //     // c += mix(
    //     //     vec4(0.),
    //     //     texture(sampler2D(diffuseTex, sampler_diffuseTex), uv + i * diff),
    //     //     step(i + 0.001, time)
    //     // );
    // }
    // count = min(16.0, time);
    
    for (int i = 0; i < 32; i++) {
        if (i == iteration) {
            break;
        }
        count += 1.0;
        c += texture(sampler2D(diffuseTex, sampler_diffuseTex), uv + i * diff);
    }

    return c / count;
}

vec4 loop_0(texture2D diffuseTex, sampler sampler_diffuseTex, vec2 uv, vec2 diff) {
    return texture(sampler2D(diffuseTex, sampler_diffuseTex), uv);
}

void main() {
    
    vec2 vMainUV = postiion_cs * diffuseMat.xy + diffuseMat.zw;

    vec4 c = vec4(0., 0., 0., 0.);

    float count = 0.;
    vec2 direct = (postiion_cs - vec2(0.5)) * 2.0 - center;
    float len = length(direct);
    float strength = smoothstep(start, start + fade, len);
    vec2 diff = normalize(direct) * offset * strength;

    if (0.001 < strength) {
        gl_FragColor = loop_n(diffuseTex, sampler_diffuseTex, vMainUV, diff, iteration);
    } else {
        gl_FragColor = loop_0(diffuseTex, sampler_diffuseTex, vMainUV, diff);
    }
    gl_FragColor.a *= vAlpha;
}