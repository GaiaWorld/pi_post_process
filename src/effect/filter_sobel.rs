use std::sync::Arc;

use crate::prelude::{ImageEffectUniformBuffer, SingleImageEffectResource};

#[derive(Clone, Copy, Debug)]
pub struct FilterSobel {
    /// 检测范围 - 像素数目
    pub size: u8,
    /// 检测阈值
    pub clip: f32,
    /// 检测结果颜色
    pub color: (u8, u8, u8, u8),
    /// 背景色
    pub bg_color: (u8, u8, u8, u8),
}

impl FilterSobel {
    pub fn is_enabled(&self) -> bool {
        self.size > 0 && self.clip > 0.
    }
}

impl Default for FilterSobel {
    fn default() -> Self {
        Self {
            size: 1,
            clip: 0.5,
            color: (255, 255, 255, 255),
            bg_color: (0, 0, 0, 0),
        }
    }
}

pub struct FilterSobelRenderer {
    pub(crate) param: FilterSobel,
    pub(crate) uniform: Arc<ImageEffectUniformBuffer>,
}
impl FilterSobelRenderer {
    pub fn new(param: &FilterSobel, resource: &SingleImageEffectResource) -> Self {
        Self { param: param.clone(), uniform: resource.uniform_buffer() }
    }
}
impl super::TEffectForBuffer for FilterSobelRenderer {
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

        temp.push(self.param.color.0 as f32 / 255.0);
        temp.push(self.param.color.1 as f32 / 255.0);
        temp.push(self.param.color.2 as f32 / 255.0);
        temp.push(self.param.color.3 as f32 / 255.0);

        temp.push(self.param.bg_color.0 as f32 / 255.0);
        temp.push(self.param.bg_color.1 as f32 / 255.0);
        temp.push(self.param.bg_color.2 as f32 / 255.0);
        temp.push(self.param.bg_color.3 as f32 / 255.0);

        temp.push(self.param.size as f32 / dst_size.0 as f32);
        temp.push(self.param.size as f32 / dst_size.1 as f32);
        temp.push(self.param.clip);
        temp.push(depth);

        temp.push(alpha);
        if src_premultiplied { temp.push(1.); } else { temp.push(0.); }
        if dst_premultiply { temp.push(1.); } else { temp.push(0.); }
        temp.push(0.);

        queue.write_buffer(self.uniform.buffer(), 0, bytemuck::cast_slice(&temp));
        self.uniform.buffer()
    }
}