#version 450

layout(location = 0) in vec2 postiion_cs;
layout(location = 0) out vec4 gl_FragColor;

layout(set = 0, binding = 0) uniform ColorEffect {
    mat4 vertexMatrix;
    vec4 diffuseMat;
    
    float flag1;
    float hsb_h;
    float hsb_s;
    float hsb_b;

    float flag2;
    float color_balance_r;
    float color_balance_g;
    float color_balance_b;

    float flag3;
    float vignette_begin;
    float vignette_end;
    float vignette_scale;

    float vignette_r;
    float vignette_g;
    float vignette_b;
    float flag4;

    float scale_shadow_in;
    float scale_shadow_out;
    float scale_mid;
    float scale_highlight_in;

    float scale_highlight_out;
    float flag5;
    float filter_r;
    float filter_g;

    float filter_b;
    float depth;
    float alpha;
    float wasm0;
};

layout(set = 0, binding = 1) uniform texture2D diffuseTex;
layout(set = 0, binding = 2) uniform sampler sampler_diffuseTex;

vec3 rgb2hsv(vec3 c)
{
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c)
{
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, vec3(0.0), vec3(1.0)), c.y);
}

vec3 colorBalance(vec3 rgb, vec3 cb) {
    return pow(rgb, cb);
}

vec3 colorFilter(vec3 rgb, vec3 cf) {
    return rgb * cf;
}

vec3 applyHSV(vec3 c, vec3 pHSV) {
    vec3 hsv = rgb2hsv(c.rgb);
    hsv.r += pHSV.r;
    c.rgb = hsv2rgb(hsv);

    // Note: When saturate is greater than 0, the formula is different from PS
    float gray = max(c.r, max(c.g, c.b)) + min(c.r, min(c.g, c.b));
    c.rgb = mix(c.rgb, vec3(0.5 * gray), -pHSV.g);

    if (pHSV.b >= 0.0) {
        c.rgb = mix(c.rgb, vec3(1.0), pHSV.b);
    } else {
        c.rgb *= 1.0 + pHSV.b;
    }

    return c;
}

vec3 applyColorScale(vec3 rgb, float csMid, float csInS, float csInH, float csOutS, float csOutH) {
    float csD = csInH - csInS;
    rgb = clamp((rgb - vec3(csInS)) / csD, 0.0, 1.0);
    rgb = pow(rgb, vec3(csMid, csMid, csMid));
    csD = csOutH - csOutS;
    rgb = clamp(rgb * csD + vec3(csOutS), 0.0, 1.0);
    return rgb;
}

vec3 vignette(vec3 rgb, vec2 uv, float start, float end, float scale, vec3 color) {
    float dist = distance(uv, vec2(0.5, 0.5)) * 2.0;
    dist = smoothstep(start, end, dist * scale);

    return mix(rgb, color, dist);
}

void main() {
    vec2 vMainUV = postiion_cs * diffuseMat.zw + diffuseMat.xy;

    vec4 c = texture(sampler2D(diffuseTex, sampler_diffuseTex), vMainUV);

    if (flag1 > 0.0) {
        c.rgb = colorBalance(c.rgb, vec3(color_balance_r, color_balance_g, color_balance_b));
    }

    if (flag2 > 0.0) {
        c.rgb = applyHSV(c.rgb, vec3(hsb_h, hsb_s, hsb_b));
    }

    if (flag3 > 0.0) {
        c.rgb = applyColorScale(c.rgb, scale_mid, scale_shadow_in, scale_highlight_in, scale_shadow_out, scale_highlight_out);
    }

    if (flag4 > 0.0) {
        c.rgb = vignette(c.rgb, postiion_cs, vignette_begin, vignette_end, vignette_scale, vec3(vignette_r, vignette_g, vignette_b));
    }

    if (flag5 > 0.0) {
        c.rgb = colorFilter(c.rgb, vec3(filter_r, filter_g, filter_b));
    }

    gl_FragColor = c;
    gl_FragColor.a *= alpha;
}