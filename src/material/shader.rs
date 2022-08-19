use super::tools::load_shader;


pub struct Shader {
    pub vs_module: wgpu::ShaderModule,
    pub fs_module: wgpu::ShaderModule,
}

#[derive(Debug, Copy, Clone)]
pub enum EPostprocessShader {
    CopyIntensity = 1,
    ColorEffect,
    BlurDual,
    BlurBokeh,
    BlurRadial,
    BlurDirect,
    HorizonGlitch,
    FilterBrightness,
    Sobel,
    RadialWave
}

pub const MOVE_E_POSTPROCESS_SHADER: u128 = 100;

pub fn get_shader(
    device: &wgpu::Device,
    shader: EPostprocessShader
) -> Shader {
    match shader {
        EPostprocessShader::CopyIntensity => load_shader(
            device,
            include_str!("../shaders/base.vert"),
            include_str!("../shaders/copy.frag"),
            "CopyIntensity_VS",
            "CopyIntensity_FS"
        ),
        EPostprocessShader::ColorEffect => load_shader(
            device,
            include_str!("../shaders/base.vert"),
            include_str!("../shaders/color_effect.frag"),
            "ColorEffect_vs",
            "ColorEffect_Fs"
        ),
        EPostprocessShader::BlurDual => load_shader(
            device,
            include_str!("../shaders/blur_dual.vert"),
            include_str!("../shaders/blur_dual.frag"),
            "BlurDual",
            "BlurDual"
        ),
        EPostprocessShader::BlurBokeh => load_shader(
            device,
            include_str!("../shaders/base.vert"),
            include_str!("../shaders/blur_bokeh.frag"),
            "blur_bokeh",
            "blur_bokeh"
        ),
        EPostprocessShader::BlurRadial => load_shader(
            device,
            include_str!("../shaders/base.vert"),
            include_str!("../shaders/blur_radial.frag"),
            "blur_radial",
            "blur_radial"
        ),
        EPostprocessShader::BlurDirect => load_shader(
            device,
            include_str!("../shaders/base.vert"),
            include_str!("../shaders/blur_direct.frag"),
            "blur_direct",
            "blur_direct"
        ),
        EPostprocessShader::HorizonGlitch => load_shader(
            device,
            include_str!("../shaders/horizon_glitch.vert"),
            include_str!("../shaders/horizon_glitch.frag"),
            "horizon_glitch",
            "horizon_glitch"
        ),
        EPostprocessShader::FilterBrightness => load_shader(
            device,
            include_str!("../shaders/base.vert"),
            include_str!("../shaders/filter_brightness.frag"),
            "FilterBrightness_VS",
            "FilterBrightness_FS"
        ),
        EPostprocessShader::Sobel => load_shader(
            device,
            include_str!("../shaders/base.vert"),
            include_str!("../shaders/sobel.frag"),
            "Sobel_VS",
            "Sobel_FS"
        ),
        EPostprocessShader::RadialWave => load_shader(
            device,
            include_str!("../shaders/base.vert"),
            include_str!("../shaders/radial_wave.frag"),
            "radial_wave",
            "radial_wave"
        ),
    }
}