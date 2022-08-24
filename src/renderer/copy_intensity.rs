use crate::{effect::{copy::CopyIntensity, alpha::Alpha}, geometry::{Geometry, vertex_buffer_layout::EVertexBufferLayout}, material::{target_format::{get_target_texture_format, ETexutureFormat}, blend::{get_blend_state, EBlend}, shader::{Shader, EPostprocessShader}, tools::{effect_render, get_uniform_bind_group, get_texture_binding_group, VERTEX_MATERIX_SIZE, DIFFUSE_MATERIX_SIZE, SimpleRenderExtendsData}, pipeline::{Pipeline, UniformBufferInfo}}, postprocess_pipeline::PostProcessPipeline, temprory_render_target:: EPostprocessTarget };

use super::{renderer::{Renderer, ERenderParam}};

const UNIFORM_PARAM_SIZE: u64 = 8 * 4;

pub struct CopyIntensityRenderer {
    pub copy: Renderer,
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
        "CopyIntensity",
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
    copyparam: &CopyIntensity,
) {
    // println!("{}, {}, ", copyparam.intensity, copyparam.polygon);
    queue.write_buffer(&renderer.uniform_buffer, renderer.ubo_info.offset_param, bytemuck::cast_slice(&[
        copyparam.intensity,
        copyparam.polygon as f32,
        copyparam.radius,
        copyparam.angle * 3.1415926 / 180.0,
        copyparam.bg_color.0 as f32 / 255.0,
        copyparam.bg_color.1 as f32 / 255.0,
        copyparam.bg_color.2 as f32 / 255.0,
        copyparam.bg_color.3 as f32 / 255.0,
    ]));
}

pub fn copy_intensity_render<'a> (
    copyparam: &CopyIntensity,
    device: & wgpu::Device,
    queue: &  wgpu::Queue,
    renderpass: & mut wgpu::RenderPass<'a>, 
    format: ETexutureFormat,
    postprocess_pipelines: &'a  PostProcessPipeline,
    renderer: &'a CopyIntensityRenderer,
    texture_bind_group: &'a  wgpu::BindGroup,
    image_effect_geo: &'a  Geometry,
    resource:  & EPostprocessTarget,
    blend: EBlend,
    matrix: & [f32],
    extends: SimpleRenderExtendsData,
) {
    let renderer = &renderer.copy;

    // let image_effect_geo = postprocess_renderer.get_geometry(device);
    let pipeline = postprocess_pipelines.get_pipeline(
        EPostprocessShader::CopyIntensity,
        EVertexBufferLayout::Position2D,
        blend,
        format,
    );

    let mut data = matrix.to_vec(); data.push(extends.depth); data.push(extends.alpha); 
    queue.write_buffer(&renderer.uniform_buffer, renderer.ubo_info.offset_vertex_matrix, bytemuck::cast_slice( &data ));
    update_uniform(renderer, &queue, copyparam);
    effect_render(
        device,
        queue,
        renderpass,
        image_effect_geo,
        resource,
        texture_bind_group,
        &pipeline.texture_bind_group_layout,
        &renderer.uniform_buffer,
        renderer.ubo_info.offset_diffuse_matrix,
        &renderer.uniform_bind_group,
        &pipeline.pipeline,
        Some("CopyIntensity")
    );
}