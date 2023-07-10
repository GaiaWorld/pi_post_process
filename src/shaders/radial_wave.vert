#version 450

layout(location = 0) in vec2 position;
layout(location = 0) out vec2 postiion_cs;

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

void main() {
    vec4 positionUpdate = vec4(position * 2.0, 1.0, 1.0);

    gl_Position = vertexMatrix * positionUpdate;
    gl_Position.z = depth;
    gl_Position.w = 1.0;

    postiion_cs = position + 0.5;
    postiion_cs.y = 1.0 - postiion_cs.y;
}