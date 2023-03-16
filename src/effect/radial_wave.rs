
#[derive(Clone, Copy, Debug)]
/// 径向波纹扭曲
pub struct RadialWave {
    /// 是否应用纵横比 - 应用则为 圆形， 否则随纵横比形变
    pub aspect_ratio: bool,
    /// 扭曲半径起点 - 渲染范围 [-1, 1]
    pub start: f32,
    /// 扭曲半径终点 - 渲染范围 [-1, 1]
    pub end: f32,
    /// 扭曲中心点坐标 x - 渲染范围 [-1, 1]
    pub center_x: f32,
    /// 扭曲中心点坐标 y - 渲染范围 [-1, 1]
    pub center_y: f32,
    /// 波纹周期数
    pub cycle: u8,
    /// 扭曲强度
    pub weight: f32,
}

impl RadialWave {
    pub fn is_enabled(&self) -> bool {
        self.cycle > 0 && self.weight > 0.0 && (
            (self.start - self.center_x).abs() < 3.
            && (self.start - self.center_y).abs() < 3.
        )
    }  
}
impl super::TEffectForBuffer for RadialWave {
    fn buffer(&self, 
        delta_time: u64,
        geo_matrix: &[f32],
        tex_matrix: (f32, f32, f32, f32),
        alpha: f32, depth: f32,
        device: &pi_render::rhi::device::RenderDevice,
        src_size: (u32, u32),
        dst_size: (u32, u32)
    ) -> pi_render::rhi::buffer::Buffer {
        let mut temp = vec![

        ];
        geo_matrix.iter().for_each(|v| { temp.push(*v) });
        temp.push(tex_matrix.0);
        temp.push(tex_matrix.1);
        temp.push(tex_matrix.2);
        temp.push(tex_matrix.3);

        let mut aspect_ratio = 1.0;
        if self.aspect_ratio {
            aspect_ratio = dst_size.1 as f32 / dst_size.0 as f32;
        }
        temp.push(self.center_x);
        temp.push(self.center_y);
        temp.push(aspect_ratio);
        temp.push(self.start);
        
        temp.push(self.end);
        temp.push(self.cycle as f32);
        temp.push(self.weight);
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