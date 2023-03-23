use super::{color_filter::ColorFilter, color_balance::ColorBalance, color_scale::ColorScale, hsb::HSB, vignette::Vignette};


pub struct ColorEffect {
    pub(crate) hsb: Option<HSB>,
    pub(crate) balance: Option<ColorBalance>,
    pub(crate) vignette: Option<Vignette>,
    pub(crate) scale: Option<ColorScale>,
    pub(crate) filter: Option<ColorFilter>,
}

impl super::TEffectForBuffer for ColorEffect {
    fn buffer(&self, 
        _: u64,
        geo_matrix: &[f32],
        tex_matrix: (f32, f32, f32, f32),
        alpha: f32, depth: f32,
        device: &pi_render::rhi::device::RenderDevice,
        _: (u32, u32),
        _: (u32, u32)
    ) -> pi_render::rhi::buffer::Buffer {
        let mut temp = vec![

        ];
        geo_matrix.iter().for_each(|v| { temp.push(*v) });
        temp.push(tex_matrix.0);
        temp.push(tex_matrix.1);
        temp.push(tex_matrix.2);
        temp.push(tex_matrix.3);

        ColorBalance::collect(self.balance.as_ref(), &mut temp);
        HSB::collect(self.hsb.as_ref(), &mut temp);
        ColorScale::collect(self.scale.as_ref(), &mut temp);
        Vignette::collect(self.vignette.as_ref(), &mut temp);
        ColorFilter::collect(self.filter.as_ref(), &mut temp);

        temp.push(depth);
        temp.push(alpha);
        // 补 1
        temp.push(0.);

        device.create_buffer_with_data(&pi_render::rhi::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&temp),
            usage: wgpu::BufferUsages::UNIFORM,
        })
    }
}

