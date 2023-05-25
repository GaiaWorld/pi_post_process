use pi_render::rhi::{device::RenderDevice, buffer::Buffer};


mod alpha;
mod area_mask;
mod blur_bokeh;
mod color_balance;
mod color_filter;
mod color_scale;
mod copy;
mod bloom_dual;
mod blur_dual;
mod blur_direct;
mod blur_gauss;
mod hsb;
mod blur_radial;
mod filter_sobel;
mod filter_brightness;
mod radial_wave;
mod vignette;
mod horizon_glitch;
mod color_effect;
mod image_mask;

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
pub use blur_gauss::*;
pub use hsb::*;
pub use blur_radial::*;
pub use filter_sobel::*;
pub use filter_brightness::*;
pub use radial_wave::*;
pub use vignette::*;
pub use horizon_glitch::*;
pub use color_effect::*;
pub use image_mask::*;

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