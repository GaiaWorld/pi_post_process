#version 450

layout(location = 0) in vec2 position;

layout(location = 0) out vec2 vUV;
layout(location = 1) out vec2 vVertexPosition;

layout(set = 0, binding = 0) uniform Model {
    mat4 vertexMatrix;
    vec4 diffuseMat;

    vec4 sdfV0;
    vec4 sdfV1;
    vec4 sdfV2;
    vec4 sdfV3;

    float mode;
    float depth;
    float alpha;
    float wasm0;
};

void main() {
    vec4 positionUpdate = vec4(position * 2.0, 1.0, 1.0);

    gl_Position = vertexMatrix * positionUpdate;
    gl_Position.z = depth;

    vVertexPosition = position + 0.5;

    vec2 uv = position + 0.5;
    uv.y = 1.0 - uv.y;

    vUV = uv * diffuseMat.zw + diffuseMat.xy;
}