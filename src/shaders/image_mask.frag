#version 450

layout(set = 0, binding = 0) uniform Param {
    mat4 vertexMatrix;
    vec4 diffuseMat;

    vec4 maskMat;

    float maskFactor;
    float maskMode;
    float depth;
    float alpha;
};

layout(set = 0, binding = 1) uniform texture2D diffuseTex;
layout(set = 0, binding = 2) uniform sampler sampler_diffuseTex;

layout(set = 0, binding = 3) uniform texture2D maskTex;
layout(set = 0, binding = 4) uniform sampler sampler_maskTex;

layout(location = 0) in vec2 vUV;
layout(location = 1) in vec2 vUV2;

layout(location = 0) out vec4 gl_FragColor;

void main() {
    vec4 baseColor  = texture(sampler2D(diffuseTex, sampler_diffuseTex), vUV);
    float maskValue = texture(sampler2D(maskTex, sampler_maskTex), vUV2).r;
    maskValue = maskValue * 0.990 + 0.005;

    if (maskMode > 0.5) {
        baseColor.a *= step(maskFactor, maskValue) * maskValue;
    } else {
        baseColor *= step(maskFactor, maskValue);
    }

    baseColor.a *= alpha;

    gl_FragColor = baseColor;
}