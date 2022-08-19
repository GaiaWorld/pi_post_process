use pi_assets::{asset::GarbageEmpty, mgr::AssetMgr};
use pi_render::{rhi::{device::RenderDevice, asset::RenderRes}, components::view::target_alloc::{SafeAtlasAllocator, ShareTargetView}};

use crate::{geometry::{Geometry, vertex_buffer_layout::EVertexBufferLayout, IDENTITY_MATRIX}, material::{target_format::{get_target_texture_format, ETexutureFormat}, blend::{get_blend_state, EBlend}, shader::{Shader, EPostprocessShader}, tools::{ effect_render, get_uniform_bind_group, VERTEX_MATERIX_SIZE, DIFFUSE_MATERIX_SIZE}, pipeline::{Pipeline, UniformBufferInfo}}, effect::blur_dual::BlurDual, temprory_render_target::{get_share_target_view, get_rect_info, TemporaryRenderTargets,  EPostprocessTarget}, postprocess_pipeline::PostProcessPipeline };

use super::{renderer::Renderer};

const UNIFORM_PARAM_SIZE: u64 = 4 * 4;
const ERROR_RENDERTARGET_NUMBER_ERROR: &str = "Blur Duar Render: Render Target View Not Enough.";

pub struct BlurDualRenderer {
    pub down: Renderer,
    pub up: Renderer,
    pub up_final: Renderer,
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
        "BlurDual",
        vs_state,
        fs_state,
        device,
        wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
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


pub fn render_down(
    pipeline: &Pipeline,
    renderer: &Renderer,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    image_effect_geo: &Geometry,
    resource:  &EPostprocessTarget,
    receiver:  &EPostprocessTarget,
) {
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
        Some("BlurDualDown")
    );
}

pub fn render_up(
    pipeline: &Pipeline,
    renderer: &Renderer,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    image_effect_geo: &Geometry,
    resource:  &EPostprocessTarget,
    receiver:  &EPostprocessTarget
) {
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
        Some("BlurDualUp")
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
    resource:  (u32, u32, usize, ETexutureFormat),
    receiver:  (u32, u32, usize, ETexutureFormat),
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
    postprocess_pipelines: & PostProcessPipeline,
    renderer: &BlurDualRenderer,
    image_effect_geo: &Geometry,
    resource:  (u32, u32, usize, ETexutureFormat),
    receiver:  (u32, u32, usize, ETexutureFormat),
    down_blend: EBlend,
    up_blend: EBlend,
    blend: EBlend,
    matrix: &[f32; 16],
    temp_targets: &mut TemporaryRenderTargets,
    temp_rt_ids: &Vec<usize>,
) -> Result<(), String> {
    
    let (mut from_w, mut from_h, start_id, start_format) = resource;
    let (mut to_w, mut to_h, final_id, final_format) = receiver;

    let mut src_id = start_id;
    let mut dst_id = final_id;
    
    let start_resource = temp_targets.get_target(start_id).unwrap();
    let final_receiver = temp_targets.get_target(final_id).unwrap();

    let realiteration = temp_rt_ids.len();

    let pipeline = postprocess_pipelines.get_pipeline(
        EPostprocessShader::BlurDual,
        EVertexBufferLayout::Position2D,
        down_blend,
        start_resource.format(),
    );

    let renderer_down = &renderer.down;
    let renderer_up = &renderer.up;

    from_w = start_resource.use_w();
    from_h = start_resource.use_h();

    queue.write_buffer(&renderer_down.uniform_buffer, renderer_down.ubo_info.offset_vertex_matrix, bytemuck::cast_slice(&IDENTITY_MATRIX));
    queue.write_buffer(&renderer_up.uniform_buffer, renderer_up.ubo_info.offset_vertex_matrix, bytemuck::cast_slice(&IDENTITY_MATRIX));
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
            image_effect_geo,
            temp_targets.get_target(src_id).unwrap(),
            temp_targets.get_target(dst_id).unwrap()
        );

        src_id = dst_id;
    }

    let mut need_normal_renderup = true;

    if dual_blur.simplified_up {
        let pipeline = postprocess_pipelines.get_pipeline(
            EPostprocessShader::BlurDual,
            EVertexBufferLayout::Position2D,
            blend,
            final_receiver.format(),
        );

        src_id = *temp_rt_ids.get(realiteration - 1).unwrap();
        dst_id = final_id;

        if temp_targets.src_to_dst_isok(Some(src_id), Some(dst_id)) == false {
            if realiteration >= 2 {
                need_normal_renderup = false;

                let src = temp_targets.get_target(src_id).unwrap();
                queue.write_buffer(&renderer.up_final.uniform_buffer, renderer_up.ubo_info.offset_vertex_matrix, bytemuck::cast_slice(matrix));
                update_uniform_up(&renderer.up_final, queue, &dual_blur, (src.use_w(), src.use_h()));
                render_up(
                    pipeline,
                    renderer_up, 
                    renderdevice.wgpu_device(),
                    &queue,
                    encoder,
                    image_effect_geo,
                    temp_targets.get_target(src_id).unwrap(),
                    final_receiver
                );
            }
        }
    }

    if need_normal_renderup {
        let pipeline = postprocess_pipelines.get_pipeline(
            EPostprocessShader::BlurDual,
            EVertexBufferLayout::Position2D,
            up_blend,
            start_resource.format(),
        );

        for i in (realiteration-1)..0 {
    
            src_id = *temp_rt_ids.get(i).unwrap();
            dst_id = *temp_rt_ids.get(i - 1).unwrap();
            render_up(
                pipeline,
                renderer_up,
                renderdevice.wgpu_device(),
                &queue,
                encoder,
                image_effect_geo,
                temp_targets.get_target(src_id).unwrap(),
                temp_targets.get_target(dst_id).unwrap(),
            );
        }
    
        let pipeline = postprocess_pipelines.get_pipeline(
            EPostprocessShader::BlurDual,
            EVertexBufferLayout::Position2D,
            blend,
            final_receiver.format(),
        );

        src_id = *temp_rt_ids.get(0).unwrap();
        let src = temp_targets.get_target(src_id).unwrap();
        queue.write_buffer(&renderer.up_final.uniform_buffer, renderer_up.ubo_info.offset_vertex_matrix, bytemuck::cast_slice(matrix));
        update_uniform_up(&renderer.up_final, queue, &dual_blur, (src.use_w(), src.use_h()));
        render_up(
            pipeline,
            renderer_up, 
            renderdevice.wgpu_device(),
            &queue,
            encoder,
            image_effect_geo,
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
    postprocess_pipelines: & PostProcessPipeline,
    renderer: &BlurDualRenderer,
    image_effect_geo: &Geometry,
    resource:  (u32, u32, usize, ETexutureFormat),
    receiver:  (u32, u32, usize, ETexutureFormat),
    down_blend: EBlend,
    up_blend: EBlend,
    blend: EBlend,
    matrix: &[f32; 16],
    temp_targets: &mut TemporaryRenderTargets,
) -> Result<(), String> {
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

    let result = blur_dual_render_2(dual_blur, renderdevice, queue, encoder, postprocess_pipelines, renderer, image_effect_geo, resource, receiver, down_blend, up_blend, blend, matrix, temp_targets, &temp_rt_ids);
    
    let realiteration = temp_rt_ids.len();
    for i in 0..realiteration {
        temp_targets.release(*temp_rt_ids.get(i).unwrap());
    }

    result
}