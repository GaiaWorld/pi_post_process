use std::{sync::Arc, ops::Range};

use pi_assets::{mgr::AssetMgr};
use pi_map::vecmap::VecMap;
use pi_map::smallvecmap::SmallVecMap;

use pi_render::{
    renderer::{
        draw_obj::{DrawObj, DrawBindGroups, DrawBindGroup},
        pipeline::DepthStencilState, sampler::SamplerRes
    },
    rhi::{
        device::RenderDevice, 
        sampler::{SamplerDesc, EAddressMode, EFilterMode, EAnisotropyClamp}, pipeline::RenderPipeline, asset::RenderRes
    },
    asset::{TAssetKeyU64, ASSET_SIZE_FOR_UNKOWN},
    components::view::target_alloc::{SafeAtlasAllocator, TargetType}
};
use pi_share::Share;

use crate::{temprory_render_target::PostprocessTexture, effect::*, material::tools::load_shader};

use super::{base::{TImageEffect, KeyPostprocessPipeline}, SingleImageEffectResource, ImageEffectResource};


pub struct EffectImageMask {}
impl EffectImageMask {
    pub fn ready(
        param: &ImageMask,
        resources: & super::base::SingleImageEffectResource,
        device: &RenderDevice,
        _: &wgpu::Queue,
        delta_time: u64,
        dst_size: (u32, u32),
        geo_matrix: &[f32],
        alpha: f32, depth: f32,
        source: &PostprocessTexture,
        safeatlas: &SafeAtlasAllocator,
        target_type: TargetType,
        pipelines: & Share<AssetMgr<RenderRes<RenderPipeline>>>,
        color_state: wgpu::ColorTargetState,
        depth_stencil: Option<DepthStencilState>,
        force_nearest_filter: bool,
    ) -> Option<DrawObj> {
        if let Some(resource) = resources.get(&String::from(Self::KEY)) {
            let param_buffer = param.buffer(delta_time, geo_matrix, source.get_tilloff(), alpha, depth, device, (source.use_w(), source.use_h()), dst_size);
            let sampler = if force_nearest_filter { &resource.sampler_nearest.0 } else { &resource.sampler.0 };
            let sampler_mask = if param.nearest_filter { &resource.sampler_nearest.0 } else { &resource.sampler.0 };
            let bind_group = device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some(Self::KEY),
                    layout: &resource.bindgroup_layout,
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: &param_buffer, offset: 0, size: None  } )  },
                        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(source.view())  },
                        wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(sampler)  },
                        wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(param.image.view())  },
                        wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::Sampler(sampler_mask)  },
                    ],
                }
            );

            // let target = Self::get_target(target, &source, dst_size, safeatlas, target_type);

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
                instances: Range { start: 0, end: 1 },
                vertex: resources.quad.value_range(),
                indices: None,
            };
            draw.vertices.insert(0, resources.quad.clone());
            Some(draw)
        } else {
            None
        }

    }
}
impl TImageEffect for EffectImageMask {

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
    const KEY: &'static str = "EffectImageMask";

    fn shader(device: &RenderDevice) -> crate::material::tools::Shader {
        load_shader(
            device,
            include_str!("../shaders/image_mask.vert"),
            include_str!("../shaders/image_mask.frag"),
            "image_mask",
            "image_mask"
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

        device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some(Self::KEY),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader.vs_module,
                    entry_point: "main",
                    buffers: &[
                        wgpu::VertexBufferLayout { array_stride: 8, step_mode: wgpu::VertexStepMode::Vertex, attributes: &base_attributes  }
                    ],
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
                        entry_point: "main",
                        targets: &[key_pipeline.color_state()],
                    }
                ),
                multiview: None,
            }
        )
    }
    fn setup(
        device: &RenderDevice,
        resources: &mut SingleImageEffectResource,
        samplers: & Share<AssetMgr<pi_render::renderer::sampler::SamplerRes>>,
    ) {
        let shader = Self::shader(device);
        let bindgroup_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some(Self::KEY),
                entries: &[
                    // Param
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                        count: None,
                    },
                    // Texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
                        count: None,
                    },
                    // Sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // Texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
                        count: None,
                    },
                    // Sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            }
        );

        let sampler_linear = if let Some(sampler) = samplers.get(&Self::SAMPLER_DESC) {
            sampler
        } else {
            samplers.insert(Self::SAMPLER_DESC.clone(), SamplerRes::new(device, &Self::SAMPLER_DESC)).ok().unwrap()
        };
        
        let sampler_nearest = if let Some(sampler) = samplers.get(&ImageEffectResource::NEAREST_FILTER) {
            sampler
        } else {
            samplers.insert(ImageEffectResource::NEAREST_FILTER.clone(), SamplerRes::new(device, &&ImageEffectResource::NEAREST_FILTER)).ok().unwrap()
        };

        resources.regist(String::from(Self::KEY), ImageEffectResource {
            shader,
            sampler: sampler_linear,
            sampler_nearest,
            bindgroup_layout,
        });
    }
}
