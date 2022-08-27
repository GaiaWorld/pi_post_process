use crate::{effect::{copy::CopyIntensity}, geometry::{Geometry, vertex_buffer_layout::{EVertexBufferLayout, get_vertex_buffer_layouts}}, material::{blend::{get_blend_state, EBlend}, shader::{Shader, EPostprocessShader}, tools::{effect_render, get_uniform_bind_group, get_texture_binding_group, VERTEX_MATERIX_SIZE, DIFFUSE_MATERIX_SIZE, SimpleRenderExtendsData, UniformBufferInfo, TextureScaleOffset}}, postprocess_pipeline::{PostProcessPipelineMgr, PostprocessMaterail, PostprocessPipeline}, temprory_render_target:: EPostprocessTarget };

use super::{renderer::{Renderer}};

const UNIFORM_PARAM_SIZE: u64 = 8 * 4;

pub struct CopyIntensityRenderer {
    pub copy: Renderer,
}

impl CopyIntensityRenderer {
    const UNIFORM_BIND_0_VISIBILITY: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT;
    pub fn check_pipeline(
        device: &wgpu::Device,
        materail: &mut PostprocessMaterail,
        geometry: &Geometry,
        target: wgpu::ColorTargetState,
        primitive: wgpu::PrimitiveState,
        depth_stencil: Option<wgpu::DepthStencilState>
    ) {
        let vertex_layouts = get_vertex_buffer_layouts(EVertexBufferLayout::Position2D, geometry);

        materail.check_pipeline(
            "CopyInstensity", device,
            &vertex_layouts,
            target,
            Self::UNIFORM_BIND_0_VISIBILITY,
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
    postprocess_pipelines: &'a  PostProcessPipelineMgr,
    renderer: &'a CopyIntensityRenderer,
    geometry: &'a  Geometry,
    texture_scale_offset: &TextureScaleOffset,
    texture_bind_group: &'a  wgpu::BindGroup,
    target: &wgpu::ColorTargetState,
    depth_stencil: &Option<wgpu::DepthStencilState>,
    matrix: & [f32],
    extends: SimpleRenderExtendsData,
) {
    let renderer = &renderer.copy;

    let primitive: wgpu::PrimitiveState = wgpu::PrimitiveState::default();
    let pipeline = postprocess_pipelines.get_material(EPostprocessShader::CopyIntensity).get_pipeline(target, &primitive, depth_stencil);

    let mut data = matrix.to_vec(); data.push(extends.depth); data.push(extends.alpha); 
    queue.write_buffer(&renderer.uniform_buffer, renderer.ubo_info.offset_vertex_matrix, bytemuck::cast_slice( &data ));
    update_uniform(renderer, &queue, copyparam);
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