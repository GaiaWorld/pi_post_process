#version 450

layout(set = 0, binding = 0) uniform Param {
    mat4 vertexMatrix;
    vec4 diffuseMat;

    vec4 clipSdf0;
    vec4 clipSdf1;
    vec4 clipSdf2;
    vec4 clipSdf3;

    float mode;
    float depth;
    float alpha;
    float wasm0;
};

layout(set = 0, binding = 1) uniform texture2D diffuseTex;
layout(set = 0, binding = 2) uniform sampler sampler_diffuseTex;

layout(location = 0) in vec2 vUV;
layout(location = 1) in vec2 vVertexPosition;

layout(location = 0) out vec4 gl_FragColor;

// 根据 d, 抗锯齿, 返回 alpha值
float antialiase(float d) 
{
    float anti = 1.0 * fwidth(d);
    
    // smoothstep(-a, a, d) 意思是 根据 d-值 将 [-a, a] 平滑到 [0, 1] 中
    // d < -a, 全内部, 得到0, 这时期望 alpha = 1.0
    // d > a, 全外部, 得到1, 这时期望 alpha = 0.0
    
    return 1.0 - smoothstep(-anti, anti, d);
}

// Border Radius
        float sdfEllipse(vec2 pt, vec2 center, vec2 ab)
        {
            pt -= center;
            
            // 求 (1/a, 1/b)
            vec2 recAB = 1.0 / ab;
            // 求 (x/a, y/b) = (x, y) * (1/a, 1/b)
            vec2 scale = pt * recAB;
            
            // 椭圆值 f = (x/a)^2 + (y/b)^2 - 1
            return dot(scale, scale) - 1.0;
        }

        float sdfRect(vec2 pt, vec2 wh)
        {
            vec2 d = abs(pt) - wh;
            return length(max(d, 0.0)) + min(max(d.x, d.y), 0.0);
        }

        float cross_pt(vec2 v1, vec2 v2) {
            return -(v1.x * v2.y - v1.y * v2.x);
        }
        // p0, p1, p2 是否 逆时针
        bool is_ccw(vec2 p0, vec2 p1, vec2 p2) {
            vec2 v1 = p1 - p0;
            vec2 v2 = p2 - p0;
            float r = cross_pt(v1, v2);
            return r > 0.0;
        }
        bool is_left_top(vec2 pt, vec2 wh, vec2 center) {
            vec2 pt0 = vec2(-wh.x, center.y);
            vec2 pt1 = vec2(center.x, -wh.y);
            return is_ccw(pt, pt0, pt1);
        }
        bool is_top_right(vec2 pt, vec2 wh, vec2 center) {
            vec2 pt0 = vec2(center.x, -wh.y);
            vec2 pt1 = vec2(wh.x, center.y);
            return is_ccw(pt, pt0, pt1);
        }
        bool is_right_bottom(vec2 pt, vec2 wh, vec2 center) {
            vec2 pt0 = vec2(wh.x, center.y);
            vec2 pt1 = vec2(center.x, wh.y);
            return is_ccw(pt, pt0, pt1);
        }
        bool is_bottom_left(vec2 pt, vec2 wh, vec2 center) {
            vec2 pt0 = vec2(center.x, wh.y);
            vec2 pt1 = vec2(-wh.x, center.y);
            return is_ccw(pt, pt0, pt1);
        }
        float antialiase_round_rect(vec2 pt, vec2 extent, vec2 offset1, vec2 offset2, vec2 offset3, vec2 offset4) {
            
            float d_rect = sdfRect(pt, extent);
            float a_rect = antialiase(d_rect);
            vec2 center = vec2(-extent.x + offset1.x, -extent.y + offset1.y); 
            if (is_left_top(pt, extent, center)) {
                float d = sdfEllipse(pt, center, abs(offset1));
                float a = antialiase(d);
                return min(a_rect, a);
            }
            center = vec2(extent.x + offset2.x, -extent.y + offset2.y); 
            if (is_top_right(pt, extent, center)) {
                float d = sdfEllipse(pt, center, abs(offset2));
                float a = antialiase(d);
                return min(a_rect, a);
            }
            center = vec2(extent.x + offset3.x, extent.y + offset3.y); 
            if (is_right_bottom(pt, extent, center)) {
                float d = sdfEllipse(pt, center, abs(offset3));
                float a = antialiase(d);
                return min(a_rect, a);
            }
            
            center = vec2(-extent.x + offset4.x, extent.y + offset4.y); 
            if (is_bottom_left(pt, extent, center)) {
                float d = sdfEllipse(pt, center, abs(offset4));
                float a = antialiase(d);
                return min(a_rect, a);
            }
            return a_rect;
        }

        float border_radius(vec2 vVertexPosition) {
            vec2 center = clipSdf0.xy;
            vec2 pos = vVertexPosition - center;
            vec4 top = clipSdf2;
            vec4 bottom = clipSdf3;
            // 左上角
            vec2 c1 = vec2(max(0.01, top.y), max(0.01, top.x));
            // 右上角
            vec2 c2 = vec2(-max(0.01, top.z), max(0.01, top.w));
            // 右下角
            vec2 c3 = vec2(-max(0.01, bottom.y), -max(0.01, bottom.x));
            // 左下角
            vec2 c4 = vec2(max(0.01, bottom.z), -max(0.01, bottom.w));
            
            vec2 extent = clipSdf0.zw;
            return antialiase_round_rect(pos, extent, c1, c2, c3, c4);
        }
// Sector
		// 扇形 sdf，负数在里面，正数在外面
		// pt 待求点
		// c 扇形 边缘处 距离 y轴的 夹角 sin, cos
		// r 半径
		// 参考 https://zhuanlan.zhihu.com/p/427587359
		float sdfPie(vec2 p, vec2 sc, float r)
		{
			p.x = abs(p.x);
			float d1 = length(p) - r;
			
			if (sc.x < 0.0001) {
				return abs(sc.y + 1.0) < 0.001 ? d1 : sc.y;
			}

			float m = length(p - sc * clamp(dot(p, sc), 0.0, r) );
			float d2 = m * sign(sc.y * p.x - sc.x * p.y);
			return max(d1, d2);
		}

		// 计算
		float sector(vec2 vVertexPosition) 
		{
			vec2 center = clipSdf0.xy;
			
			vec2 axisSC = clipSdf2.xy;
			vec2 sc = clipSdf2.zw;

			float r = clipSdf0.z;
			vec2 pos = vVertexPosition - center;

			// 逆过来乘，将 扇形 乘回 到 对称轴 为 y轴 处
			// 调整到 PI / 2 = 1.570796325
			// cos(a- pi/2) = sin(a), sin(a - pi/2) = -cos(a)
			// 要乘以 旋转矩阵 的 逆
			pos = vec2(axisSC.x * pos.x - axisSC.y * pos.y, axisSC.y * pos.x + axisSC.x * pos.y);
			float d = sdfPie(pos, sc, r);
			
			return antialiase(d);
		}
// rect
		// 计算alpha
		float rect(vec2 vVertexPosition) {
			vec2 center = clipSdf0.xy;

			vec2 uExtent = clipSdf0.zw;

			vec2 pos = vVertexPosition - center;
			float d = sdfRect(pos, uExtent);

			return antialiase(d);
		}
// Ellipse
		// 计算alpha
		float ellipse(vec2 vVertexPosition) {
			vec4 ellipse = clipSdf0;
			float d = sdfEllipse(vVertexPosition, ellipse.zw, ellipse.xy);
			
			return antialiase(d);
		}
// Circle
		// 返回 coord 到 圆的 最短距离, 负值表示 在里面, 正值表示在外面
		float sdfCircle(vec2 xy, float r) {
			return length(xy) - r;
		}

		// 计算alpha
		float circle(vec2 vVertexPosition) {
			vec2 center = clipSdf0.xy;
			float radius = clipSdf0.z;
			vec2 pos = vVertexPosition - center;
			float d = sdfCircle(pos, radius);
			
			return antialiase(d);
		}

void main() {
    vec4 baseColor  = texture(sampler2D(diffuseTex, sampler_diffuseTex), vUV);
    
    float factor = 1.0;
    if (mode > 3.5) {
        factor = border_radius(vVertexPosition);
    } else if (mode > 2.5 ) {
        factor = sector(vVertexPosition);
    } else if (mode > 1.5) {
        factor = rect(vVertexPosition);
    } else if (mode > 0.5) {
        factor = ellipse(vVertexPosition);
    } else {
        factor = circle(vVertexPosition);
    }

    baseColor *= factor;


    gl_FragColor = baseColor;
}