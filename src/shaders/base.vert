#version 450

layout(location = 0) in vec2 position;
layout(location = 0) out vec2 postiion_cs;
layout(location = 1) out float vAlpha;

layout(set = 0, binding = 0) uniform Model {
    mat4 vertexMatrix;
    float depth;
    float alpha;
    float src_preimultiplied;
    float dst_preimultiply;
};

void main() {
    vec4 positionUpdate = vec4(position * 2.0, 1.0, 1.0);

    gl_Position = vertexMatrix * positionUpdate;
    gl_Position.z = depth;

    postiion_cs = position + 0.5;
    postiion_cs.y = 1.0 - postiion_cs.y;

    vAlpha = alpha;
}