use std::sync::Arc;

use crate::prelude::{ImageEffectUniformBuffer, SingleImageEffectResource};

/// Dual 模糊
#[derive(Clone, Copy, Debug)]
pub struct BlurDual {
    /// 模糊半径 - 像素
    pub radius: u8,
    /// 迭代次数
    pub iteration: u8,
    /// Up 时的缩放强度
    pub intensity: f32,
    /// 是否简化 升采样流程
    pub simplified_up: bool,
}

impl Default for BlurDual {
    fn default() -> Self {
        Self { radius: 0, iteration: 0, intensity: 1.0, simplified_up: false }
    }
}

impl BlurDual {
    pub fn is_enabled(
        &self
    ) -> bool {
        self.radius > 0 && self.iteration > 0 && self.intensity > 0.0
    }
}

pub struct BlurDualRendererList {
    pub(crate) iteration: usize,
    pub(crate) downs: Vec<BlurDualRenderer>,
    pub(crate) ups: Vec<BlurDualRenderer>,
}
impl BlurDualRendererList {
    pub const MAX_LEVEL: usize = 4;
    pub fn new(base: &BlurDual, resources: &SingleImageEffectResource) -> Self {
        let blur_dual = BlurDual { radius: base.radius, iteration: base.iteration, intensity: 1., simplified_up: false };
        let blur_dual_up = BlurDual { radius: base.radius, iteration: base.iteration, intensity: base.intensity, simplified_up: true };

        let mut downs = vec![];
        let mut ups = vec![];
        for _ in 0..Self::MAX_LEVEL {
            downs.push(BlurDualRenderer { param: blur_dual.clone(), isup: false, uniform: resources.uniform_buffer() });
            ups.push(BlurDualRenderer { param: blur_dual_up.clone(), isup: true, uniform: resources.uniform_buffer() });
        }

        Self { 
            iteration: Self::MAX_LEVEL.min(base.iteration as usize),
            downs, ups
        }
    }
    pub fn update(&mut self, base: &BlurDual) {
        let blur_dual = BlurDual { radius: base.radius, iteration: base.iteration, intensity: 1., simplified_up: false };
        let blur_dual_up = BlurDual { radius: base.radius, iteration: base.iteration, intensity: base.intensity, simplified_up: true };

        self.iteration = Self::MAX_LEVEL.min(base.iteration as usize);
        self.downs.iter_mut().for_each(|item| {
            item.param = blur_dual.clone();
        });
        self.ups.iter_mut().for_each(|item| {
            item.param = blur_dual_up.clone();
        });
    }
}

#[derive(Clone)]
pub struct BlurDualRenderer {
    pub(crate) param: BlurDual,
    pub(crate) isup: bool,
    pub uniform: Arc<ImageEffectUniformBuffer>,
}
impl super::TEffectForBuffer for BlurDualRenderer {
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

        temp.push(self.param.radius as f32 / dst_size.0 as f32);
        temp.push(self.param.radius as f32 / dst_size.1 as f32);
        temp.push(self.param.intensity);
        if self.isup { temp.push(1.); } else { temp.push(0.); };

        temp.push(depth);
        temp.push(alpha);
        if src_premultiplied { temp.push(1.); } else { temp.push(0.); }
        if dst_premultiply { temp.push(1.); } else { temp.push(0.); }

        queue.write_buffer(self.uniform.buffer(), 0, bytemuck::cast_slice(&temp));
        self.uniform.buffer()
    }
}
