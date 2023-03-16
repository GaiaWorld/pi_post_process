#version 450

layout(location = 0) in vec2 position;
layout(location = 0) out vec2 postiion_cs;

layout(set = 0, binding = 0) uniform Param {
    mat4 vertexMatrix;
    vec4 diffuseMat;

    vec4 color;
    vec4 bgColor;

    vec2 uDiffUV;
    float clip;
    float depth;

    float alpha;
    float wasm0;
    float wasm1;
    float wasm2;
};

void main() {
    vec4 positionUpdate = vec4(position * 2.0, 1.0, 1.0);

    gl_Position = vertexMatrix * positionUpdate;
    gl_Position.z = depth;

    postiion_cs = position + 0.5;
    postiion_cs.y = 1.0 - postiion_cs.y;
}