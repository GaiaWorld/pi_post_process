#version 450

#define SHADER_NAME fragment:FilterBrightness

layout(location = 0) in vec2 postiion_cs;

layout(location = 0) out vec4 gl_FragColor;

layout(set = 0, binding = 0) uniform Param {
    mat4 vertexMatrix;
    vec4 diffuseMat;

    vec4 threshold;

    float depth;
    float alpha;
    float wasm0;
    float wasm1;
};


layout(set = 0, binding = 1) uniform texture2D diffuseTex;
layout(set = 0, binding = 2) uniform sampler sampler_diffuseTex;

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
    
    vec2 vMainUV = postiion_cs * diffuseMat.zw + diffuseMat.xy;

    vec4 c = texture(sampler2D(diffuseTex, sampler_diffuseTex), vMainUV);
    c.rgb = ApplyBrightnessThreshold(c.rgb, threshold);

    gl_FragColor = c;
    gl_FragColor.a *= alpha;
}