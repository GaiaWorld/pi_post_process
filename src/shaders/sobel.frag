#version 450

#define SHADER_NAME fragment:Sobel

layout(location = 0) in vec2 postiion_cs;

layout(location = 0) out vec4 gl_FragColor;

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

layout(set = 0, binding = 1) uniform texture2D diffuseTex;
layout(set = 0, binding = 2) uniform sampler sampler_diffuseTex;

const vec3 S0 = vec3(1., 2., 1.);
const vec3 S1 = vec3(1., 0., -1.);

float rgb2gray(vec3 rgb) {
    return 0.2126 * rgb.r + 0.7150 * rgb.g + 0.0722 * rgb.b;
}

void main() {
    
    vec2 vMainUV = postiion_cs * diffuseMat.zw + diffuseMat.xy;

    float g00 =  rgb2gray(texture(sampler2D(diffuseTex, sampler_diffuseTex), vMainUV + uDiffUV * vec2(-1., -1.)).rgb);
    float g01 =  rgb2gray(texture(sampler2D(diffuseTex, sampler_diffuseTex), vMainUV + uDiffUV * vec2(-0., -1.)).rgb);
    float g02 =  rgb2gray(texture(sampler2D(diffuseTex, sampler_diffuseTex), vMainUV + uDiffUV * vec2( 1., -1.)).rgb);
    float g10 =  rgb2gray(texture(sampler2D(diffuseTex, sampler_diffuseTex), vMainUV + uDiffUV * vec2(-1., -0.)).rgb);
    float g11 =  0.0;
    float g12 =  rgb2gray(texture(sampler2D(diffuseTex, sampler_diffuseTex), vMainUV + uDiffUV * vec2( 1., -0.)).rgb);
    float g20 =  rgb2gray(texture(sampler2D(diffuseTex, sampler_diffuseTex), vMainUV + uDiffUV * vec2(-1.,  1.)).rgb);
    float g21 =  rgb2gray(texture(sampler2D(diffuseTex, sampler_diffuseTex), vMainUV + uDiffUV * vec2( 0.,  1.)).rgb);
    float g22 =  rgb2gray(texture(sampler2D(diffuseTex, sampler_diffuseTex), vMainUV + uDiffUV * vec2( 1.,  1.)).rgb);

    mat3 a = mat3(
        g00, g01, g02,
        g10, g11, g12,
        g21, g21, g22
    );

    float gx = dot(S0, (S1 * a));
    float gy = dot(S1, (S0 * a));

    float g = sqrt(gx * gx + gy * gy);

    gl_FragColor = mix(bgColor, color, step(clip, g) * g);

    gl_FragColor.a *= alpha;
}