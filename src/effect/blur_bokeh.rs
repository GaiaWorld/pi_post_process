use std::sync::Arc;

use crate::prelude::{ImageEffectUniformBuffer, SingleImageEffectResource};


/// 散景模糊
#[derive(Clone, Copy, Debug)]
pub struct BlurBokeh {
    /// 散景模糊半径
    pub radius: f32,
    /// 散景模糊迭代次数 - 值越大效果越好,性能越差 通常 6
    /// * 暂弃用, 渲染使用 8 次迭代
    pub iteration: u8,
    /// 径向中心点坐标 x - 渲染范围 [-1, 1]
    pub center_x: f32,
    /// 径向中心点坐标 y - 渲染范围 [-1, 1]
    pub center_y: f32,
    /// 沿半径起点
    pub start: f32,
    /// 从起点开始模糊半径的变化范围(从0增加到radius)
    pub fade: f32,
}

impl BlurBokeh {
    pub fn is_enabled(
        &self
    ) -> bool {
        self.radius > 0. && self.iteration > 0
    }
}

pub struct BlurBokehRenderer {
    pub(crate) param: BlurBokeh,
    pub(crate) uniform: Arc<ImageEffectUniformBuffer>,
}
impl BlurBokehRenderer {
    pub fn new(param: &BlurBokeh, resource: &SingleImageEffectResource) -> Self {
        Self { param: param.clone(), uniform: resource.uniform_buffer() }
    }
}

impl super::TEffectForBuffer for BlurBokehRenderer {
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
        
        temp.push(self.param.center_x);
        temp.push(self.param.center_y);
        temp.push(self.param.radius as f32 / dst_size.0 as f32);
        temp.push(self.param.iteration as f32);

        temp.push(self.param.start);
        temp.push(self.param.fade);
        temp.push(depth);
        temp.push(alpha);

        if src_premultiplied { temp.push(1.); } else { temp.push(0.); }
        if dst_premultiply { temp.push(1.); } else { temp.push(0.); }
        temp.push(0.);
        temp.push(0.);

        queue.write_buffer(self.uniform.buffer(), 0, bytemuck::cast_slice(&temp));
        self.uniform.buffer()
    }
}

impl Default for BlurBokeh {
    fn default() -> Self {
        Self { radius: 1., iteration: 8, center_x: 0., center_y: 0., start: 0.25, fade: 0.25 }
    }
}