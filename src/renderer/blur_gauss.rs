

use pi_assets::mgr::AssetMgr;
use pi_render::{
    rhi::{
        device::RenderDevice, asset::RenderRes, pipeline::RenderPipeline, RenderQueue
    },
    components::view::target_alloc::{SafeAtlasAllocator, TargetType},
    renderer::pipeline::DepthStencilState
};
use pi_share::Share;

use crate::{effect::*, temprory_render_target::PostprocessTexture, image_effect::*, IDENTITY_MATRIX};


pub fn blur_gauss_render(
    hparam: &BlurGaussRenderer,
    vparam: &BlurGaussRenderer,
    renderdevice: &RenderDevice,
    queue: & RenderQueue,
    _: &[f32],
    safeatlas: &SafeAtlasAllocator,
    source: &PostprocessTexture,
    target: Option<PostprocessTexture>,
    draws: &mut Vec<PostProcessDraw>,
    resources: &SingleImageEffectResource,
    pipelines: &Share<AssetMgr<RenderRes<RenderPipeline>>>,
    color_state: wgpu::ColorTargetState,
    _: Option<DepthStencilState>,
    target_type: TargetType,
    target_format: wgpu::TextureFormat,
    src_premultiplied: bool,
    dst_premultiply: bool,
) -> PostprocessTexture {
    let dst_size = (source.use_w(), source.use_h());
    // let mut drawparam = BlurGaussRenderer { param: param.clone(), ishorizon: true, texwidth: source.width(), texheight: source.height() };
    let draw = EffectBlurGauss::ready(
        &hparam , 
        resources, renderdevice, queue, 0,
        dst_size, &IDENTITY_MATRIX, 
        1., 0., source,
        safeatlas, target_type, pipelines,
        color_state.clone(), None,
        src_premultiplied, false
    ).unwrap();
    let result = EffectBlurDual::get_target(None, &source, dst_size, safeatlas, target_type, target_format); 
    let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
    draws.push(draw);
    
    let dst_size = (result.use_w(), result.use_h());
    let draw = EffectBlurGauss::ready(
        &vparam , 
        resources, renderdevice, queue, 0,
        dst_size, &IDENTITY_MATRIX, 
        1., 0., &result,
        safeatlas, target_type, pipelines,
        color_state.clone(), None,
        false, dst_premultiply
    ).unwrap();
    let result = EffectBlurDual::get_target(target, &result, dst_size, safeatlas, target_type, target_format); 
    let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
    draws.push(draw);

    result
}