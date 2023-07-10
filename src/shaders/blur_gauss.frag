#version 450

layout(set = 0, binding = 0) uniform Model {
    mat4 vertexMatrix;
    vec4 diffuseMat;

    // vec4 uvRegion; // uv 矩形范围、采样范围不能超过该矩形区域，因此需要该参数用于判断
    vec2 textureSize; // 纹理尺寸
    float blurRadius; // 模糊半径
    float horizontal;

    float depth;
    float alpha;
    float src_preimultiplied;
    float dst_preimultiply;
};

layout(set = 0, binding = 1) uniform texture2D diffuseTex;
layout(set = 0, binding = 2) uniform sampler sampler_diffuseTex;

layout(location = 0) in float support; // 采样像素个数（大致为模糊半径的两倍，因为需要左右对称）

layout(location = 1) in vec2 vUv; // 当前点得uv坐标
layout(location = 2) in vec2 vOffsetScale; // 单个像素的uv偏移
layout(location = 3) in vec2 vGaussCoefficients; // 高斯模糊系数（由顶点着色器根据模糊半径算出）
layout(location = 4) in vec4 uvRect; 

layout(location = 0) out vec4 gl_FragColor;

vec4 texColor(vec4 src) {
    src.rgb /= mix(1., src.a, step(0.5, src_preimultiplied));
    return src;
}

void main() {
    vec4 original_color = texColor(texture(sampler2D(diffuseTex, sampler_diffuseTex), vUv));

    // Incremental Gaussian Coefficent Calculation (See GPU Gems 3 pp. 877 - 889)
    vec3 gauss_coefficient = vec3(vGaussCoefficients,
                                vGaussCoefficients.y * vGaussCoefficients.y);
    // 当前采样点的权重
    vec4 avg_color = original_color * gauss_coefficient.x;
    
    // 其他权重的点的采样（左右需要对称）
    for (float i = 1.0; i <= 300.0; i += 2.0) {
        if (i > support) {
            break;
        }
        gauss_coefficient.xy *= gauss_coefficient.yz;
        float gauss_coefficient_subtotal = gauss_coefficient.x;
        gauss_coefficient.xy *= gauss_coefficient.yz;
        gauss_coefficient_subtotal += gauss_coefficient.x;

        float gauss_ratio = gauss_coefficient.x / gauss_coefficient_subtotal;
        vec2 offset = vOffsetScale * (i + gauss_ratio);
        
        // 计算负方向和正方向上偏移的像素的像素值
        vec2 st0 = vUv - offset;
        vec2 st1 = vUv + offset;
        st0 = vec2(max(st0.x, uvRect.x), max(st0.y, uvRect.y));
        st1 = vec2(min(st1.x, uvRect.z), min(st1.y, uvRect.w));
        avg_color += (texColor(texture(sampler2D(diffuseTex, sampler_diffuseTex), st0)) + texColor(texture(sampler2D(diffuseTex, sampler_diffuseTex), st1))) *
                    gauss_coefficient_subtotal;
                    
    }

    // 输出颜色值
    gl_FragColor.rgb *= mix(1., gl_FragColor.a, step(0.5, dst_preimultiply));
    gl_FragColor = avg_color;
    gl_FragColor.a *= alpha;

    // gl_FragColor=vec4(0.0, )
}
