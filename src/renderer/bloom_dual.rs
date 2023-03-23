
use pi_assets::mgr::AssetMgr;
use pi_render::{rhi::{device::RenderDevice, asset::RenderRes, pipeline::RenderPipeline}, renderer::pipeline::DepthStencilState, components::view::target_alloc::{SafeAtlasAllocator, TargetType}};
use pi_share::Share;


use crate::{effect::*, temprory_render_target::PostprocessTexture, image_effect::*, IDENTITY_MATRIX, SimpleRenderExtendsData, material::{create_default_target, create_target, blend::{get_blend_state, EBlend}},};

// const ERROR_NOT_GET_FILTER_BRIGHNESS_USED_RT_ID: &str = "NOT_GET_FILTER_BRIGHNESS_USED_RT_ID";
// const ERROR_NOT_GET_RT_BY_FILTER_BRIGHNESS_USED_ID: &str = "NOT_GET_RT_BY_FILTER_BRIGHNESS_USED_ID";

pub fn bloom_dual_render(
    bloom_dual: &BloomDual,
    renderdevice: &RenderDevice,
    queue: & wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    matrix: &[f32],
    _: SimpleRenderExtendsData,
    safeatlas: &SafeAtlasAllocator,
    source: PostprocessTexture,
    draws: &mut Vec<PostProcessDraw>,
    resources: &SingleImageEffectResource,
    pipelines: &Share<AssetMgr<RenderRes<RenderPipeline>>>,
    depth_stencil: Option<DepthStencilState>,
    target_type: TargetType,
) -> PostprocessTexture {

    let color_state: wgpu::ColorTargetState = create_default_target();
    let color_state_for_add: wgpu::ColorTargetState = create_target(wgpu::TextureFormat::Rgba8UnormSrgb, get_blend_state(EBlend::Add), wgpu::ColorWrites::ALL);

    let blur_dual = BlurDual { radius: bloom_dual.radius, iteration: bloom_dual.iteration, intensity: 1., simplified_up: false };

    let from_w = source.use_w();
    let from_h = source.use_h();
    let mut to_w = from_w;
    let mut to_h = from_h;

    let filter = FilterBrightness { threshold: bloom_dual.threshold, threshold_knee: bloom_dual.threshold_knee };
    let (draw, filterresult) = EffectFilterBrightness::ready(
        filter, resources, renderdevice, queue, 0,
        (to_w, to_h), &IDENTITY_MATRIX,
        1., 0., source.clone(), None,
        safeatlas, target_type, pipelines,
        color_state.clone(), None
    ).unwrap();
    draw.draw(Some(encoder), None);

    let mut realiter = 0;
    let mut temptargets = vec![];
    let mut tempsource = filterresult.clone();
    temptargets.push(filterresult);
    for _ in 0..bloom_dual.iteration {
        if to_w / 2 >= 2 && to_h / 2 >= 2 {
            to_w = to_w / 2;
            to_h = to_h / 2;
            realiter += 1;
    
            let (draw, result) = EffectBlurDual::ready(
                BlurDualForBuffer { param: blur_dual.clone(), isup: false }, resources,
                renderdevice, queue,
                0, (to_w, to_h),
                matrix,
                1., 0.,
                tempsource, None, safeatlas, target_type,
                pipelines,
                color_state.clone(),
                None,
            ).unwrap();

            tempsource = result.clone();

            draw.draw(Some(encoder), None);
            temptargets.push(result);
        }
    }

    let blur_dual = BlurDual { radius: bloom_dual.radius, iteration: bloom_dual.iteration, intensity: bloom_dual.intensity, simplified_up: false };
    let mut temptarget = None;
    if realiter > 0 {
        tempsource = temptargets.pop().unwrap();
        for _ in 0..realiter {
            to_w = to_w * 2;
            to_h = to_w * 2;

            temptarget = temptargets.pop();

            let (draw, result) = EffectBlurDual::ready(
                BlurDualForBuffer { param: blur_dual.clone(), isup: true }, resources,
                renderdevice, queue,
                0, (to_w, to_h),
                matrix,
                1., 0.,
                tempsource, temptarget, safeatlas, target_type,
                pipelines,
                color_state_for_add.clone(),
                None,
            ).unwrap();

            tempsource = result;
            draw.draw(Some(encoder), None);
        }
    }

    if realiter == 0 {
        return source;
    } else {
        match &source.view {
            pi_render::renderer::texture::texture_view::ETextureViewUsage::SRT(_) => {
                let mut copyparam = CopyIntensity::default();
                copyparam.intensity = bloom_dual.intensity;
                let (draw, result) = EffectCopy::ready(
                    copyparam.clone(), resources,
                    renderdevice, queue, 0, (source.use_w(), source.use_h()),
                    &IDENTITY_MATRIX,
                    1., 0.,
                    tempsource, Some(source),
                    safeatlas, target_type, pipelines,
                    color_state_for_add.clone(), depth_stencil
                ).unwrap();
                draws.push(draw);
                return result;
            },
            _ => {
                let mut copyparam = CopyIntensity::default();
                copyparam.intensity = 1.0;
                let (draw, result) = EffectCopy::ready(
                    copyparam.clone(), resources,
                    renderdevice, queue, 0, (source.use_w(), source.use_h()),
                    &IDENTITY_MATRIX, 
                    1., 0.,
                    source, None,
                    safeatlas, target_type, pipelines,
                    color_state.clone(), None
                ).unwrap();
                draw.draw(Some(encoder), None);
                
                copyparam.intensity = bloom_dual.intensity;
                let (draw, result) = EffectCopy::ready(
                    copyparam.clone(), resources,
                    renderdevice, queue, 0, (result.use_w(), result.use_h()),
                    &IDENTITY_MATRIX,
                    1., 0.,
                    result, None,
                    safeatlas, target_type, pipelines,
                    color_state_for_add.clone(), depth_stencil
                ).unwrap();
                draws.push(draw);

                return result;
            },
        };

    };
}