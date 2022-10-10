
use pi_render::{rhi::{device::RenderDevice}};
use crate::{geometry::{Geometry, vertex_buffer_layout::EVertexBufferLayout, IDENTITY_MATRIX}, material::{blend::{get_blend_state, EBlend}, shader::{Shader, EPostprocessShader}, tools::{ effect_render, get_texture_binding_group, SimpleRenderExtendsData, TextureScaleOffset}, fragment_state::{create_target, create_default_target}}, temprory_render_target::{get_share_target_view, get_rect_info, TemporaryRenderTargets,  EPostprocessTarget}, effect::{filter_brightness::FilterBrightness, blur_dual::BlurDual, copy::CopyIntensity, bloom_dual::BloomDual, alpha::Alpha}, postprocess_pipeline::PostProcessMaterialMgr, error::EPostprocessError };

use super::{blur_dual::{BlurDualRenderer, calc_blur_dual_render, blur_dual_render_2}, filter_brightness::{filter_brightness_render, FilterBrightnessRenderer}, copy_intensity::{copy_intensity_render, CopyIntensityRenderer}, renderer::{Renderer, ERenderParam}};

const ERROR_NOT_GET_FILTER_BRIGHNESS_USED_RT_ID: &str = "NOT_GET_FILTER_BRIGHNESS_USED_RT_ID";
const ERROR_NOT_GET_RT_BY_FILTER_BRIGHNESS_USED_ID: &str = "NOT_GET_RT_BY_FILTER_BRIGHNESS_USED_ID";

pub struct BloomDualRenderer {
    pub filter_brightness: FilterBrightnessRenderer,
    pub dual: BlurDualRenderer,
    pub copy: CopyIntensityRenderer,
    pub copy_2: CopyIntensityRenderer,
    pub filter_rt_id: Option<usize>,
}

pub fn bloom_dual_render(
    bloom_dual: &BloomDual,
    renderdevice: &RenderDevice,
    queue: & wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    postprocess_pipelines: & PostProcessMaterialMgr,
    renderer: &BloomDualRenderer,
    geometry: &Geometry,
    resource:  (u32, u32, usize, wgpu::TextureFormat),
    receiver:  (u32, u32, usize, wgpu::TextureFormat),
    matrix: &[f32],
    extends: SimpleRenderExtendsData,
    temp_targets: &mut TemporaryRenderTargets,
) -> Result<(), EPostprocessError> {

    let target: wgpu::ColorTargetState = create_default_target();
    let target_for_add: wgpu::ColorTargetState = create_target(wgpu::TextureFormat::Rgba8UnormSrgb, get_blend_state(EBlend::Add), wgpu::ColorWrites::ALL);
    let depth_stencil: Option<wgpu::DepthStencilState> = None;

    let (from_w, from_h, start_id, start_format) = resource;
    let (_, _, final_id, _) = receiver;
    let to_w = from_w / 2;
    let to_h = from_h / 2;
    let filter_rt_id = temp_targets.create_share_target(Some(start_id), to_w, to_h, start_format);

    let blur_dual = BlurDual { radius: bloom_dual.radius, iteration: bloom_dual.iteration, intensity: bloom_dual.intensity, simplified_up: false };
    let temp_ids = calc_blur_dual_render(&blur_dual, (to_w, to_h, filter_rt_id, start_format), (to_w, to_h, filter_rt_id, start_format), temp_targets);

    let primitive: wgpu::PrimitiveState = wgpu::PrimitiveState::default();

    {
        let resource = temp_targets.get_target(start_id).unwrap();
        let receiver = temp_targets.get_target(filter_rt_id).unwrap();
            
        let pipeline = postprocess_pipelines.get_material(EPostprocessShader::FilterBrightness).get_pipeline(&target, &primitive, &depth_stencil);

        let texture_scale_offset: TextureScaleOffset = TextureScaleOffset::from_rect(resource.use_x(), resource.use_y(), resource.use_w(), resource.use_h(), resource.width(), resource.height());
        let texture_bind_group = get_texture_binding_group(&pipeline.texture_bind_group_layout, renderdevice, resource.view());

        let mut renderpass = encoder.begin_render_pass(
            &wgpu::RenderPassDescriptor {
                label: Some("FilterBrightness"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: receiver.view(),
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        }
                    })
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
        
        filter_brightness_render(
            &FilterBrightness { threshold: bloom_dual.threshold, threshold_knee: bloom_dual.threshold_knee },
            renderdevice, queue, &mut renderpass, postprocess_pipelines, &renderer.filter_brightness, geometry, &texture_scale_offset, &texture_bind_group, &target, &depth_stencil, &IDENTITY_MATRIX, SimpleRenderExtendsData::default()
        );
    }

    let blur_dual_result = blur_dual_render_2(
        &blur_dual,
        renderdevice, queue, encoder, postprocess_pipelines, &renderer.dual, geometry, (to_w, to_h, filter_rt_id, start_format), (to_w, to_h, filter_rt_id, start_format), &target_for_add, &target_for_add, &depth_stencil, &IDENTITY_MATRIX,
        temp_targets, &temp_ids, SimpleRenderExtendsData::default()
    );

    let realiteration = temp_ids.len();
    for i in 0..realiteration {
        temp_targets.release(*temp_ids.get(i).unwrap());
    }

    let to = temp_targets.get_target(filter_rt_id).unwrap();
    let resource = temp_targets.get_target(start_id).unwrap();

    if let Ok(_) = blur_dual_result {
        let mut copyparam = CopyIntensity::default();
        
        {
            let receiver = temp_targets.get_target(final_id).unwrap();

            let pipeline = postprocess_pipelines.get_material(EPostprocessShader::CopyIntensity).get_pipeline(&target, &primitive, &depth_stencil);
            let texture_scale_offset: TextureScaleOffset = TextureScaleOffset::from_rect(resource.use_x(), resource.use_y(), resource.use_w(), resource.use_h(), resource.width(), resource.height());
            let texture_bind_group = get_texture_binding_group(&pipeline.texture_bind_group_layout, renderdevice, resource.view());
    
            let pipeline2 = postprocess_pipelines.get_material(EPostprocessShader::CopyIntensity).get_pipeline(&target_for_add, &primitive, &None);
            let texture_scale_offset2: TextureScaleOffset = TextureScaleOffset::from_rect(to.use_x(), to.use_y(), to.use_w(), to.use_h(), to.width(), to.height());
            let texture_bind_group2 = get_texture_binding_group(&pipeline2.texture_bind_group_layout, renderdevice, to.view());
    
            let mut renderpass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("BoomCopyIntensity"),
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment {
                            view: receiver.view(),
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: true,
                            }
                        })
                    ],
                    depth_stencil_attachment: None,
                }
            );
    
            copy_intensity_render(
                &copyparam, renderdevice, queue, &mut renderpass, postprocess_pipelines, &renderer.copy, geometry, &texture_scale_offset, &texture_bind_group, &target, &depth_stencil, matrix, extends
            );
        
            copyparam.intensity = bloom_dual.intensity;
    
            copy_intensity_render(
                &copyparam, renderdevice, queue, &mut renderpass, postprocess_pipelines, &renderer.copy_2, geometry, &texture_scale_offset2, &texture_bind_group2, &target_for_add, &depth_stencil, matrix, extends
            );
        }
        temp_targets.release(filter_rt_id);
        Ok(())
    } else {
        temp_targets.release(filter_rt_id);
        blur_dual_result
    }
}