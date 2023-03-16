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
impl super::TEffectForBuffer for FilterSobel {
    fn buffer(&self, 
        delta_time: u64,
        geo_matrix: &[f32],
        tex_matrix: (f32, f32, f32, f32),
        alpha: f32, depth: f32,
        device: &pi_render::rhi::device::RenderDevice,
        src_size: (u32, u32),
        dst_size: (u32, u32),
    ) -> pi_render::rhi::buffer::Buffer {
        let mut temp = vec![

        ];
        geo_matrix.iter().for_each(|v| { temp.push(*v) });
        temp.push(tex_matrix.0);
        temp.push(tex_matrix.1);
        temp.push(tex_matrix.2);
        temp.push(tex_matrix.3);

        temp.push(self.color.0 as f32 / 255.0);
        temp.push(self.color.1 as f32 / 255.0);
        temp.push(self.color.2 as f32 / 255.0);
        temp.push(self.color.3 as f32 / 255.0);

        temp.push(self.bg_color.0 as f32 / 255.0);
        temp.push(self.bg_color.1 as f32 / 255.0);
        temp.push(self.bg_color.2 as f32 / 255.0);
        temp.push(self.bg_color.3 as f32 / 255.0);

        temp.push(self.size as f32 / dst_size.0 as f32);
        temp.push(self.size as f32 / dst_size.1 as f32);
        temp.push(self.clip);
        temp.push(depth);

        temp.push(alpha);
        temp.push(0.);
        temp.push(0.);
        temp.push(0.);


        device.create_buffer_with_data(&pi_render::rhi::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&temp),
            usage: wgpu::BufferUsages::UNIFORM,
        })
    }
}