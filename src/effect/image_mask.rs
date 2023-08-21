use crate::prelude::PostprocessTexture;

#[derive(Clone, Copy, Debug)]
pub enum EMaskMode {
    /// 蒙版值小于指定数值时剔除
    Clip,
    /// 蒙版值小于指定数值时剔除, 并且原数据乘上蒙版值
    ClipAndMultiplyAlpha,
}

/// 区域蒙版
#[derive(Clone, Debug)]
pub struct ImageMask {
    pub image: PostprocessTexture,
    pub factor: f32,
    pub mode: EMaskMode,
    pub nearest_filter: bool,
}
impl ImageMask {
    pub fn new(image: PostprocessTexture) -> Self {
        Self {
            image,
            factor: 0.,
            mode: EMaskMode::ClipAndMultiplyAlpha,
            nearest_filter: false,
        }
    }
}

impl super::TEffectForBuffer for ImageMask {
    fn buffer(
        &self, 
        _: u64,
        geo_matrix: &[f32],
        tex_matrix: (f32, f32, f32, f32),
        alpha: f32, depth: f32,
        device: &pi_render::rhi::device::RenderDevice,
        _: (u32, u32),
        _dst_size: (u32, u32),
        src_premultiplied: bool,
        dst_premultiply: bool,
    ) -> pi_render::rhi::buffer::Buffer {
        let mut temp = vec![];
        geo_matrix.iter().for_each(|v| { temp.push(*v) });
        temp.push(tex_matrix.0);
        temp.push(tex_matrix.1);
        temp.push(tex_matrix.2);
        temp.push(tex_matrix.3);

        let mask_matrix = self.image.get_tilloff();
        temp.push(mask_matrix.0);
        temp.push(mask_matrix.1);
        temp.push(mask_matrix.2);
        temp.push(mask_matrix.3);

        temp.push(self.factor);
        match self.mode {
            EMaskMode::Clip => temp.push(0.),
            EMaskMode::ClipAndMultiplyAlpha => temp.push(1.),
        }
        temp.push(depth);
        temp.push(alpha);
        
        if src_premultiplied { temp.push(1.); } else { temp.push(0.); }
        if dst_premultiply { temp.push(1.); } else { temp.push(0.); }
        temp.push(0.);
        temp.push(0.);

        device.create_buffer_with_data(&pi_render::rhi::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&temp),
            usage: wgpu::BufferUsages::UNIFORM,
        })
    }
}
