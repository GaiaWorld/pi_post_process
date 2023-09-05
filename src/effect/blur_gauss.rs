use std::sync::Arc;

use crate::prelude::{ImageEffectUniformBuffer, SingleImageEffectResource};

/// Dual 模糊
#[derive(Clone, Copy, Debug)]
pub struct BlurGauss {
    /// 模糊半径 - 像素
    pub radius: f32,
}

impl Default for BlurGauss {
    fn default() -> Self {
        Self { radius: 0. }
    }
}

impl BlurGauss {
    pub fn is_enabled(
        &self
    ) -> bool {
        self.radius > 0.0
    }
}

#[derive(Clone)]
pub struct BlurGaussRenderer {
    pub(crate) param: BlurGauss,
    pub(crate) ishorizon: bool,
    pub(crate) uniform: Arc<ImageEffectUniformBuffer>,
}
impl BlurGaussRenderer {
    pub fn new(param: &BlurGauss, resource: &SingleImageEffectResource) -> Self {
        Self { param: param.clone(), ishorizon: false, uniform: resource.uniform_buffer() }
    }
}
impl super::TEffectForBuffer for BlurGaussRenderer {
    fn buffer(&self, 
        _: u64,
        geo_matrix: &[f32],
        tex_matrix: (f32, f32, f32, f32),
        alpha: f32, depth: f32,
        device: &pi_render::rhi::device::RenderDevice,
        queue: &pi_render::rhi::RenderQueue,
        src_size: (u32, u32),
        _dst_size: (u32, u32),
        src_premultiplied: bool,
        dst_premultiply: bool,
    ) -> &pi_render::rhi::buffer::Buffer {
        let mut temp = vec![];
        geo_matrix.iter().for_each(|v| { temp.push(*v) });
        temp.push(tex_matrix.0);
        temp.push(tex_matrix.1);
        temp.push(tex_matrix.2);
        temp.push(tex_matrix.3);
        
        temp.push(1.0 / src_size.0 as f32);
        temp.push(1.0 / src_size.1 as f32);
        temp.push(self.param.radius as f32);
        if self.ishorizon { temp.push(1.); } else { temp.push(0.); };

        temp.push(depth);
        temp.push(alpha);
        if src_premultiplied { temp.push(1.); } else { temp.push(0.); }
        if dst_premultiply { temp.push(1.); } else { temp.push(0.); }

        queue.write_buffer(self.uniform.buffer(), 0, bytemuck::cast_slice(&temp));
        self.uniform.buffer()
    }
}
