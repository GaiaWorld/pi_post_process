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

#[derive(Clone, Copy, Debug)]
pub struct BlurDualForBuffer {
    pub(crate) param: BlurDual,
    pub(crate) isup: bool,
}
impl super::TEffectForBuffer for BlurDualForBuffer {
    fn buffer(&self, 
        _: u64,
        geo_matrix: &[f32],
        tex_matrix: (f32, f32, f32, f32),
        alpha: f32, depth: f32,
        device: &pi_render::rhi::device::RenderDevice,
        _: (u32, u32),
        dst_size: (u32, u32)
    ) -> pi_render::rhi::buffer::Buffer {
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
        temp.push(0.);
        temp.push(0.);

        device.create_buffer_with_data(&pi_render::rhi::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&temp),
            usage: wgpu::BufferUsages::UNIFORM,
        })
    }
}
