use std::sync::Arc;

use crate::prelude::{ImageEffectUniformBuffer, SingleImageEffectResource};

#[derive(Clone, Copy, Debug)]
pub struct CopyIntensity {
    /// 拷贝时强度因子
    pub intensity: f32,
    /// * 多边形边数
    ///   * 小于2 无效
    ///   * 2 - 圆形
    ///   * 3 - 三角形
    ///   * ...
    ///   * 255 - 255 边形
    pub polygon: u8,
    /// * 多边形外切圆半径
    pub radius: f32,
    /// * 多边形旋转角度
    pub angle: f32,
    /// * 形状之外颜色
    pub bg_color: (u8, u8, u8, u8),
}

impl Default for CopyIntensity {
    fn default() -> Self {
        Self {
            intensity: 1.0,
            polygon: 0,
            radius: 1.0,
            angle: 0.0,
            bg_color: (0, 0, 0, 0),
        }
    }
}

pub struct CopyIntensityRenderer {
    pub(crate) param: CopyIntensity,
    pub(crate) uniform: Arc<ImageEffectUniformBuffer>,
}
impl CopyIntensityRenderer {
    pub fn new(param: &CopyIntensity, resource: &SingleImageEffectResource) -> Self {
        Self { param: param.clone(), uniform: resource.uniform_buffer() }
    }
}
impl super::TEffectForBuffer for CopyIntensityRenderer {
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

        temp.push(self.param.intensity);
        temp.push(self.param.polygon as f32);
        temp.push(self.param.radius);
        temp.push(self.param.angle);
        
        temp.push(self.param.bg_color.0 as f32 / 255.);
        temp.push(self.param.bg_color.1 as f32 / 255.);
        temp.push(self.param.bg_color.2 as f32 / 255.);
        temp.push(self.param.bg_color.3 as f32 / 255.);

        temp.push(depth);
        temp.push(alpha);
        if src_premultiplied { temp.push(1.); } else { temp.push(0.); }
        if dst_premultiply { temp.push(1.); } else { temp.push(0.); }

        queue.write_buffer(self.uniform.buffer(), 0, bytemuck::cast_slice(&temp));
        self.uniform.buffer()
    }
}