use std::sync::Arc;

use crate::prelude::{ImageEffectUniformBuffer, SingleImageEffectResource};

#[derive(Clone, Copy, Debug)]
/// 径向模糊
pub struct BlurRadial {
    /// 模糊半径 - 像素值
    pub radius: u8,
    /// * 迭代次数 - 越大效果越好, 性能越差, 一般设置 6
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

impl BlurRadial {
    pub fn is_enabled(
        &self
    ) -> bool {
        self.radius > 0 && self.iteration > 0 && ((self.start - self.center_x).abs() < 3. && (self.start - self.center_y).abs() < 3.)
    }
}
pub struct BlurRadialRenderer {
    pub(crate) param: BlurRadial,
    pub(crate) uniform: Arc<ImageEffectUniformBuffer>,
}
impl BlurRadialRenderer {
    pub fn new(param: &BlurRadial, resource: &SingleImageEffectResource) -> Self {
        Self { param: param.clone(), uniform: resource.uniform_buffer() }
    }
}
impl super::TEffectForBuffer for BlurRadialRenderer {
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
