use pi_assets::{asset::GarbageEmpty, mgr::AssetMgr};
use pi_render::{rhi::{device::RenderDevice, asset::RenderRes}, components::view::target_alloc::{SafeAtlasAllocator, ShareTargetView}};

use crate::{geometry::{Geometry, vertex_buffer_layout::{EVertexBufferLayout, get_vertex_buffer_layouts}, IDENTITY_MATRIX}, material::{blend::{get_blend_state, EBlend}, shader::{Shader, EPostprocessShader}, tools::{ effect_render, get_uniform_bind_group, VERTEX_MATERIX_SIZE, DIFFUSE_MATERIX_SIZE, get_texture_binding_group, SimpleRenderExtendsData, UniformBufferInfo, TextureScaleOffset}, fragment_state::create_default_target}, effect::blur_dual::BlurDual, temprory_render_target::{get_share_target_view, get_rect_info, TemporaryRenderTargets,  EPostprocessTarget}, postprocess_pipeline::{PostProcessMaterialMgr, PostprocessMaterial, PostprocessPipeline}, error::EPostprocessError };

use super::{renderer::Renderer};

const UNIFORM_PARAM_SIZE: u64 = 4 * 4;
const ERROR_RENDERTARGET_NUMBER_ERROR: &str = "Blur Duar Render: Render Target View Not Enough.";

pub struct BlurDualRenderer {
    pub down: Renderer,
    pub up: Renderer,
    pub up_final: Renderer,
}

impl BlurDualRenderer {
    const UNIFORM_BIND_0_VISIBILITY: wgpu::ShaderStages = wgpu::ShaderStages::VERTEX_FRAGMENT;
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
            "BlurDual", device,
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


pub fn render_down(
    pipeline: &PostprocessPipeline,
    renderer: &Renderer,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    image_effect_geo: &Geometry,
    resource:  &EPostprocessTarget,
    receiver:  &EPostprocessTarget,
) {
    
    let texture_scale_offset: TextureScaleOffset = TextureScaleOffset::from_rect(resource.use_x(), resource.use_y(), resource.use_w(), resource.use_h(), resource.width(), resource.height());
    let texture_bind_group = get_texture_binding_group(&pipeline.texture_bind_group_layout, device, resource.view());
    let mut renderpass = encoder.begin_render_pass(
        &wgpu::RenderPassDescriptor {
            label: Some("ColorEffect"),
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

    // println!("{:?}", to_width);
    renderpass.set_viewport(
        receiver.use_x() as f32,
        receiver.use_y() as f32,
        receiver.use_w() as f32,
        receiver.use_h() as f32,
        0.,
        1.
    );

    effect_render(
        queue,
        &mut renderpass,
        image_effect_geo,
        &texture_scale_offset,
        &texture_bind_group,
        &renderer.uniform_buffer,
        renderer.ubo_info.offset_diffuse_matrix,
        &renderer.uniform_bind_group,
        &pipeline.pipeline,
    );
}

pub fn render_up(
    pipeline: &PostprocessPipeline,
    renderer: &Renderer,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    image_effect_geo: &Geometry,
    resource:  &EPostprocessTarget,
    receiver:  &EPostprocessTarget
) {
    let texture_scale_offset: TextureScaleOffset = TextureScaleOffset::from_rect(resource.use_x(), resource.use_y(), resource.use_w(), resource.use_h(), resource.width(), resource.height());
    let texture_bind_group = get_texture_binding_group(&pipeline.texture_bind_group_layout, device, resource.view());

    let mut renderpass = encoder.begin_render_pass(
        &wgpu::RenderPassDescriptor {
            label: Some("ColorEffect"),
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

    // println!("{:?}", to_width);
    renderpass.set_viewport(
        receiver.use_x() as f32,
        receiver.use_y() as f32,
        receiver.use_w() as f32,
        receiver.use_h() as f32,
        0.,
        1.
    );
    
    effect_render(
        queue,
        &mut renderpass,
        image_effect_geo,
        &texture_scale_offset,
        &texture_bind_group,
        &renderer.uniform_buffer,
        renderer.ubo_info.offset_diffuse_matrix,
        &renderer.uniform_bind_group,
        &pipeline.pipeline,
    );
}


pub fn update_uniform_down(
    renderer: &Renderer,
    queue: &wgpu::Queue,
    dualblur: &BlurDual,
    size: (u32, u32),
) {
    queue.write_buffer(&renderer.uniform_buffer, renderer.ubo_info.offset_param, bytemuck::cast_slice(&[dualblur.radius as f32 / size.0 as f32, dualblur.radius as f32 / size.1 as f32, 1.0, 0.0]));
}

pub fn update_uniform_up(
    renderer: &Renderer,
    queue: &wgpu::Queue,
    dualblur: &BlurDual,
    size: (u32, u32),
) {
    // println!("{}", dualblur.intensity);
    queue.write_buffer(&renderer.uniform_buffer, renderer.ubo_info.offset_param, bytemuck::cast_slice(&[dualblur.radius as f32 / size.0 as f32, dualblur.radius as f32 / size.1 as f32, dualblur.intensity, 1.0]));
}

pub fn calc_blur_dual_render(
    dual_blur: &BlurDual,
    resource:  (u32, u32, usize, wgpu::TextureFormat),
    receiver:  (u32, u32, usize, wgpu::TextureFormat),
    temp_targets: &mut TemporaryRenderTargets,
) -> Vec<usize> {
    let (mut from_w, mut from_h, start_id, start_format) = resource;
    let (mut to_w, mut to_h, _, _) = receiver;

    let mut src_id = start_id;

    let mut temp_rt_ids: Vec<usize> = vec![];
    for _ in 0..dual_blur.iteration {

        to_w = from_w / 2;
        to_h = from_h / 2;

        if to_w > 4 && to_h > 4 {
            let id = temp_targets.create_share_target(Some(src_id), to_w, to_h, start_format);
            temp_rt_ids.push(id);
            src_id = id;
        }

        from_w = to_w;
        from_h = to_h;
    }

    temp_rt_ids
}

pub fn blur_dual_render_2(
    dual_blur: &BlurDual,
    renderdevice: &RenderDevice,
    queue: & wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    postprocess_pipelines: & PostProcessMaterialMgr,
    renderer: &BlurDualRenderer,
    geometry: &Geometry,
    resource:  (u32, u32, usize, wgpu::TextureFormat),
    receiver:  (u32, u32, usize, wgpu::TextureFormat),
    target_for_up: &wgpu::ColorTargetState,
    target: &wgpu::ColorTargetState,
    depth_stencil: &Option<wgpu::DepthStencilState>,
    matrix: &[f32],
    temp_targets: &mut TemporaryRenderTargets,
    temp_rt_ids: &Vec<usize>,
    extends: SimpleRenderExtendsData,
) -> Result<(), EPostprocessError> {
    
    let (mut from_w, mut from_h, start_id, start_format) = resource;
    let (mut to_w, mut to_h, final_id, final_format) = receiver;

    let mut src_id = start_id;
    let mut dst_id = final_id;
    
    let start_resource = temp_targets.get_target(start_id).unwrap();
    let final_receiver = temp_targets.get_target(final_id).unwrap();

    let realiteration = temp_rt_ids.len();

    let primitive: wgpu::PrimitiveState = wgpu::PrimitiveState::default();
    let pipeline = postprocess_pipelines.get_material(EPostprocessShader::BlurDual).get_pipeline(&create_default_target(), &primitive, &None);

    let renderer_down = &renderer.down;
    let renderer_up = &renderer.up;

    from_w = start_resource.use_w();
    from_h = start_resource.use_h();

    let mut data = IDENTITY_MATRIX.to_vec(); data.push(1.0); data.push(1.0); 
    queue.write_buffer(&renderer_down.uniform_buffer, renderer_down.ubo_info.offset_vertex_matrix, bytemuck::cast_slice( &data ));
    let mut data = IDENTITY_MATRIX.to_vec(); data.push(1.0); data.push(1.0); 
    queue.write_buffer(&renderer_up.uniform_buffer, renderer_up.ubo_info.offset_vertex_matrix, bytemuck::cast_slice( &data ));

    // queue.write_buffer(&renderer_down.uniform_buffer, renderer_down.ubo_info.offset_vertex_matrix, bytemuck::cast_slice(&IDENTITY_MATRIX));
    // queue.write_buffer(&renderer_up.uniform_buffer, renderer_up.ubo_info.offset_vertex_matrix, bytemuck::cast_slice(&IDENTITY_MATRIX));

    update_uniform_down(renderer_down, queue, &dual_blur, (from_w, from_h));
    update_uniform_up(renderer_up, queue, &dual_blur, (from_w, from_h));

    src_id = start_id;
    for i in 0..realiteration {
        dst_id = *temp_rt_ids.get(i).unwrap();

        // println!(">{}, {}, {}", from_w, from_h, i);
        render_down(
            pipeline,
            renderer_down,
            renderdevice.wgpu_device(),
            &queue,
            encoder,
            geometry,
            temp_targets.get_target(src_id).unwrap(),
            temp_targets.get_target(dst_id).unwrap()
        );

        src_id = dst_id;
    }

    let mut need_normal_renderup = true;

    if dual_blur.simplified_up {
        let pipeline = postprocess_pipelines.get_material(EPostprocessShader::BlurDual).get_pipeline(target, &primitive, depth_stencil);

        src_id = *temp_rt_ids.get(realiteration - 1).unwrap();
        dst_id = final_id;

        if temp_targets.src_to_dst_isok(Some(src_id), Some(dst_id)) == false {
            if realiteration >= 2 {
                need_normal_renderup = false;

                let src = temp_targets.get_target(src_id).unwrap();

                let mut data = matrix.to_vec(); data.push(extends.depth); data.push(extends.alpha); 
                queue.write_buffer(&renderer.up_final.uniform_buffer, renderer.up_final.ubo_info.offset_vertex_matrix, bytemuck::cast_slice( &data ));

                // queue.write_buffer(&renderer.up_final.uniform_buffer, renderer_up.ubo_info.offset_vertex_matrix, bytemuck::cast_slice(matrix));

                update_uniform_up(&renderer.up_final, queue, &dual_blur, (src.use_w(), src.use_h()));
                render_up(
                    pipeline,
                    renderer_up, 
                    renderdevice.wgpu_device(),
                    &queue,
                    encoder,
                    geometry,
                    temp_targets.get_target(src_id).unwrap(),
                    final_receiver
                );
            }
        }
    }

    if need_normal_renderup {
        let pipeline = postprocess_pipelines.get_material(EPostprocessShader::BlurDual).get_pipeline(target_for_up, &primitive, &None);

        for i in (realiteration-1)..0 {
    
            src_id = *temp_rt_ids.get(i).unwrap();
            dst_id = *temp_rt_ids.get(i - 1).unwrap();
            render_up(
                pipeline,
                renderer_up,
                renderdevice.wgpu_device(),
                &queue,
                encoder,
                geometry,
                temp_targets.get_target(src_id).unwrap(),
                temp_targets.get_target(dst_id).unwrap(),
            );
        }
    
        let pipeline = postprocess_pipelines.get_material(EPostprocessShader::BlurDual).get_pipeline(target, &primitive, depth_stencil);

        src_id = *temp_rt_ids.get(0).unwrap();
        let src = temp_targets.get_target(src_id).unwrap();

        let mut data = matrix.to_vec(); data.push(extends.depth); data.push(extends.alpha); 
        queue.write_buffer(&renderer.up_final.uniform_buffer, renderer_up.ubo_info.offset_vertex_matrix, bytemuck::cast_slice(matrix));
        update_uniform_up(&renderer.up_final, queue, &dual_blur, (src.use_w(), src.use_h()));
        render_up(
            pipeline,
            renderer_up, 
            renderdevice.wgpu_device(),
            &queue,
            encoder,
            geometry,
            src,
            final_receiver
        );
    }

    return Ok(());
}

pub fn blur_dual_render(
    dual_blur: &BlurDual,
    renderdevice: &RenderDevice,
    queue: & wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    postprocess_pipelines: & PostProcessMaterialMgr,
    renderer: &BlurDualRenderer,
    image_effect_geo: &Geometry,
    resource:  (u32, u32, usize, wgpu::TextureFormat),
    receiver:  (u32, u32, usize, wgpu::TextureFormat),
    matrix: &[f32],
    extends: SimpleRenderExtendsData,
    temp_targets: &mut TemporaryRenderTargets,
) -> Result<(), EPostprocessError> {
    
    let target_for_up: wgpu::ColorTargetState = create_default_target();
    let target: wgpu::ColorTargetState = create_default_target();
    let depth_stencil: Option<wgpu::DepthStencilState> = None;

    let (mut from_w, mut from_h, start_id, start_format) = resource;
    let (mut to_w, mut to_h, final_id, final_format) = receiver;

    let mut src_id = start_id;
    let mut dst_id = final_id;

    let mut temp_rt_ids: Vec<usize> = vec![];
    for _ in 0..dual_blur.iteration {

        to_w = from_w / 2;
        to_h = from_h / 2;

        if to_w > 4 && to_h > 4 {
            let id = temp_targets.create_share_target(Some(src_id), to_w, to_h, start_format);
            temp_rt_ids.push(id);
            src_id = id;
        }

        from_w = to_w;
        from_h = to_h;
    }

    let result = blur_dual_render_2(dual_blur, renderdevice, queue, encoder, postprocess_pipelines, renderer, image_effect_geo, resource, receiver, &target_for_up, &target, &depth_stencil, matrix, temp_targets, &temp_rt_ids, extends);
    
    let realiteration = temp_rt_ids.len();
    for i in 0..realiteration {
        temp_targets.release(*temp_rt_ids.get(i).unwrap());
    }

    result
}