use std::{sync::Arc, ops::Range, f32::consts::E};

use pi_assets::{mgr::AssetMgr, asset::Handle};
use pi_atom::Atom;
use pi_hash::XHashMap;
use pi_render::{renderer::{draw_obj::DrawObj, vertices::{RenderVertices, EVerticesBufferUsage, RenderIndices}, vertex_buffer::{KeyVertexBuffer, VertexBufferAllocator}, sampler::SamplerRes, pipeline::DepthStencilState, texture::texture_view::ETextureViewUsage}, rhi::{bind_group_layout::BindGroupLayout, device::RenderDevice, BufferInitDescriptor, buffer::Buffer, asset::RenderRes, texture::Sampler, sampler::SamplerDesc, pipeline::RenderPipeline, bind_group::BindGroup, RenderQueue}, asset::{TAssetKeyU64, ASSET_SIZE_FOR_UNKOWN}, components::view::target_alloc::{SafeAtlasAllocator, TargetType}};
use pi_share::Share;
use wgpu::CommandEncoder;

use crate::{material::{tools::{load_shader, Shader}}, temprory_render_target::PostprocessTexture, effect::TEffectForBuffer};

pub struct ImageEffectResource {
    pub shader: Shader,
    pub sampler: Handle<SamplerRes>,
    pub bindgroup_layout: BindGroupLayout,
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

pub struct PostProcessDraw {
    pub(crate) viewport: (u32, u32, u32, u32),
    pub(crate) draw: DrawObj,
    pub(crate) target: ETextureViewUsage,
}
impl PostProcessDraw {
    pub fn draw<'a>(
        &'a self,
        encoder:  Option<&mut CommandEncoder>,
        renderpass: Option<&mut wgpu::RenderPass<'a>>,
    ) {
        let (x, y, w, h) = self.viewport;
        if let Some(pipeline) = &self.draw.pipeline {
            if let Some(renderpass) = renderpass {
                renderpass.set_viewport(x as f32, y as f32, w as f32, h as f32, 0., 1.);
                renderpass.set_scissor_rect(x, y, w, h);
                renderpass.set_pipeline(pipeline);
                self.draw.bindgroups.set(renderpass);
                self.draw.vertices.iter().for_each(|v| {
                    if let Some(v) = v {
                        renderpass.set_vertex_buffer(v.slot, v.slice());
                    }
                });
                if let Some(indeice) = &self.draw.indices {
                    renderpass.set_index_buffer(indeice.slice(), indeice.format);
                    renderpass.draw_indexed(indeice.value_range(), 0, self.draw.instances.clone());
                } else {
                    renderpass.draw(self.draw.vertex.clone(), self.draw.instances.clone());
                }
            } else if let Some(encoder) = encoder {
                let mut renderpass = encoder.begin_render_pass(
                    &wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[
                            Some(wgpu::RenderPassColorAttachment {
                                view: self.target.view(),
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
                renderpass.set_viewport(x as f32, y as f32, w as f32, h as f32, 0., 1.);
                renderpass.set_scissor_rect(x, y, w, h);
                renderpass.set_pipeline(pipeline);
                self.draw.bindgroups.set(&mut renderpass);
                self.draw.vertices.iter().for_each(|v| {
                    if let Some(v) = v {
                        renderpass.set_vertex_buffer(v.slot, v.slice());
                    }
                });
                if let Some(indeice) = &self.draw.indices {
                    renderpass.set_index_buffer(indeice.slice(), indeice.format);
                    renderpass.draw_indexed(indeice.value_range(), 0, self.draw.instances.clone());
                } else {
                    renderpass.draw(self.draw.vertex.clone(), self.draw.instances.clone());
                }
            };

        }
    }
}

pub struct SingleImageEffectResource {
    pub(crate) triangle: RenderVertices,
    pub(crate) quad: RenderVertices,
    pub(crate) triangle_indices: RenderIndices,
    pub(crate) quad_indices: RenderIndices,
    map: XHashMap<String, Arc<ImageEffectResource>>,
}
impl SingleImageEffectResource {
    pub fn new(device: &RenderDevice, queue: &RenderQueue, vballocator: &mut VertexBufferAllocator) -> Self {
        let vertices: [f32; 6] = [-0.5, -0.5, 1.5, -0.5, -0.5, 1.5];
        let key = KeyVertexBuffer::from("ImageEffectTriangle");
        let buffer = vballocator.create_not_updatable_buffer(device, queue, bytemuck::cast_slice(&vertices)).unwrap();
        let triangle = RenderVertices {
            slot: 0,
            buffer: EVerticesBufferUsage::EVBRange(Arc::new(buffer)),
            buffer_range: None,
            size_per_value: 8,
        };
        let indices: [u16; 4] = [0, 1, 2, 0];
        let buffer = vballocator.create_not_updatable_buffer(device, queue, bytemuck::cast_slice(&indices)).unwrap();
        let triangle_indices = RenderIndices {
            buffer: EVerticesBufferUsage::EVBRange(Arc::new(buffer)),
            buffer_range: None,
            format: wgpu::IndexFormat::Uint16,
        };

        let vertices: [f32; 12] = [-0.5, -0.5, 0.5, -0.5, -0.5, 0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5];
        let key = KeyVertexBuffer::from("ImageEffectQuad");
        let buffer = vballocator.create_not_updatable_buffer(device, queue, bytemuck::cast_slice(&vertices)).unwrap();
        let quad = RenderVertices {
            slot: 0,
            buffer: EVerticesBufferUsage::EVBRange(Arc::new(buffer)),
            buffer_range: None,
            size_per_value: 8,
        };
        let indices: [u16; 6] = [0, 1, 2, 3, 4, 5];
        let buffer = vballocator.create_not_updatable_buffer(device, queue, bytemuck::cast_slice(&indices)).unwrap();
        let quad_indices = RenderIndices {
            buffer: EVerticesBufferUsage::EVBRange(Arc::new(buffer)),
            buffer_range: None,
            format: wgpu::IndexFormat::Uint16,
        };

        Self {
            triangle,
            quad,
            triangle_indices,
            quad_indices,
            map: XHashMap::default(),
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
    fn get_target(target: Option<PostprocessTexture>, source: &PostprocessTexture, dst_size: (u32, u32), safeatlas: &SafeAtlasAllocator, target_type: TargetType) -> PostprocessTexture {
        let mut templist = vec![];
        let target = if let Some(target) = target {
            target
        } else if let Some(temp) = source.get_share_target() {
            templist.push(temp);
            let target = safeatlas.allocate(dst_size.0, dst_size.1, target_type, templist.iter());
            PostprocessTexture::from_share_target(target, source.format())
        } else {
            let target = safeatlas.allocate(dst_size.0, dst_size.1, target_type, templist.iter());
            PostprocessTexture::from_share_target(target, source.format())
        };
        target
    }
    const SAMPLER_DESC: SamplerDesc;
    const KEY: &'static str;
    fn bind_group<P: TEffectForBuffer>(
        device: &RenderDevice,
        param: &P,
        resource: &ImageEffectResource,
        delta_time: u64,
        dst_size: (u32, u32),
        geo_matrix: &[f32],
        tex_matrix: (f32, f32, f32, f32),
        alpha: f32, depth: f32,
        source: &PostprocessTexture,
    ) -> (Buffer, BindGroup) {
        let param_buffer = param.buffer(delta_time, geo_matrix, tex_matrix, alpha, depth, device, (source.use_w(), source.use_h()), dst_size);
        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some(Self::KEY),
                layout: &resource.bindgroup_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: &param_buffer, offset: 0, size: None  } )  },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(source.view())  },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&resource.sampler.0)  },
                ],
            }
        );
        (param_buffer, bind_group)
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

        let sampler = if let Some(sampler) = samplers.get(&Self::SAMPLER_DESC) {
            sampler
        } else {
            samplers.insert(Self::SAMPLER_DESC.clone(), SamplerRes::new(device, &Self::SAMPLER_DESC)).unwrap()
        };

        resources.regist(String::from(Self::KEY), ImageEffectResource {
            shader,
            sampler,
            bindgroup_layout,
        });
    }
}
