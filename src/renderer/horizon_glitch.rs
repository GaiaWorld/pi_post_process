
use std::sync::Arc;

use pi_assets::mgr::AssetMgr;
use pi_render::{rhi::{device::RenderDevice, asset::RenderRes, pipeline::RenderPipeline, BufferInitDescriptor, buffer::Buffer, RenderQueue,}, components::view::target_alloc::{SafeAtlasAllocator, TargetType}, renderer::{pipeline::DepthStencilState, vertices::{EVerticesBufferUsage, RenderVertices}, vertex_buffer::VertexBufferAllocator}, };
use pi_share::Share;

use crate::{effect::*, temprory_render_target::PostprocessTexture, image_effect::*, IDENTITY_MATRIX};

const UNIFORM_PARAM_SIZE: u64 = 4 * 4;
const MAX_INSTANCE_COUNT: usize = 200;
 
pub fn horizon_glitch_render(
    param: &HorizonGlitch,
    renderdevice: &RenderDevice,
    queue: & RenderQueue,
    vballocator: &mut VertexBufferAllocator,
    matrix: &[f32],
    safeatlas: &mut SafeAtlasAllocator,
    source: PostprocessTexture,
    target: Option<PostprocessTexture>,
    draws: &mut Vec<PostProcessDraw>,
    resources: &SingleImageEffectResource,
    pipelines: &Share<AssetMgr<RenderRes<RenderPipeline>>>,
    color_state: wgpu::ColorTargetState,
    depth_stencil: Option<DepthStencilState>,
    target_type: TargetType,
) -> PostprocessTexture {
    let depth_stencil: Option<wgpu::DepthStencilState> = None;
    
    let copyparam = CopyIntensity::default();
    let (draw, result) = EffectCopy::ready(
        copyparam, resources, renderdevice, queue, 0,
        (source.use_w(), source.use_h()), &IDENTITY_MATRIX, source.get_tilloff(),
        1., 0., source.clone(), target,
        safeatlas, target_type, pipelines,
        color_state.clone(), None
    ).unwrap();
    draws.push(draw);

    let items = param.get_items();
    let count = items.len();

    let mut instance_data: Vec<f32> = Vec::new();

    let mut instance_count = 0u32;
    for i in 0..count {
        let temp = items.get(i).unwrap();

        let mut y = temp.0;
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
        let data = bytemuck::cast_slice(&instance_data);
        let buffer = vballocator.create_not_updatable_buffer(renderdevice, queue, data).unwrap();
        let buffer = EVerticesBufferUsage::EVBRange(Arc::new(buffer));
        let instance = RenderVertices {
            slot: 1,
            buffer,
            buffer_range: None,
            size_per_value: 16,
        };

        let (draw, result) = EffectHorizonGlitch::ready(
            param.clone(), instance, resources, renderdevice, queue, 0,
            (result.use_w(), result.use_h()), &IDENTITY_MATRIX, source.get_tilloff(),
            1., 0., source, Some(result),
            safeatlas, target_type, pipelines,
            color_state.clone(), None
        ).unwrap();
        draws.push(draw);
    
        result
    } else {
        result
    }

}