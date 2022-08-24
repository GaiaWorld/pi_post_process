use crate::{material::{shader::{Shader, EPostprocessShader}, target_format::ETexutureFormat, blend::EBlend, pipeline::{Pipeline, UniformBufferInfo}}, temprory_render_target::EPostprocessTarget};

pub enum ERenderParam<'a> {
    Encoder((&'a mut wgpu::CommandEncoder, &'a EPostprocessTarget<'a>)),
    RenderPass((&'a mut wgpu::RenderPass<'a>, ETexutureFormat)),
}

pub struct Renderer {
    pub pipeline_key: u128,
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
}

pub fn get_renderer(
    device: &wgpu::Device,
    pipeline: &Pipeline,
    shader_key: EPostprocessShader,
) -> Renderer {
    match shader_key {
        EPostprocessShader::CopyIntensity => super::copy_intensity::get_renderer(device, pipeline),
        EPostprocessShader::ColorEffect => super::color_effect::get_renderer(device, pipeline),
        EPostprocessShader::BlurDual => super::blur_dual::get_renderer(device, pipeline),
        EPostprocessShader::BlurBokeh => super::blur_bokeh::get_renderer(device, pipeline),
        EPostprocessShader::BlurRadial => super::blur_radial::get_renderer(device, pipeline),
        EPostprocessShader::BlurDirect => super::blur_direct::get_renderer(device, pipeline),
        EPostprocessShader::HorizonGlitch => super::horizon_glitch::get_renderer(device, pipeline),
        EPostprocessShader::FilterBrightness => super::filter_brightness::get_renderer(device, pipeline),
        EPostprocessShader::Sobel => super::filter_sobel::get_renderer(device, pipeline),
        EPostprocessShader::RadialWave => super::radial_wave::get_renderer(device, pipeline),
    }
}

pub fn get_pipeline(
    key: u128,
    vertex_layouts: &Vec<wgpu::VertexBufferLayout>,
    device: &wgpu::Device,
    shader_key: EPostprocessShader,
    shader: &Shader,
    blend: EBlend,
    format: ETexutureFormat,
) -> Pipeline {
    match shader_key {
        EPostprocessShader::CopyIntensity => super::copy_intensity::get_pipeline(key, vertex_layouts, device, shader, blend, format),
        EPostprocessShader::ColorEffect => super::color_effect::get_pipeline(key, vertex_layouts, device, shader, blend, format),
        EPostprocessShader::BlurDual => super::blur_dual::get_pipeline(key, vertex_layouts, device, shader, blend, format),
        EPostprocessShader::BlurBokeh => super::blur_bokeh::get_pipeline(key, vertex_layouts, device, shader, blend, format),
        EPostprocessShader::BlurRadial => super::blur_radial::get_pipeline(key, vertex_layouts, device, shader, blend, format),
        EPostprocessShader::BlurDirect => super::blur_direct::get_pipeline(key, vertex_layouts, device, shader, blend, format),
        EPostprocessShader::HorizonGlitch => super::horizon_glitch::get_pipeline(key, vertex_layouts, device, shader, blend, format),
        EPostprocessShader::FilterBrightness => super::filter_brightness::get_pipeline(key, vertex_layouts, device, shader, blend, format),
        EPostprocessShader::Sobel => super::filter_sobel::get_pipeline(key, vertex_layouts, device, shader, blend, format),
        EPostprocessShader::RadialWave => super::radial_wave::get_pipeline(key, vertex_layouts, device, shader, blend, format),
    }
}