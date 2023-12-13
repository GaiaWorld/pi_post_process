#version 450

#define SHADER_NAME fragment:BlurDirect

layout(location = 0) in vec2 postiion_cs;

layout(location = 0) out vec4 gl_FragColor;

layout(set = 0, binding = 0) uniform Model {
    mat4 vertexMatrix;
    vec4 diffuseMat;

    vec2 direct;
    float offset;
    float iteration;

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

vec2 clampUV(vec2 uv, vec2 minUV, vec2 maxUV) {
    return vec2(
        clamp(uv.x, minUV.x, maxUV.x),
        clamp(uv.y, minUV.y, maxUV.y)
    );
}

vec4 loop_n(texture2D diffuseTex, sampler sampler_diffuseTex, vec2 uv, vec2 diff, float time) {
    vec4 c = vec4(0., 0., 0., 0.);
    float count = 0.0;
    // for (int i = 0; i < 16; i++) {
    //     count += step(i + 0.01, time);
    //     c += mix(
    //         texture(sampler2D(diffuseTex, sampler_diffuseTex), uv + i * diff),
    //         vec4(0.),
    //         step(i + 0.01, time)
    //     );
    // }
    for (int i = 0; i < 32; i++) {
        if (i == iteration) {
            break;
        }
        count += 1.0;
        c += texColor(texture(sampler2D(diffuseTex, sampler_diffuseTex), clampUV(uv + i * diff, diffuseMat.xy, diffuseMat.zw + diffuseMat.xy) ));
    }

    return c / count;
}

vec4 loop_0(texture2D diffuseTex, sampler sampler_diffuseTex, vec2 uv, vec2 diff) {
    return texture(sampler2D(diffuseTex, sampler_diffuseTex), uv);
}

void main() {
    
    vec2 vMainUV = postiion_cs * diffuseMat.zw + diffuseMat.xy;

    vec4 c = vec4(0., 0., 0., 0.);

    vec2 diff = normalize(direct) * offset;

    gl_FragColor = loop_n(diffuseTex, sampler_diffuseTex, vMainUV, diff, iteration);
    gl_FragColor.a *= alpha;
    gl_FragColor.rgb *= mix(1., gl_FragColor.a, step(0.5, dst_preimultiply));

}