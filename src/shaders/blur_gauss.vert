#version 450

layout(location = 0) in vec2 position;

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

layout(location = 0) out float support; // 采样像素个数（大致为模糊半径的两倍，因为需要左右对称）

layout(location = 1) out vec2 vUv; // 当前点得uv坐标
layout(location = 2) out vec2 vOffsetScale; // 单个像素的uv偏移
layout(location = 3) out vec2 vGaussCoefficients; // 高斯模糊系数（由顶点着色器根据模糊半径算出）
layout(location = 4) out vec4 uvRect; 

// 根据半径计算高斯函数最大值（该最大值）
// 高斯模糊公式： f(x) = 1/(√2πσ²) * e^-x²/2σ²
// 公式中存在两个常量 A: 1/(√2πσ²), B: e^-/2σ²
// 因此，该公式符合二项式定理:
// 当x = 0时， f(x) = A, 
// 当x = 1时， f(x) = A*B, 令 C = B²
// 当x = 2时， f(x) = (A*B)*C, 令C1= C*B²
// 当x = 4时， f(x) = (A*B)*C1, 令C2= C1*B² ....
void calculate_gauss_coefficients(float sigma) {
	vGaussCoefficients = vec2(1.0 / (sqrt(2.0 * 3.14159265) * sigma),
                              exp(-0.5 / (sigma * sigma)));

	// x: A, y: B, z: B²
    vec3 gauss_coefficient = vec3(vGaussCoefficients,
                                  vGaussCoefficients.y * vGaussCoefficients.y);
	
	// 积分 对覆盖到的像素权重求和
    float gauss_coefficient_total = gauss_coefficient.x;

	// int support = int(ceil(support));
    for (float i = 1.0; i <= 300.0; i += 2.0) {
		if (i > support) {
			break;
		}
        gauss_coefficient.xy *= gauss_coefficient.yz;
        float gauss_coefficient_subtotal = gauss_coefficient.x;
        gauss_coefficient.xy *= gauss_coefficient.yz;
        gauss_coefficient_subtotal += gauss_coefficient.x;

		// 除x=0的像素，其他像素都是对称的，所以乘2
        gauss_coefficient_total += 2.0 * gauss_coefficient_subtotal;
    }

	// 求x=0是的权重值（原权重/总权重，因为求值过程是乘的操作，因此用该值重复上面的计算，可得到其他像素的新权重）
    vGaussCoefficients.x = vGaussCoefficients.x/gauss_coefficient_total;
}

void main() {
    vec4 positionUpdate = vec4(position * 2.0, 1.0, 1.0);

    gl_Position = vertexMatrix * positionUpdate;
    gl_Position.z = depth;

    vUv = position + 0.5;
    vUv.y = 1.0 - vUv.y;

	support = blurRadius;
	support = ceil(1.5 * blurRadius) * 2.0;
    if (support > 0.0) {
		// 以σ为blurRadius来计算权重值
        calculate_gauss_coefficients(blurRadius);
    } else {
		// support不大于0，则默认为1.0
        vGaussCoefficients = vec2(1.0, 1.0);
    }

	// 计算一个像素在纹理上的uv偏移（0~1的数）
	vOffsetScale = mix(vec2(0., textureSize.y), vec2(textureSize.x, 0.), step(0.01, horizontal));

	// 像素单位计算到uv单位，因为是采用线性采样，为防止溢出，这里会有一定偏移
	// uvRect = vec4(uvRegion.xy + 0.5, uvRegion.zw - 0.5) / textureSize.xyxy;
	uvRect = vec4(diffuseMat.xy + textureSize * 0.5, diffuseMat.zw + diffuseMat.xy - textureSize * 0.5);

	vUv = vUv * diffuseMat.zw + diffuseMat.xy;
}