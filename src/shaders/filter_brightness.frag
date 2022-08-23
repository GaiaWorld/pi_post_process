#version 450

#define SHADER_NAME fragment:FilterBrightness

layout(location = 0) in vec2 postiion_cs;
layout(location = 1) in float vAlpha;

layout(location = 0) out vec4 gl_FragColor;

layout(set = 0, binding = 1) uniform Param {
    vec4 threshold;
};

layout(set = 0, binding = 2) uniform TextureMatrix {
    vec4 diffuseMat;
};

layout(set = 1, binding = 0) uniform sampler sampler_diffuseTex;
layout(set = 1, binding = 1) uniform texture2D diffuseTex;

vec3 ApplyBrightnessThreshold (vec3 color, vec4 _BloomThreshold) {
    float brightness = max(color.r, max(color.g, color.b));
    float soft = brightness + _BloomThreshold.y;
    soft = clamp(soft, 0.0, _BloomThreshold.z);
    soft = soft * soft * _BloomThreshold.w;
    float contribution = max(soft, brightness - _BloomThreshold.x);
    contribution /= max(brightness, 0.00001);
    return color * contribution;
}

void main() {
    
    vec2 vMainUV = postiion_cs * diffuseMat.xy + diffuseMat.zw;

    vec4 c = texture(sampler2D(diffuseTex, sampler_diffuseTex), vMainUV);
    c.rgb = ApplyBrightnessThreshold(c.rgb, threshold);

    gl_FragColor = c;
    gl_FragColor.a *= vAlpha;
}