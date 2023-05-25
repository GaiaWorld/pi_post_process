/// Dual 模糊
#[derive(Clone, Copy, Debug, Default)]
pub struct ClipSdf {
    data: [f32;16],
    mode: f32,
}

impl ClipSdf {
    pub fn border_radius(data: [f32;16]) -> Self {
        Self { mode: 4., data }
    }
    pub fn sector(data: [f32;16]) -> Self {
        Self { mode: 3., data }
    }
    pub fn rect(data: [f32;16]) -> Self {
        Self { mode: 2., data }
    }
    pub fn ellipse(data: [f32;16]) -> Self {
        Self { mode: 1., data }
    }
    pub fn circle(data: [f32;16]) -> Self {
        Self { mode: 0., data }
    }
}

impl ClipSdf {
    pub fn is_enabled(
        &self
    ) -> bool {
        true
    }
}

impl super::TEffectForBuffer for ClipSdf {
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
        
        self.data.iter().for_each(|v| { temp.push(*v) });

        temp.push(self.mode);
        temp.push(depth);
        temp.push(alpha);
        temp.push(0.);

        device.create_buffer_with_data(&pi_render::rhi::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&temp),
            usage: wgpu::BufferUsages::UNIFORM,
        })
    }
}
