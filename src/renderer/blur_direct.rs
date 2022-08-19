use crate::{geometry::{Geometry, vertex_buffer_layout::EVertexBufferLayout}, material::{target_format::{get_target_texture_format, ETexutureFormat}, blend::{get_blend_state, EBlend}, shader::{Shader, EPostprocessShader}, tools::{effect_render, get_uniform_bind_group, VERTEX_MATERIX_SIZE, DIFFUSE_MATERIX_SIZE}, pipeline::{Pipeline, UniformBufferInfo}}, effect::blur_direct::BlurDirect, postprocess_pipeline::PostProcessPipeline, temprory_render_target::{EPostprocessTarget} };

use super::{renderer::{Renderer}};

const UNIFORM_PARAM_SIZE: u64 = 4 * 4;

pub struct BlurDirectRenderer {
    pub direct: Renderer,
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
                write_mask: wgpu::ColorWrites::ALL,
            }
        ],
    };

    Pipeline::new(
        key,
        "BlurDirect",
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
    param: &BlurDirect,
    size: (u32, u32),
) {
    queue.write_buffer(
        &renderer.uniform_buffer, 
        renderer.ubo_info.offset_param, 
        bytemuck::cast_slice(
            &[
                param.direct_x,
                param.direct_y,
                param.radius as f32 / size.0 as f32,
                param.iteration as f32
            ]
        )
    );
}

pub fn blur_direct_render(
    blur_direct: &BlurDirect,
    device: &wgpu::Device,
    queue: & wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    postprocess_pipelines: & PostProcessPipeline,
    renderer: &BlurDirectRenderer,
    image_effect_geo: &Geometry,
    resource: &EPostprocessTarget,
    receiver: &EPostprocessTarget,
    blend: EBlend,
    matrix: &[f32; 16],
) {
    let renderer = &renderer.direct;

    // let image_effect_geo = postprocess_renderer.get_geometry(device);
    let pipeline = postprocess_pipelines.get_pipeline(
        EPostprocessShader::BlurDirect,
        EVertexBufferLayout::Position2D,
        blend,
        receiver.format(),
    );

    queue.write_buffer(&renderer.uniform_buffer, renderer.ubo_info.offset_vertex_matrix, bytemuck::cast_slice(matrix));
    update_uniform(renderer, &queue, blur_direct, (resource.use_w(), resource.use_h()));
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
        Some("BlurDirect")
    );
}