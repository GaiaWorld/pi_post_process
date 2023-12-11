use std::sync::Arc;

use crate::prelude::ImageEffectUniformBuffer;

use super::{color_filter::ColorFilter, color_balance::ColorBalance, color_scale::ColorScale, hsb::HSB, vignette::Vignette};


pub struct ColorEffectRenderer {
    pub(crate) hsb: Option<HSB>,
    pub(crate) balance: Option<ColorBalance>,
    pub(crate) vignette: Option<Vignette>,
    pub(crate) scale: Option<ColorScale>,
    pub(crate) filter: Option<ColorFilter>,
    pub(crate) uniform: Arc<ImageEffectUniformBuffer>,
}

impl super::TEffectForBuffer for ColorEffectRenderer {
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

        ColorBalance::collect(self.balance.as_ref(), &mut temp);
        HSB::collect(self.hsb.as_ref(), &mut temp);
        ColorScale::collect(self.scale.as_ref(), &mut temp);
        Vignette::collect(self.vignette.as_ref(), &mut temp);
        ColorFilter::collect(self.filter.as_ref(), &mut temp);

        temp.push(depth);
        temp.push(alpha);
        if src_premultiplied { temp.push(1.); } else { temp.push(0.); }

        if dst_premultiply { temp.push(1.); } else { temp.push(0.); }
        temp.push(0.);
        temp.push(0.);
        temp.push(0.);

        queue.write_buffer(self.uniform.buffer(), 0, bytemuck::cast_slice(&temp));
        self.uniform.buffer()
    }
}

