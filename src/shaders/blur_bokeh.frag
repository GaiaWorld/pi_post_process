#version 450

#define SHADER_NAME fragment:BlurDirect

layout(location = 0) in vec2 postiion_cs;

layout(location = 0) out vec4 gl_FragColor;

layout(set = 0, binding = 0) uniform Model {
    mat4 vertexMatrix;
    vec4 diffuseMat;

    vec2 center;
    float offset;
    float iteration;

    float start;
    float fade;
    float depth;
    float alpha;

    float src_preimultiplied;
    float dst_preimultiply;
    float _wasm_0;
    float _wasm_1;
};

layout(set = 0, binding = 1) uniform texture2D diffuseTex;
layout(set = 0, binding = 2) uniform sampler sampler_diffuseTex;

#define GLODEN_COS -0.7373688782616119
#define GLODEN_SIN 0.675490294061441
#define GLODEN_ROT mat2(GLODEN_COS, GLODEN_SIN, -GLODEN_SIN, GLODEN_COS)

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

vec4 BokehBlur(texture2D diffuseTex, sampler sampler_diffuseTex, vec2 uv, float blurRadius, float time) {

    vec4 accumulator = vec4(0.0);
    vec4 divisor = vec4(0.0);

    float r = 1.0;
    vec2 angle = vec2(0.0, blurRadius);

    vec2 tempuv = uv;

    for (int j = 0; j < 32; j++)
    {
        if (j == iteration) {
            break;
        }
        r += 1.0 / r;
        angle = GLODEN_ROT * angle;

        tempuv = uv + (r - 1.0) * angle;
        vec4 bokeh = texColor(texture(sampler2D(diffuseTex, sampler_diffuseTex), clampUV(tempuv, diffuseMat.xy, diffuseMat.zw + diffuseMat.xy) ));

        accumulator += bokeh * bokeh;
        divisor += bokeh;
    }

    return accumulator / divisor;
}

vec4 loop_0(texture2D diffuseTex, sampler sampler_diffuseTex, vec2 uv) {
    return texColor(texture(sampler2D(diffuseTex, sampler_diffuseTex), uv));
}

void main() {
    
    vec2 vMainUV = postiion_cs * diffuseMat.zw + diffuseMat.xy;

    vec4 c = vec4(0., 0., 0., 0.);

    float count = 0.;
    vec2 direct = (postiion_cs - vec2(0.5)) * 2.0 - center;
    float len = length(direct);
    float strength = smoothstep(start, start + fade, len);

    if (0.001 < strength) {
        gl_FragColor = BokehBlur(diffuseTex, sampler_diffuseTex, vMainUV, offset * strength, iteration);
    } else {
        gl_FragColor = loop_0(diffuseTex, sampler_diffuseTex, vMainUV);
    }
    gl_FragColor.a *= alpha;
    gl_FragColor.rgb *= mix(1., gl_FragColor.a, step(0.5, dst_preimultiply));
}