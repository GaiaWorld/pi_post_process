
use std::ops::Range;

use pi_assets::mgr::AssetMgr;
use pi_render::{
    rhi::{
        device::RenderDevice, asset::RenderRes, pipeline::RenderPipeline, RenderQueue
    },
    components::view::target_alloc::{SafeAtlasAllocator, TargetType},
    renderer::{pipeline::DepthStencilState, vertices::{EVerticesBufferUsage, RenderVertices}}
};
use pi_share::Share;

use crate::{effect::*, temprory_render_target::PostprocessTexture, image_effect::*, IDENTITY_MATRIX};

const MAX_INSTANCE_COUNT: usize = SingleImageEffectResource::INSTANCE_RANGE_SIZE / (4 * 4);

pub fn horizon_glitch_render_calc(
    base: &HorizonGlitch,
    param: &HorizonGlitchRenderer,
    _renderdevice: &RenderDevice,
    queue: & RenderQueue,
    resources: &SingleImageEffectResource,
) -> Option<RenderVertices> {
    let items = base.get_items();
    let count = items.len();

    let mut instance_data: Vec<f32> = Vec::new();

    let mut instance_count = 0u32;
    for i in 0..count {
        let temp = items.get(i).unwrap();

        let y = temp.0;
        let mut h = temp.1;
        
        let mut y0 = y - h / 2.0;
        let mut y1 = y + h / 2.0;

        y0 = y0.min(1.0).max(0.0);
        y1 = y1.min(1.0).max(0.0);

        h = y1 - y0;

        if h > 0. {
            if instance_count < MAX_INSTANCE_COUNT as u32 {
                instance_data.push(y0);
                instance_data.push(h);
                if instance_data.len() % 2 == 0 {
                    instance_data.push(1.0);
                    instance_data.push(1.0);
                } else {
                    instance_data.push(-1.0);
                    instance_data.push(1.0);
                }
    
                instance_count += 1;
            }
        }
        
    }

    if instance_count > 0 {
        if let Some(offset) = &param.instance {
            let data = bytemuck::cast_slice(&instance_data);
            queue.write_buffer(resources.instancebuffer.buffer(), offset.0 as u64, data);
            let buffer = EVerticesBufferUsage::EVBRange(resources.instancebuffer.clone());
            let instance = RenderVertices {
                slot: 1,
                buffer,
                buffer_range: Some(Range { start: offset.0 as u64, end: data.len() as u64 }),
                size_per_value: 16,
            };
    
            Some(instance)
        } else {
            None
        }
    } else {
        None
    }
}
 
pub fn horizon_glitch_render(
    param: &HorizonGlitchRenderer,
    renderdevice: &RenderDevice,
    queue: & RenderQueue,
    instances: Option<RenderVertices>,
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
    let draw = EffectCopy::ready(
        &param.copy, resources, renderdevice, queue, 0,
        dst_size, &IDENTITY_MATRIX, 
        1., 0., source,
        safeatlas, target_type, pipelines,
        color_state.clone(), None, false,
        src_premultiplied, dst_premultiply
    ).unwrap();
    let result = EffectBlurDual::get_target(target, &source, dst_size, safeatlas, target_type, target_format, true); 
    let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
    draws.push(draw);

    if let Some(instances) = instances {
        let dst_size = (result.use_w(), result.use_h());
        let draw = EffectHorizonGlitch::ready(
            param, instances, resources, renderdevice, queue, 0,
            dst_size, &IDENTITY_MATRIX,
            1., 0., source,
            safeatlas, target_type, pipelines,
            color_state.clone(), None,
            src_premultiplied, dst_premultiply
        ).unwrap();
        let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
        draws.push(draw);
        result
    } else {
        result
    }

}