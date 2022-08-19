use super::{shader::{EPostprocessShader, Shader}, blend::EBlend, target_format::ETexutureFormat};

pub struct Pipeline {
    pub key: u128,
    pub pipeline: wgpu::RenderPipeline,
    pub uniform_bind_group_layout: wgpu::BindGroupLayout,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub pipeline_layout: wgpu::PipelineLayout,
}

impl Pipeline {
    pub fn new(
        key: u128,
        name: &str,
        vs_state: wgpu::VertexState,
        fs_state: wgpu::FragmentState,
        device: &wgpu::Device,
        uniform_bind_0_visibility: wgpu::ShaderStages,
    ) -> Self {
        let uniform_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    // Vertex Matrix
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            // min_binding_size: wgpu::BufferSize::new(uniform_size)
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Effect Param
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: uniform_bind_0_visibility,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            // min_binding_size: wgpu::BufferSize::new(uniform_size)
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Diffuse Texture Matrix
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            // min_binding_size: wgpu::BufferSize::new(uniform_size)
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
            }
        );

        let texture_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    }
                ],
            }
        );

        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[
                    &uniform_bind_group_layout,
                    &texture_bind_group_layout
                ],
                push_constant_ranges: &[

                ],
            }
        );

        let pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some(name),
                layout: Some(&pipeline_layout),
                vertex: vs_state,
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false
                },
                fragment: Some(fs_state),
                multiview: None,
            }
        );

        Self { key, pipeline, uniform_bind_group_layout, texture_bind_group_layout, pipeline_layout }
    }
}


pub struct UniformBufferInfo {
    pub offset_vertex_matrix: u64,
    pub size_vertex_matrix: u64,
    pub offset_param: u64,
    pub size_param: u64,
    pub offset_diffuse_matrix: u64,
    pub size_diffuse_matrix: u64,
    pub uniform_size: u64,
}