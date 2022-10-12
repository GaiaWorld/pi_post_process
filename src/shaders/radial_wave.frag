#version 450

#define SHADER_NAME fragment:RadialWave

layout(location = 0) in vec2 postiion_cs;
layout(location = 1) in float vAlpha;

layout(location = 0) out vec4 gl_FragColor;

layout(set = 0, binding = 1) uniform Param {
    float centerx;
    float centery;
    float aspect_ratio;
    float start;
    float end;
    float cycle;
    float weight;
    float _wasm_0;
};

layout(set = 0, binding = 2) uniform TextureMatrix {
    vec4 diffuseMat;
};

layout(set = 1, binding = 0) uniform sampler sampler_diffuseTex;
layout(set = 1, binding = 1) uniform texture2D diffuseTex;

void main() {

    vec2 vMainUV = postiion_cs * diffuseMat.xy + diffuseMat.zw;

    vec2 local = postiion_cs * 2.0 - 1.0;
    local.y *= aspect_ratio;

    float len = distance(local, vec2(centerx, centery));
    float width = end - start;
    float diff = max(width / 2., 0.1);

    float fade = smoothstep(start, start + diff, len) * (1.0 - smoothstep(end - diff, end, len));
    float t = (len - start) / width * cycle;
    diff = fade * weight * sin(t * 3.141592653589793);

    gl_FragColor = texture(sampler2D(diffuseTex, sampler_diffuseTex), fract(vMainUV + diff * diffuseMat.xy));
    gl_FragColor.a *= vAlpha;
}