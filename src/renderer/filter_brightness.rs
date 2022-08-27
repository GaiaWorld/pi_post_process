
use crate::{geometry::{Geometry, vertex_buffer_layout::{EVertexBufferLayout, get_vertex_buffer_layouts}}, material::{blend::{get_blend_state, EBlend}, shader::{Shader, EPostprocessShader}, tools::{effect_render, VERTEX_MATERIX_SIZE, get_uniform_bind_group, DIFFUSE_MATERIX_SIZE, SimpleRenderExtendsData, UniformBufferInfo, TextureScaleOffset}}, effect::filter_brightness::FilterBrightness, postprocess_pipeline::{PostProcessMaterialMgr, PostprocessMaterial, PostprocessPipeline}, temprory_render_target::{ EPostprocessTarget, TemporaryRenderTargets} };

use super::{renderer::{Renderer, ERenderParam}};


const UNIFORM_PARAM_SIZE: u64 = 4 * 4;

pub struct FilterBrightnessRenderer {
    pub filter: Renderer,
}

impl FilterBrightnessRenderer {
    const UNIFORM_BIND_0_VISIBILITY: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT;
    pub fn check_pipeline(
        device: &wgpu::Device,
        material: &mut PostprocessMaterial,
        geometry: & Geometry,
        target: wgpu::ColorTargetState,
        primitive: wgpu::PrimitiveState,
        depth_stencil: Option<wgpu::DepthStencilState>
    ) {
        let vertex_layouts = get_vertex_buffer_layouts(EVertexBufferLayout::Position2D, geometry);

        material.check_pipeline(
            "FilterBrightness", device,
            &vertex_layouts,
            target, 
            wgpu::ShaderStages::FRAGMENT,
            primitive, depth_stencil
        );
    }

    pub fn get_renderer(
        device: &wgpu::Device,
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

        let uniform_bind_group_layout = PostprocessPipeline::uniform_bind_group_layout(
            device, 
            Self::UNIFORM_BIND_0_VISIBILITY,
        );
    
        let (uniform_buffer, uniform_bind_group) = get_uniform_bind_group(
            device,
            &uniform_bind_group_layout,
            &ubo_info
        );
    
        Renderer {
            uniform_buffer,
            uniform_bind_group,
            ubo_info,
        }
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
pub fn filter_brightness_render<'a>(
    filter_brightness: & FilterBrightness,
    device: & wgpu::Device,
    queue: &  wgpu::Queue,
    renderpass: & mut wgpu::RenderPass<'a>, 
    postprocess_pipelines: &'a PostProcessMaterialMgr,
    renderer: &'a FilterBrightnessRenderer,
    geometry: &'a Geometry,
    texture_scale_offset: &TextureScaleOffset,
    texture_bind_group: &'a wgpu::BindGroup,
    target: &wgpu::ColorTargetState,
    depth_stencil: &Option<wgpu::DepthStencilState>,
    matrix: & [f32],
    extends: SimpleRenderExtendsData,
) {
    let renderer = &renderer.filter;

    let primitive: wgpu::PrimitiveState = wgpu::PrimitiveState::default();
    let pipeline = postprocess_pipelines.get_material(EPostprocessShader::FilterBrightness).get_pipeline(target, &primitive, depth_stencil);

    let mut data = matrix.to_vec(); data.push(extends.depth); data.push(extends.alpha); 
    queue.write_buffer(&renderer.uniform_buffer, renderer.ubo_info.offset_vertex_matrix, bytemuck::cast_slice( &data ));
    update_uniform(renderer, &queue, filter_brightness);
    effect_render(
        queue,
        renderpass,
        geometry,
        texture_scale_offset,
        texture_bind_group,
        &renderer.uniform_buffer,
        renderer.ubo_info.offset_diffuse_matrix,
        &renderer.uniform_bind_group,
        &pipeline.pipeline,
    );
}