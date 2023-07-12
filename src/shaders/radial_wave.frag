#version 450

#define SHADER_NAME fragment:RadialWave

layout(location = 0) in vec2 postiion_cs;

layout(location = 0) out vec4 gl_FragColor;

layout(set = 0, binding = 0) uniform Param {
    mat4 vertexMatrix;
    vec4 diffuseMat;

    float centerx;
    float centery;
    float aspect_ratio;
    float start;

    float end;
    float cycle;
    float weight;
    float depth;

    float alpha;
    float src_preimultiplied;
    float dst_preimultiply;
    // float wasm0;
    // float wasm1;
    float wasm2;
};


layout(set = 0, binding = 1) uniform texture2D diffuseTex;
layout(set = 0, binding = 2) uniform sampler sampler_diffuseTex;

vec4 texColor(vec4 src) {
    src.rgb /= mix(1., src.a, step(0.5, src_preimultiplied));
    return src;
}

void main() {

    vec2 vMainUV = postiion_cs * diffuseMat.zw + diffuseMat.xy;

    vec2 local = postiion_cs * 2.0 - 1.0;
    local.y *= aspect_ratio;

    float len = distance(local, vec2(centerx, centery));
    float width = end - start;
    float diff = max(width / 2., 0.1);

    float fade = smoothstep(start, start + diff, len) * (1.0 - smoothstep(end - diff, end, len));
    float t = (len - start) / width * cycle;
    diff = fade * weight * sin(t * 3.141592653589793);

    gl_FragColor = texColor(texture(sampler2D(diffuseTex, sampler_diffuseTex), fract(vMainUV + diff * diffuseMat.zw)));
    gl_FragColor.a *= alpha;
    gl_FragColor.rgb *= mix(1., gl_FragColor.a, step(0.5, dst_preimultiply));
}