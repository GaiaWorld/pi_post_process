use pi_render::rhi::{device::RenderDevice, buffer::Buffer};


pub mod alpha;
pub mod area_mask;
pub mod blur_bokeh;
pub mod color_balance;
pub mod color_filter;
pub mod color_scale;
pub mod copy;
pub mod bloom_dual;
pub mod blur_dual;
pub mod blur_direct;
pub mod hsb;
pub mod blur_radial;
pub mod filter_sobel;
pub mod filter_brightness;
pub mod radial_wave;
pub mod vignette;
pub mod horizon_glitch;
pub mod color_effect;

pub use alpha::*;
pub use area_mask::*;
pub use blur_bokeh::*;
pub use color_balance::*;
pub use color_filter::*;
pub use color_scale::*;
pub use copy::*;
pub use bloom_dual::*;
pub use blur_dual::*;
pub use blur_direct::*;
pub use hsb::*;
pub use blur_radial::*;
pub use filter_sobel::*;
pub use filter_brightness::*;
pub use radial_wave::*;
pub use vignette::*;
pub use horizon_glitch::*;
pub use color_effect::*;

pub trait TEffectForBuffer {
    fn buffer(
        &self,
        delta_time: u64,
        geo_matrix: &[f32],
        tex_matrix: (f32, f32, f32, f32),
        alpha: f32,
        depth: f32,
        device: &RenderDevice,
        src_size: (u32, u32),
        dst_size: (u32, u32)
    ) -> Buffer;
}