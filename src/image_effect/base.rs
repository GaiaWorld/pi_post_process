use std::sync::Arc;

use crossbeam::queue::SegQueue;
use pi_assets::{mgr::AssetMgr, asset::Handle};
use pi_hash::XHashMap;
use pi_render::{
    renderer::{
        draw_obj::DrawObj, vertices::{RenderVertices, EVerticesBufferUsage}, vertex_buffer::{VertexBufferAllocator, EVertexBufferRange},
        sampler::SamplerRes, pipeline::DepthStencilState, texture::*
    },
    rhi::{
        bind_group_layout::BindGroupLayout, device::RenderDevice, buffer::Buffer,
        sampler::{SamplerDesc, EAddressMode, EFilterMode, EAnisotropyClamp}, pipeline::RenderPipeline, bind_group::BindGroup, RenderQueue
    },
    asset::TAssetKeyU64,
    components::view::target_alloc::{SafeAtlasAllocator, TargetType}
};
use pi_share::Share;
use wgpu::CommandEncoder;

use crate::{material::tools::Shader, temprory_render_target::PostprocessTexture, effect::TEffectForBuffer};

pub struct ImageEffectResource {
    pub shader: Shader,
    pub sampler: Handle<SamplerRes>,
    pub sampler_nearest: Handle<SamplerRes>,
    pub bindgroup_layout: BindGroupLayout,
}
impl ImageEffectResource {
    pub const NEAREST_FILTER: SamplerDesc  = SamplerDesc {
        address_mode_u: EAddressMode::ClampToEdge,
        address_mode_v: EAddressMode::ClampToEdge,
        address_mode_w: EAddressMode::ClampToEdge,
        mag_filter: EFilterMode::Nearest,
        min_filter: EFilterMode::Nearest,
        mipmap_filter: EFilterMode::Nearest,
        compare: None,
        anisotropy_clamp: EAnisotropyClamp::None,
        border_color: None,
    };
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KeyPostprocessPipeline {
    pub key: String,
    pub depth_stencil: Option<DepthStencilState>,
    pub color_state: wgpu::ColorTargetState,
}
impl KeyPostprocessPipeline {
    pub fn depth_stencil(&self) -> Option<wgpu::DepthStencilState> {
        if let Some(val) = &self.depth_stencil {
            Some(val.depth_stencil_state())
        } else {
            None
        }
    }
    pub fn color_state(&self) -> Option<wgpu::ColorTargetState> {
        Some(self.color_state.clone())
    }
}
impl TAssetKeyU64 for KeyPostprocessPipeline {}

pub enum PostProcessDraw {
    Temp((u32, u32, u32, u32), DrawObj, ETextureViewUsage),
    Final(DrawObj),
}
impl PostProcessDraw {
    pub fn draw<'a>(
        &'a self,
        encoder:  Option<&mut CommandEncoder>,
        renderpass: Option<&mut wgpu::RenderPass<'a>>,
    ) {
        match (self, encoder, renderpass) {
            (PostProcessDraw::Temp(viewport, draw, target), Some(encoder), None) => {
                let (x, y, w, h) = *viewport;
                if let Some(pipeline) = &draw.pipeline {
                    let mut renderpass = encoder.begin_render_pass(
                        &wgpu::RenderPassDescriptor {
                            label: None,
                            color_attachments: &[
                                Some(wgpu::RenderPassColorAttachment {
                                    view: target.view(),
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Load,
                                        store: wgpu::StoreOp::Store,
                                    }
                                })
                            ],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        }
                    );
                    // log::warn!("Viewport: {:?}", (x as f32, y as f32, w as f32, h as f32, target.key()));
                    renderpass.set_viewport(x as f32, y as f32, w as f32, h as f32, 0., 1.);
                    renderpass.set_scissor_rect(x, y, w, h);
                    renderpass.set_pipeline(pipeline);
                    draw.bindgroups.set(&mut renderpass);
                    draw.vertices.iter().for_each(|(v, _)| {
                        renderpass.set_vertex_buffer(v.slot, v.slice());
                    });
                    if let Some(indeice) = &draw.indices {
                        renderpass.set_index_buffer(indeice.slice(), indeice.format);
                        renderpass.draw_indexed(indeice.value_range(), 0, draw.instances.clone());
                    } else {
                        renderpass.draw(draw.vertex.clone(), draw.instances.clone());
                    }
                }
            },
            (PostProcessDraw::Final(draw), None, Some(renderpass)) => {
                if let Some(pipeline) = &draw.pipeline {
                    // renderpass.set_viewport(x as f32, y as f32, w as f32, h as f32, 0., 1.);
                    // renderpass.set_scissor_rect(x, y, w, h);
                    renderpass.set_pipeline(pipeline);
                    draw.bindgroups.set(renderpass);
                    draw.vertices.iter().for_each(|(v, _)| {
                        renderpass.set_vertex_buffer(v.slot, v.slice());
                    });
                    if let Some(indeice) = &draw.indices {
                        renderpass.set_index_buffer(indeice.slice(), indeice.format);
                        renderpass.draw_indexed(indeice.value_range(), 0, draw.instances.clone());
                    } else {
                        renderpass.draw(draw.vertex.clone(), draw.instances.clone());
                    }
                }
            },
            _ => {

            }
        }
    }
}

pub struct ImageEffectUniformBuffer {
    pub buffer: Buffer,
    pub pool: Share<SegQueue<Buffer>>,
}
impl ImageEffectUniformBuffer {
    const SIZE: usize = 256;
    pub fn new(device: &RenderDevice, pool: Share<SegQueue<Buffer>>) -> Self {
        let buffer = if let Some(buffer) = pool.pop() {
            buffer
        } else {
            let mut data = Vec::with_capacity(Self::SIZE);
            for _ in 0..Self::SIZE {
                data.push(0.);
            }
            device.create_buffer_with_data(&pi_render::rhi::BufferInitDescriptor {
                label: Some("PostProcessUniform"),
                contents: bytemuck::cast_slice(&data),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            })
        };
        Self { buffer, pool }
    }
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
}
impl Drop for ImageEffectUniformBuffer {
    fn drop(&mut self) {
        self.pool.push(self.buffer.clone());
    }
}

pub struct ImageEffectInstanceBufferOffset(pub(crate) usize, pub(crate) Share<SegQueue<usize>>);
impl Drop for ImageEffectInstanceBufferOffset {
    fn drop(&mut self) {
        self.1.push(self.0)
    }
}

pub struct SingleImageEffectResource {
    // pub(crate) triangle: RenderVertices,
    pub(crate) quad: RenderVertices,
    // pub(crate) triangle_indices: RenderIndices,
    // pub(crate) quad_indices: RenderIndices,
    map: XHashMap<String, Arc<ImageEffectResource>>,
    pub uniforms: Share<SegQueue<Buffer>>,
    pub(crate) device: RenderDevice,
    pub(crate) instancebuffer: Arc<EVertexBufferRange>,
    pub(crate) instancebufferranges: Share<SegQueue<usize>>,
}

impl SingleImageEffectResource {
    pub const MAX_INSTANCE_RANGE_COUNT: usize = 16;
    pub const INSTANCE_RANGE_SIZE: usize = 2048;
    pub fn new(device: &RenderDevice, queue: &RenderQueue, vballocator: &mut VertexBufferAllocator) -> Self {
        // let vertices: [f32; 6] = [-0.5, -0.5, 1.5, -0.5, -0.5, 1.5];
        // let key = KeyVertexBuffer::from("ImageEffectTriangle");
        // let buffer = vballocator.create_not_updatable_buffer(device, queue, bytemuck::cast_slice(&vertices)).unwrap();
        // let triangle = RenderVertices {
        //     slot: 0,
        //     buffer: EVerticesBufferUsage::EVBRange(Arc::new(buffer)),
        //     buffer_range: None,
        //     size_per_value: 8,
        // };
        // let indices: [u16; 4] = [0, 1, 2, 0];
        // let buffer = vballocator.create_not_updatable_buffer(device, queue, bytemuck::cast_slice(&indices)).unwrap();
        // let triangle_indices = RenderIndices {
        //     buffer: EVerticesBufferUsage::EVBRange(Arc::new(buffer)),
        //     buffer_range: None,
        //     format: wgpu::IndexFormat::Uint16,
        // };

        let vertices: [f32; 12] = [-0.5, -0.5, 0.5, -0.5, -0.5, 0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5];
        // let key = KeyVertexBuffer::from("ImageEffectQuad");
        let buffer = vballocator.create_not_updatable_buffer(device, queue, bytemuck::cast_slice(&vertices), None).unwrap();
        let quad = RenderVertices {
            slot: 0,
            buffer: EVerticesBufferUsage::EVBRange(Arc::new(buffer)),
            buffer_range: None,
            size_per_value: 8,
        };
        // let indices: [u16; 6] = [0, 1, 2, 3, 4, 5];
        // let buffer = vballocator.create_not_updatable_buffer(device, queue, bytemuck::cast_slice(&indices)).unwrap();
        // let quad_indices = RenderIndices {
        //     buffer: EVerticesBufferUsage::EVBRange(Arc::new(buffer)),
        //     buffer_range: None,
        //     format: wgpu::IndexFormat::Uint16,
        // };

        let bytes = Self::INSTANCE_RANGE_SIZE * Self::MAX_INSTANCE_RANGE_COUNT;
        let mut data = Vec::with_capacity(bytes);
        for _ in 0..bytes {
            data.push(0);
        }
        let instancebuffer = vballocator.create_not_updatable_buffer(device, queue, &data, None).unwrap();

        let instancebufferranges = Share::new(SegQueue::default());
        for idx in 0..Self::MAX_INSTANCE_RANGE_COUNT {
            instancebufferranges.push(idx * Self::INSTANCE_RANGE_SIZE);
        }

        Self {
            // triangle,
            quad,
            // triangle_indices,
            // quad_indices,
            map: XHashMap::default(),
            uniforms: Share::new(SegQueue::new()),
            device: device.clone(),
            instancebuffer: Arc::new(instancebuffer),
            instancebufferranges
        }
    }
    pub fn uniform_buffer(&self) -> Arc<ImageEffectUniformBuffer> {
        Arc::new(ImageEffectUniformBuffer::new(&self.device, self.uniforms.clone()))
    }
    pub fn instance_range(&self) -> Option<ImageEffectInstanceBufferOffset> {
        if let Some(offset) = self.instancebufferranges.pop() {
            Some(
                ImageEffectInstanceBufferOffset(offset, self.instancebufferranges.clone())
            )
        } else {
            None
        }
    }
    pub fn regist(
        &mut self,
        key: String,
        resource: ImageEffectResource,
    ) {
        self.map.insert(key, Arc::new(resource));
    }
    pub fn get(&self, key: &String) -> Option<Arc<ImageEffectResource>> {
        if let Some(val) = self.map.get(key) {
            Some(val.clone())
        } else{
            None
        }
    }
}

pub trait TImageEffect {
    fn get_target(target: Option<PostprocessTexture>, source: &PostprocessTexture, dst_size: (u32, u32), safeatlas: &SafeAtlasAllocator, target_type: TargetType, target_format: wgpu::TextureFormat, onlyonce: bool) -> PostprocessTexture {
        let mut templist = vec![];
        let target = if let Some(target) = target {
            target
        } else if let Some(temp) = source.get_share_target() {
            templist.push(temp);
            let target = safeatlas.allocate_not_share(dst_size.0, dst_size.1, target_type, templist.iter(), true);
            PostprocessTexture::from_share_target(Share::new(target), target_format)
        } else {
            let target = safeatlas.allocate_not_share(dst_size.0, dst_size.1, target_type, templist.iter(), true);
            PostprocessTexture::from_share_target(Share::new(target), target_format)
        };
        target
    }
    const SAMPLER_DESC: SamplerDesc;
    const KEY: &'static str;
    fn bind_group<P: TEffectForBuffer>(
        device: &RenderDevice,
        queue: &RenderQueue,
        param: &P,
        resource: &ImageEffectResource,
        delta_time: u64,
        dst_size: (u32, u32),
        geo_matrix: &[f32],
        tex_matrix: (f32, f32, f32, f32),
        alpha: f32, depth: f32,
        source: &PostprocessTexture,
        force_nearest_filter: bool,
        src_premultiplied: bool,
        dst_premultiply: bool,
    ) -> BindGroup {
        let param_buffer = param.buffer(delta_time, geo_matrix, tex_matrix, alpha, depth, device, queue, (source.use_w(), source.use_h()), dst_size, src_premultiplied, dst_premultiply);
        let sampler = if force_nearest_filter { &resource.sampler_nearest.0 } else { &resource.sampler.0 };
        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some(Self::KEY),
                layout: &resource.bindgroup_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: &param_buffer, offset: 0, size: None  } )  },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(source.view())  },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(sampler)  },
                ],
            }
        );
        bind_group
    }
    fn shader(device: &RenderDevice) -> Shader;
    fn pipeline(
        device: &RenderDevice,
        shader: &Shader,
        pipeline_layout: &wgpu::PipelineLayout,
        key_pipeline: &KeyPostprocessPipeline,
    ) -> RenderPipeline;
    fn setup(
        device: &RenderDevice,
        resources: &mut SingleImageEffectResource,
        samplers: & Share<AssetMgr<SamplerRes>>,
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
