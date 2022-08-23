
use crate::{geometry::{Geometry, EGeometryBuffer}, temprory_render_target::EPostprocessTarget};

use super::{shader::Shader, pipeline::UniformBufferInfo};

pub const SIMPLE_RENDER_EXTEND_FLOAT_COUNT: u16 = 2;
pub const VERTEX_MATERIX_SIZE: u64 = (16 + (SIMPLE_RENDER_EXTEND_FLOAT_COUNT / 4 + 1) * 4) as u64 * 4;
pub const DIFFUSE_MATERIX_SIZE: u64 = 4 * 4;

#[derive(Debug, Clone, Copy)]
pub struct SimpleRenderExtendsData {
    pub alpha: f32,
    pub depth: f32,
}

impl SimpleRenderExtendsData {
    pub fn default() -> Self {
        Self {
            alpha: 1.0,
            depth: 1.0,
        }
    }
}

pub fn get_texture_binding_group(
    texture_bind_group_layout: &wgpu::BindGroupLayout,
    device: &wgpu::Device,
    textureview: &wgpu::TextureView,
) -> wgpu::BindGroup {
    device.create_bind_group(
        &wgpu::BindGroupDescriptor {
            label: None,
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler (&device.create_sampler(
                        &wgpu::SamplerDescriptor {
                            label: None,
                            address_mode_u: wgpu::AddressMode::MirrorRepeat,
                            address_mode_v: wgpu::AddressMode::MirrorRepeat,
                            address_mode_w: wgpu::AddressMode::MirrorRepeat,
                            mag_filter: wgpu::FilterMode::Linear,
                            min_filter: wgpu::FilterMode::Linear,
                            mipmap_filter: wgpu::FilterMode::Linear,
                            ..Default::default()
                        }
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView (
                        textureview
                    ),
                }
            ],
        }
    )
}

pub fn effect_render<'a>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    renderpass: &mut wgpu::RenderPass<'a>,
    image_effect_geo: &'a Geometry,
    resource: &EPostprocessTarget,
    texture_bind_group: &'a wgpu::BindGroup,
    texture_bind_group_layout: &'a wgpu::BindGroupLayout,
    texture_uniform_buffer: &'a wgpu::Buffer,
    texture_uniform_offset: u64,
    uniform_bind_group: &'a wgpu::BindGroup,
    pipeline: &'a wgpu::RenderPipeline,
    label: Option<&str>,
) {

    let us = resource.use_w() as f32 / resource.width () as f32;
    let vs = resource.use_h() as f32 / resource.height() as f32;
    let uo = resource.use_x() as f32 / resource.width () as f32;
    let vo = resource.use_y() as f32 / resource.height() as f32;
    // println!("{:?}", (x, y, w, h));
    queue.write_buffer(texture_uniform_buffer, texture_uniform_offset, bytemuck::cast_slice(&[us, vs, uo, vo]));

    // renderpass.set_scissor_rect(
    //     x,
    //     y,
    //     w,
    //     h
    // );

    renderpass.set_pipeline(pipeline);

    renderpass.set_bind_group(0, uniform_bind_group, &[]);
    renderpass.set_bind_group(1, texture_bind_group, &[]);

    renderpass.set_vertex_buffer(
        0, 
        image_effect_geo.vertex_buffers.get(&(EGeometryBuffer::Position2D as u16)).unwrap().slice(..)
    );
    renderpass.set_index_buffer(
        image_effect_geo.indices_buffers.get(&(EGeometryBuffer::Indices as u16)).unwrap().slice(..),
        wgpu::IndexFormat::Uint16
    );

    let indices_count = *image_effect_geo.indices_records.get(&(EGeometryBuffer::Indices as u16)).unwrap();
    renderpass.draw_indexed(0..indices_count, 0, 0..1);
}

pub fn get_uniform_bind_group(
    device: &wgpu::Device,
    uniform_bind_group_layout: & wgpu::BindGroupLayout,
    ubo_info: &UniformBufferInfo,
) -> (
    wgpu::Buffer,
    wgpu::BindGroup,
) {

    let uniform_buffer = device.create_buffer(
        &wgpu::BufferDescriptor {
            label: None,
            size: ubo_info.uniform_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }
    );

    let uniform_bind_group = device.create_bind_group(
        &wgpu::BindGroupDescriptor {
            label: None,
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer (
                        wgpu::BufferBinding {
                            buffer: &uniform_buffer,
                            offset: ubo_info.offset_vertex_matrix,
                            size: wgpu::BufferSize::new(ubo_info.size_vertex_matrix)
                        }
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer (
                        wgpu::BufferBinding {
                            buffer: &uniform_buffer,
                            offset: ubo_info.offset_param,
                            // offset: 0 + VERTEX_MATERIX_SIZE,
                            size: wgpu::BufferSize::new(ubo_info.size_param)
                        }
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer (
                        wgpu::BufferBinding {
                            buffer: &uniform_buffer,
                            offset: ubo_info.offset_diffuse_matrix,
                            // offset: 0 + VERTEX_MATERIX_SIZE + uniform_bind_0_size,
                            size: wgpu::BufferSize::new(ubo_info.size_diffuse_matrix)
                        }
                    ),
                }
            ],
        }
    );

    (
        uniform_buffer,
        uniform_bind_group,
    )
}

pub fn blend_one_one() -> wgpu::BlendState {
    wgpu::BlendState {
        color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent::OVER,
    }
}

pub fn blend_one_zero() -> wgpu::BlendState {
    wgpu::BlendState {
        color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::Zero,
            operation: wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent::OVER,
    }
}

pub fn load_shader(
    device: &wgpu::Device,
    vs_text: &str,
    fs_text: &str,
    vs_label: &str,
    fs_label: &str,
) -> Shader {
    let vs_module = device.create_shader_module(
        &wgpu::ShaderModuleDescriptor {
            label: Some(vs_label),
            source: wgpu::ShaderSource::Glsl {
                shader: std::borrow::Cow::Borrowed(vs_text),
                stage: naga::ShaderStage::Vertex,
                defines: naga::FastHashMap::default(),
            }
        }
    );

    let fs_module = device.create_shader_module(
        &wgpu::ShaderModuleDescriptor {
            label: Some(fs_label),
            source: wgpu::ShaderSource::Glsl {
                shader: std::borrow::Cow::Borrowed(fs_text),
                stage: naga::ShaderStage::Fragment,
                defines: naga::FastHashMap::default(),
            }
        }
    );

    Shader {
        vs_module,
        fs_module
    }
}