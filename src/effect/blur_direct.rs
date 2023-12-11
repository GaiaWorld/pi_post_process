use std::sync::Arc;

use crate::prelude::{ImageEffectUniformBuffer, SingleImageEffectResource};


/// 定向模糊
#[derive(Clone, Copy, Debug)]
pub struct BlurDirect {
    /// 模糊半径 - 像素
    pub radius: u8,
    /// * 迭代次数 - 数值越大效果越好, 性能越差 - 通常 6
    /// * 暂弃用, 渲染使用 8 次迭代
    pub iteration: u8,
    /// 方向 x 轴
    pub direct_x: f32,
    /// 方向 y 轴
    pub direct_y: f32,
}

impl BlurDirect {
    pub fn is_enabled(
        &self
    ) -> bool {
        self.radius > 0 && self.iteration > 0 && (self.direct_x > 0. || self.direct_y > 0.)
    }
}
pub struct BlurDirectRenderer {
    pub(crate) param: BlurDirect,
    pub(crate) uniform: Arc<ImageEffectUniformBuffer>,
}
impl BlurDirectRenderer {
    pub fn new(param: &BlurDirect, resource: &SingleImageEffectResource) -> Self {
        Self { param: param.clone(), uniform: resource.uniform_buffer() }
    }
}

impl super::TEffectForBuffer for BlurDirectRenderer {
    fn buffer(&self, 
        _: u64,
        geo_matrix: &[f32],
        tex_matrix: (f32, f32, f32, f32),
        alpha: f32, depth: f32,
        _device: &pi_render::rhi::device::RenderDevice,
        queue: &pi_render::rhi::RenderQueue,
        _: (u32, u32),
        dst_size: (u32, u32),
        src_premultiplied: bool,
        dst_premultiply: bool,
    ) -> &pi_render::rhi::buffer::Buffer {
        let mut temp = vec![

        ];
        geo_matrix.iter().for_each(|v| { temp.push(*v) });
        temp.push(tex_matrix.0);
        temp.push(tex_matrix.1);
        temp.push(tex_matrix.2);
        temp.push(tex_matrix.3);
        
        temp.push(self.param.direct_x);
        temp.push(self.param.direct_y);
        temp.push(self.param.radius as f32 / dst_size.0 as f32);
        temp.push(self.param.iteration as f32);

        temp.push(depth);
        temp.push(alpha);
        if src_premultiplied { temp.push(1.); } else { temp.push(0.); }
        if dst_premultiply { temp.push(1.); } else { temp.push(0.); }

        queue.write_buffer(self.uniform.buffer(), 0, bytemuck::cast_slice(&temp));
        self.uniform.buffer()

    }
}
