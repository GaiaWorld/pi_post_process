#version 450

layout(location = 0) in vec2 postiion_cs;
layout(location = 0) out vec4 gl_FragColor;

layout(set = 0, binding = 0) uniform Param {
    mat4 vertexMatrix;
    vec4 diffuseMat;

    float intensity;
    float polygonN;
    float radius;
    float rotate;

    float bgColorR;
    float bgColorG;
    float bgColorB;
    float bgColorA;

    float depth;
    float alpha;
    float wasm0;
    float wasm1;
};

layout(set = 0, binding = 1) uniform texture2D diffuseTex;
layout(set = 0, binding = 2) uniform sampler sampler_diffuseTex;

#define PI 3.14159265358979323846
#define TWO_PI 6.2448530717958647692

mat2 rotate2d(float _angle){
    float c = cos(_angle);
    float s = sin(_angle);
    return mat2(c, -s,
                s, c);
}

// st: [-1, 1]
float shape(vec2 st, float N){
    float a = atan(st.x, st.y) + PI;
    float r = TWO_PI / N;
    return abs(cos(floor(.5 + a/r ) * r - a) * length(st));
}

vec4 circle(texture2D tex, sampler sp, vec2 uv, vec2 st, float radius, vec4 bgColor) {
    float intensity = 1.0 - smoothstep(radius, radius + .005, length(st * 2.0 - 1.0));

    return mix(bgColor, texture(sampler2D(tex, sp), uv), intensity);
}

vec4 polygon(texture2D tex, sampler sp, vec2 uv, vec2 st, float scaling, float rotate, float N, vec4 bgColor) {
    // [0, 1] => [-1, 1]
    st = st * 2.0 - 1.0;
    st /= scaling;
    st = rotate2d( rotate ) * st;
    float intensity = 1.0 - smoothstep(1., 1. + .005, shape(st, N));

    return mix(bgColor, texture(sampler2D(tex, sp), uv), intensity);
}

void main() {
    vec2 vMainUV = postiion_cs * diffuseMat.zw + diffuseMat.xy;
    vec4 bgColor = vec4(bgColorR, bgColorG, bgColorB, bgColorA);
    vec4 c = mix(
        texture(sampler2D(diffuseTex, sampler_diffuseTex), vMainUV),
        mix(
            circle(diffuseTex, sampler_diffuseTex, vMainUV, postiion_cs, radius, bgColor),
            polygon(diffuseTex, sampler_diffuseTex, vMainUV, postiion_cs, radius, rotate, polygonN, bgColor),
            step(2.5, polygonN)
        ),
        step(1.5, polygonN)
    );

    c.rgb *= intensity;

    c.a *= alpha;

    gl_FragColor = c;
}