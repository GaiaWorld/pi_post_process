use pi_hash::XHashMap;

use crate::{material::{shader::{Shader, EPostprocessShader, get_shader}, blend::{EBlend, MOVE_E_BLEND}, pipeline::{gen_pipeline_key, PipelineKeyCalcolator}, fragment_state::gen_fragment_state_key}, geometry::{vertex_buffer_layout::{EVertexBufferLayout, MOVE_E_VERTEX_BUFFER_LAYOUT, get_vertex_buffer_layouts}, Geometry}, renderer::{renderer::{self, Renderer}, copy_intensity::CopyIntensityRenderer, color_effect::ColorEffectRenderer, blur_dual::BlurDualRenderer, blur_bokeh::BlurBokehRenderer, blur_radial::BlurRadialRenderer, blur_direct::BlurDirectRenderer, horizon_glitch::HorizonGlitchRenderer, filter_brightness::FilterBrightnessRenderer, filter_sobel::FilterSobelRenderer, radial_wave::RadialWaveRenderer}};


pub struct PostprocessPipeline {
    pub key: u128,
    pub pipeline: wgpu::RenderPipeline,
    pub uniform_bind_group_layout: wgpu::BindGroupLayout,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub pipeline_layout: wgpu::PipelineLayout,
}

impl PostprocessPipeline {
    pub fn primitive() -> wgpu::PrimitiveState {
        wgpu::PrimitiveState::default()
    }
    pub fn uniform_bind_group_layout(
        device: &wgpu::Device,
        uniform_bind_0_visibility: wgpu::ShaderStages,
    ) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(
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
        )
    }

    pub fn texture_bind_group_layout(
        device: &wgpu::Device,
    ) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(
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
        )
    }
    
    pub fn pipeline_layout(
        device: &wgpu::Device,
        uniform_bind_group_layout: &wgpu::BindGroupLayout,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::PipelineLayout {
        device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[
                    uniform_bind_group_layout,
                    texture_bind_group_layout
                ],
                push_constant_ranges: &[],
            }
        )
    }

    pub fn new(
        key: u128,
        name: &str,
        vs_state: wgpu::VertexState,
        fs_state: wgpu::FragmentState,
        device: &wgpu::Device,
        uniform_bind_0_visibility: wgpu::ShaderStages,
        primitive: wgpu::PrimitiveState,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> Self {
        let uniform_bind_group_layout: wgpu::BindGroupLayout = PostprocessPipeline::uniform_bind_group_layout(device, uniform_bind_0_visibility);
        let texture_bind_group_layout: wgpu::BindGroupLayout = PostprocessPipeline::texture_bind_group_layout(device);
        let pipeline_layout: wgpu::PipelineLayout = PostprocessPipeline::pipeline_layout(device, &uniform_bind_group_layout, &texture_bind_group_layout);

        let pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some(name),
                layout: Some(&pipeline_layout),
                vertex: vs_state,
                fragment: Some(fs_state),
                primitive,
                depth_stencil,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false
                },
                multiview: None,
            }
        );

        Self { key, pipeline, uniform_bind_group_layout, texture_bind_group_layout, pipeline_layout }
    }
}


/// 后处理材质, 此处每个 Shader 只对应一种 VertexState, 一种 uniform_bind_group_layout 和 一种 texture_bind_group_layout
pub struct PostprocessMaterail {
    pub shader: Shader,
    pub pipelines: XHashMap<u128, PostprocessPipeline>,
}

impl PostprocessMaterail {
    pub fn new(shader: Shader) -> Self {
        Self { shader, pipelines: XHashMap::default() }
    }

    pub fn check_pipeline(
        &mut self,
        name: &str,
        device: &wgpu::Device,
        vertex_layouts: &Vec<wgpu::VertexBufferLayout>,
        target: wgpu::ColorTargetState,
        uniform_bind_0_visibility: wgpu::ShaderStages,
        primitive: wgpu::PrimitiveState,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) {

        let mut calcolator = PipelineKeyCalcolator::new();

        gen_pipeline_key(&mut calcolator, &primitive, &depth_stencil, 0, 1);
        gen_fragment_state_key(&mut calcolator, &target);

        let key = calcolator.key;

        match self.pipelines.contains_key(&key) {
            true => {},
            false => {
                let targets = [target];
                let fs_state = Renderer::fs_state(&self.shader, &targets);
                let vs_state = Renderer::vs_state(&self.shader, vertex_layouts);
                let pipeline = PostprocessPipeline::new(
                    key,
                    name,
                    vs_state,
                    fs_state,
                    device,
                    uniform_bind_0_visibility,
                    primitive,
                    depth_stencil,
                );
                self.pipelines.insert(key, pipeline);
            },
        }
    }

    pub fn get_pipeline(
        &self,
        target: &wgpu::ColorTargetState,
        primitive: &wgpu::PrimitiveState,
        depth_stencil: &Option<wgpu::DepthStencilState>,
    ) -> &PostprocessPipeline {
        let mut calcolator = PipelineKeyCalcolator::new();

        gen_pipeline_key(&mut calcolator, &primitive, &depth_stencil, 0, 1);
        gen_fragment_state_key(&mut calcolator, target);

        let key = calcolator.key;

        self.pipelines.get(&key).unwrap()
    }

    pub fn get_pipeline_by_key(
        &self,
        key: u128,
    ) -> Option<&PostprocessPipeline> {
        self.pipelines.get(&key)
    }
}

pub struct PostProcessPipelineMgr {
    materails: XHashMap<u8, PostprocessMaterail>,
}

impl PostProcessPipelineMgr {
    pub fn new() -> Self {
        Self {
            materails: XHashMap::default(),
        }
    }

    pub fn check_pipeline(
        &mut self,
        device: &wgpu::Device,
        geometry: &Geometry,
        shader_key: EPostprocessShader,
        target: wgpu::ColorTargetState,
        primitive: wgpu::PrimitiveState,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) {
        let material = self.materails.get(&(shader_key as u8));
        if material.is_none() {
            let shader = get_shader(device, shader_key);
            let materail = PostprocessMaterail::new(shader);
            self.materails.insert(shader_key as u8, materail);
        }
        let materail = self.materails.get_mut(&(shader_key as u8)).unwrap();

        match shader_key {
            EPostprocessShader::CopyIntensity => CopyIntensityRenderer::check_pipeline(device, materail, geometry, target, primitive, depth_stencil),
            EPostprocessShader::ColorEffect => ColorEffectRenderer::check_pipeline(device, materail, geometry, target, primitive, depth_stencil),
            EPostprocessShader::BlurDual => BlurDualRenderer::check_pipeline(device, materail, geometry, target, primitive, depth_stencil),
            EPostprocessShader::BlurBokeh => BlurBokehRenderer::check_pipeline(device, materail, geometry, target, primitive, depth_stencil),
            EPostprocessShader::BlurRadial => BlurRadialRenderer::check_pipeline(device, materail, geometry, target, primitive, depth_stencil),
            EPostprocessShader::BlurDirect => BlurDirectRenderer::check_pipeline(device, materail, geometry, target, primitive, depth_stencil),
            EPostprocessShader::HorizonGlitch => HorizonGlitchRenderer::check_pipeline(device, materail, geometry, target, primitive, depth_stencil),
            EPostprocessShader::FilterBrightness => FilterBrightnessRenderer::check_pipeline(device, materail, geometry, target, primitive, depth_stencil),
            EPostprocessShader::Sobel => FilterSobelRenderer::check_pipeline(device, materail, geometry, target, primitive, depth_stencil),
            EPostprocessShader::RadialWave => RadialWaveRenderer::check_pipeline(device, materail, geometry, target, primitive, depth_stencil),
        };
    }

    pub fn get_material(
        &self,
        shader_key: EPostprocessShader,
    ) -> &PostprocessMaterail {
        self.materails.get(&(shader_key as u8)).unwrap()
    }

}