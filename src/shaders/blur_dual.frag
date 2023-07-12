#version 450

#define SHADER_NAME fragment:DualBlur

layout(location = 0) in vec4 uv01;
layout(location = 1) in vec4 uv23;
layout(location = 2) in vec4 uv45;
layout(location = 3) in vec4 uv67;
layout(location = 5) in vec2 postiion_cs;

layout(location = 0) out vec4 gl_FragColor;


layout(set = 0, binding = 0) uniform Model {
    mat4 vertexMatrix;
    vec4 diffuseMat;

    vec2 offset;
    float intensity;
    float dualmode;
    
    float depth;
    float alpha;
    float src_preimultiplied;
    float dst_preimultiply;
};

layout(set = 0, binding = 1) uniform texture2D diffuseTex;
layout(set = 0, binding = 2) uniform sampler sampler_diffuseTex;

vec4 texColor(vec4 src) {
    src.rgb /= mix(1., src.a, step(0.5, src_preimultiplied));
    return src;
}

vec4 down(vec2 uv) {
    vec4 color =  texColor(texture(sampler2D(diffuseTex, sampler_diffuseTex), uv + uv01.xy ))
                + texColor(texture(sampler2D(diffuseTex, sampler_diffuseTex), uv + uv01.zw ))
                + texColor(texture(sampler2D(diffuseTex, sampler_diffuseTex), uv + uv23.xy ))
                + texColor(texture(sampler2D(diffuseTex, sampler_diffuseTex), uv + uv23.zw ));

    return color * 0.25;
}

vec4 up(vec2 uv) {
    vec4 color =  texColor(texture(sampler2D(diffuseTex, sampler_diffuseTex), uv + uv01.xy )) * 2.
                + texColor(texture(sampler2D(diffuseTex, sampler_diffuseTex), uv + uv01.zw )) * 2.
                + texColor(texture(sampler2D(diffuseTex, sampler_diffuseTex), uv + uv23.xy )) * 2.
                + texColor(texture(sampler2D(diffuseTex, sampler_diffuseTex), uv + uv23.zw )) * 2.
                + texColor(texture(sampler2D(diffuseTex, sampler_diffuseTex), uv + uv45.xy ))
                + texColor(texture(sampler2D(diffuseTex, sampler_diffuseTex), uv + uv45.zw ))
                + texColor(texture(sampler2D(diffuseTex, sampler_diffuseTex), uv + uv67.xy ))
                + texColor(texture(sampler2D(diffuseTex, sampler_diffuseTex), uv + uv67.zw ));
    return color * 0.0833333; // 1/12
}

void main() {
    vec2 vMainUV = postiion_cs * diffuseMat.zw + diffuseMat.xy;

    if (dualmode < 0.5) {
        gl_FragColor = down(vMainUV);
    } else {
        gl_FragColor = up(vMainUV);
    }
    
    gl_FragColor.a *= alpha;
    gl_FragColor.rgb *= mix(1., gl_FragColor.a, step(0.5, dst_preimultiply));
    gl_FragColor.rgb *= intensity;
}