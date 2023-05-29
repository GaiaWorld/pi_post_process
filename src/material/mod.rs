

// pub mod shader;
pub mod tools;
pub mod blend;
mod fragment_state;
// pub mod depth_and_stencil;
// pub mod pipeline;
// pub mod vertex_state;
// pub mod fragment_state;
// pub mod texture_sampler;

// pub mod material;
// pub mod texture_block;
// pub mod flag_block;
// pub mod color_block;
// pub mod float_block;
// pub mod float2_block;
// pub mod float4_block;
// pub mod int_block;


pub fn create_target(
    format: wgpu::TextureFormat,
    blend: Option<wgpu::BlendState>,
    write_mask: wgpu::ColorWrites,
) -> wgpu::ColorTargetState {
    wgpu::ColorTargetState {
        format,
        blend,
        write_mask,
    }
}

pub const FORMAT: wgpu::TextureFormat =  wgpu::TextureFormat::Rgba8Unorm;

pub fn create_default_target() -> wgpu::ColorTargetState {
    wgpu::ColorTargetState {
        format: FORMAT,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
    }
}