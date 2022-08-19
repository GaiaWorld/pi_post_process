#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 glitch;

layout(location = 0) out vec2 postiion_cs;
layout(location = 1) out vec2 vGlitch;

layout(set = 0, binding = 0) uniform Model {
    mat4 vertexMatrix;
};

void main() {
    vec2 positionGlitch = position + 0.5; // [0, 1]
    positionGlitch.y = positionGlitch.y * glitch.y + glitch.x;

    vec4 positionUpdate = vec4((positionGlitch - 0.5) * 2.0, 1.0, 1.0);

    gl_Position = vertexMatrix * positionUpdate;

    postiion_cs = positionGlitch;
    postiion_cs.y = 1.0 - positionGlitch.y;

    float halfSize = glitch.y / 2.;
    vGlitch = vec2(halfSize, position.y + 0.5);
}