#version 450

layout(location = 0) in vec2 postiion_cs;
layout(location = 1) in vec4 vGlitch;
layout(location = 0) out vec4 gl_FragColor;

layout(set = 0, binding = 0) uniform Model {
    mat4 vertexMatrix;
    vec4 diffuseMat;

    float strength;
    float fade;
    float depth;
    float alpha;
};


layout(set = 0, binding = 1) uniform texture2D diffuseTex;
layout(set = 0, binding = 2) uniform sampler sampler_diffuseTex;

void main() {
    vec2 vMainUV = postiion_cs * diffuseMat.zw + diffuseMat.xy;

    float diff_u = (1.0 - smoothstep(0.5 - fade / vGlitch.x, 0.5, abs(vGlitch.y - 0.5))) * strength * diffuseMat.z;
    // float diff_u = strength * diffuseMat.x;

    vec4 src1 = texture(sampler2D(diffuseTex, sampler_diffuseTex), vMainUV);
    vec4 src2 = texture(sampler2D(diffuseTex, sampler_diffuseTex), vMainUV + vec2(vGlitch.z * diff_u, 0.));
    vec4 src3 = texture(sampler2D(diffuseTex, sampler_diffuseTex), vMainUV + vec2(vGlitch.z * diff_u * 1.5, 0.));

    gl_FragColor = vec4(src1.r, src2.g, src3.b, (src1.a + src2.a + src3.a) * 0.333334);
    // gl_FragColor = vec4(diff_u, diff_u, diff_u, 1.0);
}