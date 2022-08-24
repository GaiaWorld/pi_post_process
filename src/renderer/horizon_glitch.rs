
use pi_render::{rhi::{device::RenderDevice,}, };
use crate::{geometry::{Geometry, vertex_buffer_layout::EVertexBufferLayout, GlitchInstanceViewer, EGeometryBuffer}, material::{target_format::{get_target_texture_format, ETexutureFormat}, blend::{get_blend_state, EBlend}, shader::{Shader, EPostprocessShader}, tools::{ effect_render, get_texture_binding_group, VERTEX_MATERIX_SIZE, get_uniform_bind_group, DIFFUSE_MATERIX_SIZE, SimpleRenderExtendsData}, pipeline::{Pipeline, UniformBufferInfo}}, effect::{horizon_glitch::HorizonGlitch, copy::CopyIntensity, alpha::Alpha}, postprocess_pipeline::PostProcessPipeline, temprory_render_target:: EPostprocessTarget };

use super::{renderer::{Renderer}, copy_intensity::{copy_intensity_render, CopyIntensityRenderer}};

const UNIFORM_PARAM_SIZE: u64 = 4 * 4;

pub struct HorizonGlitchRenderer {
    pub copy: CopyIntensityRenderer,
    pub glitch: Renderer,
}

pub fn get_pipeline(
    key: u128,
    vertex_layouts: &Vec<wgpu::VertexBufferLayout>,
    device: &wgpu::Device,
    shader: &Shader,
    blend: EBlend,
    format: ETexutureFormat,
) -> Pipeline {

    let vs_state = wgpu::VertexState {
        module: &shader.vs_module,
        entry_point: "main",
        buffers: &vertex_layouts,
    };

    let fs_state = wgpu::FragmentState {
        module: &shader.fs_module,
        entry_point: "main",
        targets: &[
            wgpu::ColorTargetState {
                format: get_target_texture_format(format),
                blend: get_blend_state(blend),
                // blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            }
        ],
    };

    Pipeline::new(
        key,
        "HorizonGlitch",
        vs_state,
        fs_state,
        device,
        wgpu::ShaderStages::FRAGMENT,
    )
}

pub fn get_renderer(
    device: &wgpu::Device,
    pipeline: &Pipeline,
) -> Renderer {
    let ubo_info: UniformBufferInfo = UniformBufferInfo {
        offset_vertex_matrix: 0,
        size_vertex_matrix: VERTEX_MATERIX_SIZE,
        offset_param: device.limits().min_uniform_buffer_offset_alignment as u64,
        size_param: UNIFORM_PARAM_SIZE,
        offset_diffuse_matrix: device.limits().min_uniform_buffer_offset_alignment as u64 * 2,
        size_diffuse_matrix: DIFFUSE_MATERIX_SIZE,
        uniform_size: device.limits().min_uniform_buffer_offset_alignment as u64 * 3,
    };

    let (uniform_buffer, uniform_bind_group) = get_uniform_bind_group(
        device,
        &pipeline.uniform_bind_group_layout,
        &ubo_info
    );

    Renderer {
        pipeline_key: pipeline.key,
        uniform_buffer,
        uniform_bind_group,
        ubo_info,
    }
}

 
pub fn horizon_glitch_render(
    param: &HorizonGlitch,
    renderdevice: &RenderDevice,
    queue: & wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    postprocess_pipelines: & PostProcessPipeline,
    renderer: &HorizonGlitchRenderer,
    image_effect_geo: &Geometry,
    geometry: &Geometry,
    resource:   &EPostprocessTarget,
    receiver:   &EPostprocessTarget,
    blend: EBlend,
    matrix: &[f32],
    extends: SimpleRenderExtendsData,
) {
    let renderer_copy = &renderer.copy;
    let renderer_glitch = &renderer.glitch;

    let copyparam = CopyIntensity::default();
    let device = &renderdevice.wgpu_device();

    let pipeline = postprocess_pipelines.get_pipeline(
        EPostprocessShader::CopyIntensity,
        EVertexBufferLayout::Position2D,
        EBlend::None,
        receiver.format(),
    );
    let texture_bind_group = get_texture_binding_group(&pipeline.texture_bind_group_layout, device, resource.view());

    // let geometry = postprocess_renderer.get_geometry(device);
    let pipeline2 = postprocess_pipelines.get_pipeline(
        EPostprocessShader::HorizonGlitch,
        EVertexBufferLayout::Position2DGlitchInstance,
        EBlend::Premultiply,
        receiver.format(),
    );
    
    let texture_bind_group2 = get_texture_binding_group(&pipeline2.texture_bind_group_layout, device, resource.view());

    let mut renderpass = encoder.begin_render_pass(
        &wgpu::RenderPassDescriptor {
            label: Some("HorizonGlitch"),
            color_attachments: &[
                wgpu::RenderPassColorAttachment {
                    view: receiver.view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    }
                }
            ],
            depth_stencil_attachment: None,
        }
    );
    copy_intensity_render(
        &copyparam, device, queue, &mut renderpass, resource.format(), postprocess_pipelines, renderer_copy, &texture_bind_group, image_effect_geo, resource, blend, matrix, extends
    );

    let items = param.get_items();
    let count = items.len();

    let mut instance_data: Vec<f32> = Vec::new();

    let mut instance_count = 0u32;
    for i in 0..count {
        let temp = items.get(i).unwrap();

        let mut y = temp.0;
        let mut h = temp.1;
        
        let mut y0 = y - h / 2.0;
        let mut y1 = y + h / 2.0;

        y0 = y0.min(1.0).max(0.0);
        y1 = y1.min(1.0).max(0.0);

        h = y1 - y0;

        if h > 0. {
            if instance_count < GlitchInstanceViewer::MAX_INSTANCE_COUNT as u32 {
                instance_data.push(y0);
                instance_data.push(h);
                if instance_data.len() % 2 == 0 {
                    instance_data.push(1.0);
                    instance_data.push(1.0);
                } else {
                    instance_data.push(-1.0);
                    instance_data.push(1.0);
                }
    
                instance_count += 1;
            }
        }
        
    }

    if instance_count > 0 {

        {
            let mut data = matrix.to_vec(); data.push(extends.depth); data.push(extends.alpha); 
            queue.write_buffer(&renderer_glitch.uniform_buffer, renderer_glitch.ubo_info.offset_vertex_matrix, bytemuck::cast_slice( &data ));
            queue.write_buffer(
                &renderer_glitch.uniform_buffer,
                // renderer.ubo_info.offset_param, 
                renderer_glitch.ubo_info.offset_param,
                bytemuck::cast_slice(
                    &[
                        param.strength,
                        param.fade,
                    ]
                )
            );
    
            let us = resource.use_w() as f32 / resource.width () as f32;
            let vs = resource.use_h() as f32 / resource.height() as f32;
            let uo = resource.use_x() as f32 / resource.width () as f32;
            let vo = resource.use_y() as f32 / resource.height() as f32;
            // println!("{:?}", (x, y, w, h));
            queue.write_buffer(
                &renderer_glitch.uniform_buffer,
                // renderer.ubo_info.offset_param + UNIFORM_PARAM_SIZE,
                0 + device.limits().min_uniform_buffer_offset_alignment as u64 * 2,
                bytemuck::cast_slice(&[us, vs, uo, vo])
            );

            queue.write_buffer(
                &geometry.vertex_buffers.get(&(EGeometryBuffer::GlitchInstance as u16)).unwrap(),
                0,
                &bytemuck::cast_slice(&instance_data)
            );
    
    
            // println!("{:?}", to_width);
            renderpass.set_viewport(
                receiver.use_x() as f32,
                receiver.use_y() as f32,
                receiver.use_w() as f32,
                receiver.use_h() as f32,
                0.,
                1.
            );
            // renderpass.set_scissor_rect(
            //     x,
            //     y,
            //     w,
            //     h
            // );
    
            renderpass.set_pipeline(&pipeline2.pipeline);
    
            renderpass.set_bind_group(0, &renderer_glitch.uniform_bind_group, &[]);
            renderpass.set_bind_group(1, &texture_bind_group2, &[]);
            
            renderpass.set_vertex_buffer(
                0, 
                geometry.vertex_buffers.get(&(EGeometryBuffer::Position2D as u16)).unwrap().slice(..)
            );
            renderpass.set_vertex_buffer(
                1, 
                geometry.vertex_buffers.get(&(EGeometryBuffer::GlitchInstance as u16)).unwrap().slice(..),
            );
            renderpass.set_index_buffer(
                geometry.indices_buffers.get(&(EGeometryBuffer::Indices as u16)).unwrap().slice(..),
                wgpu::IndexFormat::Uint16
            );

            let indices_count = *geometry.indices_records.get(&(EGeometryBuffer::Indices as u16)).unwrap();
            renderpass.draw_indexed(0..indices_count, 0, 0..instance_count);
        }
    }
}