
use crate::{geometry::{Geometry, vertex_buffer_layout::EVertexBufferLayout}, material::{target_format::{get_target_texture_format, ETexutureFormat}, blend::{get_blend_state, EBlend}, shader::{Shader, EPostprocessShader}, tools::{effect_render, VERTEX_MATERIX_SIZE, get_uniform_bind_group, DIFFUSE_MATERIX_SIZE}, pipeline::{Pipeline, UniformBufferInfo}}, effect::filter_brightness::FilterBrightness, postprocess_pipeline::PostProcessPipeline, temprory_render_target::{ EPostprocessTarget, TemporaryRenderTargets} };

use super::{renderer::Renderer};


const UNIFORM_PARAM_SIZE: u64 = 4 * 4;

pub struct FilterBrightnessRenderer {
    pub filter: Renderer,
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
        "FilterBrightness",
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


pub fn update_uniform(
    renderer: &Renderer,
    queue: &wgpu::Queue,
    param: &FilterBrightness,
) {
    let threshold_x = f32::powf(param.threshold, 2.2);
    let mut threshold_y = threshold_x * param.threshold_knee;
    let threshold_z = 2. * threshold_y;
    let threshold_w = 0.25 / (threshold_y + 0.00001);
    threshold_y -= threshold_x;

    queue.write_buffer(&renderer.uniform_buffer, renderer.ubo_info.offset_param, bytemuck::cast_slice(&[
        threshold_x,
        threshold_y,
        threshold_z,
        threshold_w
    ]));
}

/// 提取满足辉光亮度的像素
pub fn filter_brightness_render(
    filter_brightness: &FilterBrightness,
    device: &wgpu::Device,
    queue: & wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    postprocess_pipelines: & PostProcessPipeline,
    renderer: &FilterBrightnessRenderer,
    image_effect_geo: &Geometry,
    resource:  (u32, u32, usize, ETexutureFormat),
    receiver:  (u32, u32, usize, ETexutureFormat),
    blend: EBlend,
    matrix: &[f32; 16],
    temp_targets: &mut TemporaryRenderTargets,
) {
    let resource = temp_targets.get_target(resource.2).unwrap();
    let receiver = temp_targets.get_target(receiver.2).unwrap();

    let renderer = &renderer.filter;

    // let image_effect_geo = postprocess_renderer.get_geometry(device);
    let pipeline = postprocess_pipelines.get_pipeline(
        EPostprocessShader::FilterBrightness,
        EVertexBufferLayout::Position2D,
        blend,
        receiver.format(),
    );

    queue.write_buffer(&renderer.uniform_buffer, renderer.ubo_info.offset_vertex_matrix, bytemuck::cast_slice(matrix));
    update_uniform(renderer, &queue, filter_brightness);
    effect_render(
        device,
        queue,
        encoder,
        image_effect_geo,
        resource,
        receiver,
        &pipeline.texture_bind_group_layout,
        &renderer.uniform_buffer,
        renderer.ubo_info.offset_diffuse_matrix,
        &renderer.uniform_bind_group,
        &pipeline.pipeline,
        Some("FilterBrightness")
    );
}