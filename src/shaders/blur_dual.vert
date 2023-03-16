#version 450

#define SHADER_NAME vertex:DualBlur

layout(location = 0) in vec2 position;

layout(set = 0, binding = 0) uniform Model {
    mat4 vertexMatrix;
    vec4 diffuseMat;

    vec2 offset;
    float intensity;
    float dualmode;
    
    float depth;
    float alpha;
    vec2 _wasm_0;
};

layout(location = 0) out vec4 uv01;
layout(location = 1) out vec4 uv23;
layout(location = 2) out vec4 uv45;
layout(location = 3) out vec4 uv67;
layout(location = 5) out vec2 postiion_cs;

void main() {
    vec4 positionUpdate = vec4(position * 2.0, 1.0, 1.0);

    gl_Position = vertexMatrix * positionUpdate;
    gl_Position.z = depth;

    postiion_cs = position + 0.5;
    postiion_cs.y = 1.0 - postiion_cs.y;

    vec2 diff = offset * diffuseMat.zw;

    uv01 = vec4(vec2( diff.x,  diff.y), vec2(-diff.x,  diff.y));
    uv23 = vec4(vec2( diff.x, -diff.y), vec2(-diff.x, -diff.y));
    uv45 = vec4(vec2( diff.x * 2.,  0.),  vec2(-diff.x * 2.,  0.));
    uv67 = vec4(vec2( 0.,  diff.y * 2.),  vec2(0.,  diff.y * 2.));
}