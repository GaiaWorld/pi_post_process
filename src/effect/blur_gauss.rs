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

#[derive(Clone, Copy, Debug)]
pub struct BlurGaussForBuffer {
    pub(crate) param: BlurGauss,
    pub(crate) ishorizon: bool,
    pub(crate) texwidth: u32,
    pub(crate) texheight: u32,
}
impl super::TEffectForBuffer for BlurGaussForBuffer {
    fn buffer(&self, 
        _: u64,
        geo_matrix: &[f32],
        tex_matrix: (f32, f32, f32, f32),
        alpha: f32, depth: f32,
        device: &pi_render::rhi::device::RenderDevice,
        _: (u32, u32),
        dst_size: (u32, u32)
    ) -> pi_render::rhi::buffer::Buffer {
        let mut temp = vec![];
        geo_matrix.iter().for_each(|v| { temp.push(*v) });
        temp.push(tex_matrix.0);
        temp.push(tex_matrix.1);
        temp.push(tex_matrix.2);
        temp.push(tex_matrix.3);
        
        temp.push(1.0 / self.texwidth as f32);
        temp.push(1.0 / self.texheight as f32);
        temp.push(self.param.radius as f32);
        if self.ishorizon { temp.push(1.); } else { temp.push(0.); };

        temp.push(depth);
        temp.push(alpha);
        temp.push(0.);
        temp.push(0.);

        device.create_buffer_with_data(&pi_render::rhi::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&temp),
            usage: wgpu::BufferUsages::UNIFORM,
        })
    }
}
