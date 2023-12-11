use std::sync::Arc;

use crate::prelude::{ImageEffectUniformBuffer, SingleImageEffectResource};

#[derive(Clone, Copy, Debug)]
pub struct FilterBrightness {
    /// 检测阈值
    pub threshold: f32,
    /// 检测阈值的变化曲线参数
    pub threshold_knee: f32,
}

impl FilterBrightness {
    pub fn is_enabled(&self) -> bool {
        true
    }
}

impl Default for FilterBrightness {
    fn default() -> Self {
        Self {
            threshold: 0.5,
            threshold_knee: 1.0,
        }
    }
}
pub struct FilterBrightnessRenderer {
    pub(crate) param: FilterBrightness,
    pub uniform: Arc<ImageEffectUniformBuffer>,
}
impl FilterBrightnessRenderer {
    pub fn new(param: &FilterBrightness, resource: &SingleImageEffectResource) -> Self {
        Self { param: param.clone(), uniform: resource.uniform_buffer() }
    }
}
impl super::TEffectForBuffer for FilterBrightnessRenderer {
    fn buffer(&self, 
        _: u64,
        geo_matrix: &[f32],
        tex_matrix: (f32, f32, f32, f32),
        alpha: f32, depth: f32,
        _device: &pi_render::rhi::device::RenderDevice,
        queue: &pi_render::rhi::RenderQueue,
        _: (u32, u32),
        _: (u32, u32),
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

        let threshold_x = f32::powf(self.param.threshold, 2.2);
        let mut threshold_y = threshold_x * self.param.threshold_knee;
        let threshold_z = 2. * threshold_y;
        let threshold_w = 0.25 / (threshold_y + 0.00001);
        threshold_y -= threshold_x;
    
        temp.push(threshold_x);
        temp.push(threshold_y);
        temp.push(threshold_z);
        temp.push(threshold_w);

        temp.push(depth);
        temp.push(alpha);
        if src_premultiplied { temp.push(1.); } else { temp.push(0.); }
        if dst_premultiply { temp.push(1.); } else { temp.push(0.); }

        queue.write_buffer(self.uniform.buffer(), 0, bytemuck::cast_slice(&temp));
        self.uniform.buffer()
    }
}