/// Dual 模糊
#[derive(Clone, Copy, Debug, Default)]
pub struct ClipSdf {
    data: [f32;16],
    mode: f32,
}

impl ClipSdf {
    /// * center 中心点
    /// * width height 矩形宽高
    /// * border_radius_x 左上 右上 右下 左下 的 x 方向半径
    /// * border_radius_y 左上 右上 右下 左下 的 y 方向半径
    pub fn border_radius(
        center: (f32, f32),
        width: f32, height: f32,
        border_radius_x: &[f32; 4],
        border_radius_y: &[f32; 4],
    ) -> Self {
        Self { mode: 4., data: [
            center.0, center.1, width, height,
            width / 2., height / 2., 0., 0.,
            border_radius_y[0], border_radius_x[0], border_radius_x[1], border_radius_y[1],
            border_radius_y[2], border_radius_x[2], border_radius_x[3], border_radius_y[3],
        ] }
    }
    ///
    /// * center 中心点
    /// * radius 半径 
    /// * 中心轴相对 y轴正向 夹角的 sin cos
    /// * 弧度的一半 的 sin cos 
    pub fn sector(center: (f32, f32), radius: f32, central_axis_sincos: (f32, f32), half_radian_sincos: (f32, f32)) -> Self {
        Self { mode: 3., data: [
            center.0, center.1, radius, 0.,
            central_axis_sincos.0, central_axis_sincos.1, half_radian_sincos.0, half_radian_sincos.1,
            0., 0., 0., 0.,
            0., 0., 0., 0.,
        ] }
    }
    ///
    /// * center 中心点
    /// * half_width 矩形宽度的一半 
    /// * half_width 矩形高度的一半 
    pub fn rect(center: (f32, f32), half_width: f32, half_height: f32) -> Self {
        let mut result = Self::default();
        result.mode = 0.;
        result.data[0] = center.0;
        result.data[1] = center.1;
        result.data[2] = half_width;
        result.data[3] = half_height;

        result
    }
    /// * center 中心点
    /// * x_axis_len x 方向半轴长
    /// * y_axis_len y 方向半轴长
    pub fn ellipse(center: (f32, f32), x_axis_len: f32, y_axis_len: f32) -> Self {
        let mut result = Self::default();
        result.mode = 0.;
        result.data[0] = center.0;
        result.data[1] = center.1;
        result.data[2] = x_axis_len;
        result.data[3] = y_axis_len;

        result
    }
    pub fn circle(center: (f32, f32), radius: f32) -> Self {
        let mut result = Self::default();
        result.mode = 0.;
        result.data[0] = center.0;
        result.data[1] = center.1;
        result.data[2] = radius;

        result
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
