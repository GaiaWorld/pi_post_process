use std::primitive;

use crate::{geometry::{Geometry, vertex_buffer_layout::{EVertexBufferLayout, get_vertex_buffer_layouts}}, material::{blend::{get_blend_state, EBlend}, shader::{Shader, EPostprocessShader}, tools::{effect_render, get_uniform_bind_group, VERTEX_MATERIX_SIZE, DIFFUSE_MATERIX_SIZE, SimpleRenderExtendsData, UniformBufferInfo, TextureScaleOffset}}, effect::blur_bokeh::BlurBokeh, postprocess_pipeline::{PostProcessMaterialMgr, PostprocessMaterial, PostprocessPipeline}, temprory_render_target::{EPostprocessTarget} };

use super::{renderer::{Renderer, ERenderParam}};

pub struct BlurBokehRenderer {
    pub bokeh: Renderer,
}

impl BlurBokehRenderer {
    const UNIFORM_BIND_0_VISIBILITY: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT;
    pub fn check_pipeline(
        device: &wgpu::Device,
        material: &mut PostprocessMaterial,
        geometry: & Geometry,
        targets: &[wgpu::ColorTargetState],
        primitive: wgpu::PrimitiveState,
        depth_stencil: Option<wgpu::DepthStencilState>
    ) {
        let vertex_layouts = get_vertex_buffer_layouts(EVertexBufferLayout::Position2D, geometry);

        material.check_pipeline(
            "BlurBokeh", device,
            &vertex_layouts,
            targets,
            BlurBokehRenderer::UNIFORM_BIND_0_VISIBILITY,
            primitive, depth_stencil
        );
    }

    pub fn get_renderer(
        device: &wgpu::Device,
    ) -> Renderer {
        let o1 = UniformBufferInfo::calc(device, VERTEX_MATERIX_SIZE);
        let o2 = UniformBufferInfo::calc(device, UNIFORM_PARAM_SIZE);
        let o3 = UniformBufferInfo::calc(device, DIFFUSE_MATERIX_SIZE);
        let ubo_info: UniformBufferInfo = UniformBufferInfo {
            offset_vertex_matrix: 0,
            size_vertex_matrix: VERTEX_MATERIX_SIZE,
            offset_param: 0 + o1,
            size_param: UNIFORM_PARAM_SIZE,
            offset_diffuse_matrix: 0 + o1 + o2,
            size_diffuse_matrix: DIFFUSE_MATERIX_SIZE,
            uniform_size: 0 + o1 + o2 + o3,
        };
    
        let uniform_bind_group_layout = PostprocessPipeline::uniform_bind_group_layout(
            device, 
            BlurBokehRenderer::UNIFORM_BIND_0_VISIBILITY,
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

const UNIFORM_PARAM_SIZE: u64 = 6 * 4;


pub fn update_uniform(
    renderer: &Renderer,
    queue: &wgpu::Queue,
    param: &BlurBokeh,
    size: (u32, u32),
) {
    queue.write_buffer(
        &renderer.uniform_buffer, 
        renderer.ubo_info.offset_param, 
        bytemuck::cast_slice(
            &[
                param.center_x,
                param.center_y,
                param.radius as f32 / size.0 as f32,
                param.iteration as f32,
                param.start,
                param.fade,
            ]
        )
    );
}

pub fn blur_bokeh_render<'a>(
    blur_bokeh: & BlurBokeh,
    device: & wgpu::Device,
    queue: & wgpu::Queue,
    renderpass: & mut wgpu::RenderPass<'a>, 
    postprocess_pipelines: &'a PostProcessMaterialMgr,
    renderer: &'a BlurBokehRenderer,
    geometry: &'a Geometry,
    texture_scale_offset: & TextureScaleOffset,
    texture_bind_group: &'a wgpu::BindGroup,
    target: &wgpu::ColorTargetState,
    depth_stencil: &Option<wgpu::DepthStencilState>,
    matrix: & [f32],
    extends: SimpleRenderExtendsData,
) {
    let renderer = &renderer.bokeh;

    let primitive: wgpu::PrimitiveState = wgpu::PrimitiveState::default();
    let pipeline = postprocess_pipelines.get_material(EPostprocessShader::BlurBokeh).get_pipeline(target, &primitive,depth_stencil);

    let mut data = matrix.to_vec(); data.push(extends.depth); data.push(extends.alpha); 
    queue.write_buffer(&renderer.uniform_buffer, renderer.ubo_info.offset_vertex_matrix, bytemuck::cast_slice( &data ));
    update_uniform(renderer, &queue, blur_bokeh, (texture_scale_offset.use_w, texture_scale_offset.use_h));

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