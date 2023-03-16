#version 450

layout(location = 0) in vec2 position;
layout(location = 0) out vec2 postiion_cs;

layout(set = 0, binding = 0) uniform ColorEffect {
    mat4 vertexMatrix;
    vec4 diffuseMat;
    
    float flag1;
    float color_balance_r;
    float color_balance_g;
    float color_balance_b;

    float flag2;
    float hsb_h;
    float hsb_s;
    float hsb_b;

    float flag3;
    float scale_shadow_in;
    float scale_shadow_out;
    float scale_mid;

    float scale_highlight_in;
    float scale_highlight_out;
    float flag4;
    float vignette_begin;

    float vignette_end;
    float vignette_scale;
    float vignette_r;
    float vignette_g;

    float vignette_b;
    float flag5;
    float filter_r;
    float filter_g;

    float filter_b;
    float depth;
    float alpha;
    float wasm0;
};

void main() {
    vec4 positionUpdate = vec4(position * 2.0, 1.0, 1.0);

    gl_Position = vertexMatrix * positionUpdate;
    gl_Position.z = depth;

    postiion_cs = position + 0.5;
    postiion_cs.y = 1.0 - postiion_cs.y;
}