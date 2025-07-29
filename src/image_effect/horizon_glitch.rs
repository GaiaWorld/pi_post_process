use std::sync::Arc;

use pi_map::smallvecmap::SmallVecMap;
use pi_assets::mgr::AssetMgr;

use pi_render::{
    renderer::{
        draw_obj::{DrawObj, DrawBindGroups, DrawBindGroup},
        pipeline::DepthStencilState, vertices::RenderVertices
    },
    rhi::{
        device::RenderDevice, 
        sampler::{SamplerDesc, EAddressMode, EFilterMode, EAnisotropyClamp}, pipeline::RenderPipeline, asset::RenderRes
    },
    asset::{TAssetKeyU64, ASSET_SIZE_FOR_UNKOWN},
    components::view::target_alloc::{SafeAtlasAllocator, TargetType}
};
use pi_share::Share;

use crate::{material::{tools::load_shader}, temprory_render_target::PostprocessTexture, effect::*};

use super::base::{TImageEffect, KeyPostprocessPipeline};


pub struct EffectHorizonGlitch {
    // resource: Arc<ImageEffectResource>,
    // bind_group: BindGroup,
    // param_buffer: Buffer,
    // temp_tex: ETextureViewUsage,
    // vertex: RenderVertices,
    // instance: RenderVertices,
    // viewport: (u32, u32, u32, u32),
}
impl EffectHorizonGlitch {
    pub fn ready(
        param: &HorizonGlitchRenderer,
        instance: RenderVertices,
        resources: & super::base::SingleImageEffectResource,
        device: &RenderDevice,
        queue: &pi_render::rhi::RenderQueue,
        delta_time: u64,
        dst_size: (u32, u32),
        geo_matrix: &[f32],
        // tex_matrix: (f32, f32, f32, f32),
        alpha: f32, depth: f32,
        source: &PostprocessTexture,
        // target: Option<PostprocessTexture>,
        _safeatlas: &SafeAtlasAllocator,
        _target_type: TargetType,
        pipelines: & Share<AssetMgr<RenderRes<RenderPipeline>>>,
        color_state: wgpu::ColorTargetState,
        depth_stencil: Option<DepthStencilState>,
        src_premultiplied: bool,
        dst_premultiply: bool,
    ) -> Option<DrawObj> {
        if let Some(resource) = resources.get(&String::from(Self::KEY)) {

            // let target = Self::get_target(target, &source, dst_size, safeatlas, target_type);

            let bind_group = Self::bind_group(device, queue, param, &resource, delta_time, dst_size, geo_matrix, source.get_tilloff(), alpha, depth, source, false, src_premultiplied, dst_premultiply);

            // log::info!(">>>>>>>>>> {:?}: {:?} >> {:?}", Self::KEY, source.get_rect(), target.get_rect());

            let mut bindgroups = DrawBindGroups::default();
            bindgroups.insert_group(0, DrawBindGroup::Arc(Arc::new(bind_group)));

            let key_pipeline = KeyPostprocessPipeline { key: String::from(Self::KEY), depth_stencil, color_state };
            let key_pipeline_u64 = key_pipeline.asset_u64();
            let pipeline = if let Some(pipeline) = pipelines.get(&key_pipeline_u64) {
                pipeline
            } else {
                let pipeline_layout = device.create_pipeline_layout(
                    &wgpu::PipelineLayoutDescriptor {
                        label: Some(Self::KEY),
                        bind_group_layouts: &[&resource.bindgroup_layout.value()],
                        push_constant_ranges: &[],
                    }
                );
                let pipeline = Self::pipeline(device, &resource.shader, &pipeline_layout, &key_pipeline);
                pipelines.insert(key_pipeline_u64, RenderRes::new(pipeline, ASSET_SIZE_FOR_UNKOWN)).unwrap()
            };

            let mut draw = DrawObj {
                pipeline: Some(pipeline),
                bindgroups,
                vertices: SmallVecMap::default(),
                instances: instance.value_range(),
                vertex: resources.quad.value_range(),
                indices: None,
            };
            draw.vertices.insert(0, resources.quad.clone());
            draw.vertices.insert(1, instance);
            Some(draw)
        } else {
            None
        }

    }
}
impl TImageEffect for EffectHorizonGlitch {

    const SAMPLER_DESC: SamplerDesc = SamplerDesc {
        address_mode_u: EAddressMode::ClampToEdge,
        address_mode_v: EAddressMode::ClampToEdge,
        address_mode_w: EAddressMode::ClampToEdge,
        mag_filter: EFilterMode::Linear,
        min_filter: EFilterMode::Linear,
        mipmap_filter: EFilterMode::Nearest,
        compare: None,
        anisotropy_clamp: EAnisotropyClamp::None,
        border_color: None,
    };
    const KEY: &'static str = "EffectHorizonGlitch";

    fn shader(device: &RenderDevice) -> crate::material::tools::Shader {
        load_shader(
            device,
            include_str!("../shaders/horizon_glitch.vert"),
            include_str!("../shaders/horizon_glitch.frag"),
            "horizon_glitch",
            "horizon_glitch"
        )
    }

    fn pipeline(
        device: &RenderDevice,
        shader: &crate::material::tools::Shader,
        pipeline_layout: &wgpu::PipelineLayout,
        key_pipeline: &KeyPostprocessPipeline,
    ) -> RenderPipeline {
        let base_attributes = vec![
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 0, shader_location: 0 },
        ];
        let attributes2 = vec![
            wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x4, offset: 0, shader_location: 1 },
        ];

        device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some(Self::KEY),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader.vs_module,
                    entry_point: Some("main"),
                    buffers: &[
                        wgpu::VertexBufferLayout { array_stride: 8, step_mode: wgpu::VertexStepMode::Vertex, attributes: &base_attributes  },
                        wgpu::VertexBufferLayout { array_stride: 16, step_mode: wgpu::VertexStepMode::Instance, attributes: &attributes2  }
                    ],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    ..Default::default()
                },
                depth_stencil: key_pipeline.depth_stencil(),
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(
                    wgpu::FragmentState {
                        module: &shader.fs_module,
                        entry_point: Some("main"),
                        targets: &[key_pipeline.color_state()],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }
                ),
                multiview: None,
                cache: None,
            }
        )
    }
}
