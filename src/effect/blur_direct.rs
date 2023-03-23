
/// 定向模糊
#[derive(Clone, Copy, Debug)]
pub struct BlurDirect {
    /// 模糊半径 - 像素
    pub radius: u8,
    /// * 迭代次数 - 数值越大效果越好, 性能越差 - 通常 6
    /// * 暂弃用, 渲染使用 8 次迭代
    pub iteration: u8,
    /// 方向 x 轴
    pub direct_x: f32,
    /// 方向 y 轴
    pub direct_y: f32,
}

impl BlurDirect {
    pub fn is_enabled(
        &self
    ) -> bool {
        self.radius > 0 && self.iteration > 0 && (self.direct_x > 0. || self.direct_y > 0.)
    }
}

impl super::TEffectForBuffer for BlurDirect {
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
        
        temp.push(self.direct_x);
        temp.push(self.direct_y);
        temp.push(self.radius as f32 / dst_size.0 as f32);
        temp.push(self.iteration as f32);

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
