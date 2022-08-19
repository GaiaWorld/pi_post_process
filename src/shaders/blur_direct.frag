#version 450

#define SHADER_NAME fragment:BlurDirect

layout(location = 0) in vec2 postiion_cs;

layout(location = 0) out vec4 gl_FragColor;

layout(set = 0, binding = 1) uniform Param {
    vec2 direct;
    float offset;
    float iteration;
};

layout(set = 0, binding = 2) uniform TextureMatrix {
    vec4 diffuseMat;
};

layout(set = 1, binding = 0) uniform sampler sampler_diffuseTex;
layout(set = 1, binding = 1) uniform texture2D diffuseTex;

vec4 loop_n(texture2D diffuseTex, sampler sampler_diffuseTex, vec2 uv, vec2 diff, float time) {
    vec4 c = vec4(0., 0., 0., 0.);
    float count = 0.0;
    // for (int i = 0; i < 16; i++) {
    //     count += step(i + 0.01, time);
    //     c += mix(
    //         texture(sampler2D(diffuseTex, sampler_diffuseTex), uv + i * diff),
    //         vec4(0.),
    //         step(i + 0.01, time)
    //     );
    // }
    for (int i = 0; i < 8; i++) {
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

    vec2 diff = normalize(direct) * offset;

    gl_FragColor = loop_n(diffuseTex, sampler_diffuseTex, vMainUV, diff, iteration);

}