use crate::{material::{shader::{Shader, EPostprocessShader}, tools::UniformBufferInfo}, temprory_render_target::EPostprocessTarget};

use super::{copy_intensity::CopyIntensityRenderer, color_effect::ColorEffectRenderer, blur_dual::BlurDualRenderer, blur_bokeh::BlurBokehRenderer, blur_radial::BlurRadialRenderer, blur_direct::BlurDirectRenderer, horizon_glitch::HorizonGlitchRenderer, filter_brightness::FilterBrightnessRenderer, filter_sobel::FilterSobelRenderer, radial_wave::RadialWaveRenderer};

pub enum ERenderParam<'a> {
    Encoder((&'a mut wgpu::CommandEncoder, &'a EPostprocessTarget<'a>)),
    RenderPass((&'a mut wgpu::RenderPass<'a>, wgpu::TextureFormat)),
}

pub struct Renderer {
    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
    pub ubo_info: UniformBufferInfo,
}

impl Renderer {
    pub fn update_vertex_matrix(
        &self,
        queue: &wgpu::Queue,
        vertex_matrix: &[f32]
    ) {

    }
    pub fn update_param(
        &self,
        queue: &wgpu::Queue,
        param: &[f32]
    ) {

    }
    pub fn update_diffuse_matrix(
        &self,
        queue: &wgpu::Queue,
        param: &[f32]
    ) {

    }

    pub fn vs_state<'a>(
        shader: &'a Shader,
        vertex_layouts: &'a Vec<wgpu::VertexBufferLayout>,
    ) -> wgpu::VertexState<'a> {
        wgpu::VertexState {
            module: &shader.vs_module,
            entry_point: "main",
            buffers: vertex_layouts,
        }
    }

    pub fn fs_state<'a>(
        shader: &'a Shader,
        targets: &'a [wgpu::ColorTargetState],
    ) -> wgpu::FragmentState<'a> {
        wgpu::FragmentState {
            module: &shader.fs_module,
            entry_point: "main",
            targets,
        }
    }
}

pub fn get_renderer(
    device: &wgpu::Device,
    shader_key: EPostprocessShader,
) -> Renderer {
    match shader_key {
        EPostprocessShader::CopyIntensity => CopyIntensityRenderer::get_renderer(device),
        EPostprocessShader::ColorEffect => ColorEffectRenderer::get_renderer(device),
        EPostprocessShader::BlurDual => BlurDualRenderer::get_renderer(device),
        EPostprocessShader::BlurBokeh => BlurBokehRenderer::get_renderer(device),
        EPostprocessShader::BlurRadial => BlurRadialRenderer::get_renderer(device),
        EPostprocessShader::BlurDirect => BlurDirectRenderer::get_renderer(device),
        EPostprocessShader::HorizonGlitch => HorizonGlitchRenderer::get_renderer(device),
        EPostprocessShader::FilterBrightness => FilterBrightnessRenderer::get_renderer(device),
        EPostprocessShader::Sobel => FilterSobelRenderer::get_renderer(device),
        EPostprocessShader::RadialWave => RadialWaveRenderer::get_renderer(device),
    }
}