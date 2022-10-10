

use crate::{renderer::{bloom_dual::BloomDualRenderer, horizon_glitch::HorizonGlitchRenderer, filter_sobel::FilterSobelRenderer, filter_brightness::FilterBrightnessRenderer, copy_intensity::{CopyIntensityRenderer, self}, color_effect::ColorEffectRenderer, blur_radial::BlurRadialRenderer, blur_direct::BlurDirectRenderer, blur_bokeh::BlurBokehRenderer, blur_dual::BlurDualRenderer, radial_wave::RadialWaveRenderer, renderer}, postprocess_geometry::PostProcessGeometryManager, postprocess_pipeline::PostProcessMaterialMgr, material::{shader::EPostprocessShader, blend::{EBlend, get_blend_state}, fragment_state::{create_default_target, create_target}}, geometry::vertex_buffer_layout::EVertexBufferLayout};

#[derive(Debug, Copy, Clone)]
pub enum EPostprocessRenderType {
    ColorEffect,
    BlurDual,
    BlurDirect,
    BlurRadial,
    BlurBokeh,
    BloomDual,
    RadialWave,
    HorizonGlitch,
    FilterSobel,
    CopyIntensity,
    FinalCopyIntensity,
}

pub struct PostProcessRenderer {
    pub bloom_dual:         Option<BloomDualRenderer>,
    pub blur_bokeh:         Option<BlurBokehRenderer>,
    pub blur_direct:        Option<BlurDirectRenderer>,
    pub blur_dual:          Option<BlurDualRenderer>,
    pub blur_radial:        Option<BlurRadialRenderer>,
    pub color_effect:       Option<ColorEffectRenderer>,
    pub copy_intensity:     Option<CopyIntensityRenderer>,
    pub filter_brightness:  Option<FilterBrightnessRenderer>,
    pub filter_sobel:       Option<FilterSobelRenderer>,
    pub horizon_glitch:     Option<HorizonGlitchRenderer>,
    pub radial_wave:        Option<RadialWaveRenderer>,
    pub final_copy_renderer:Option<CopyIntensityRenderer>,
    pub flags:              (u8, bool, bool, bool, bool, bool, bool, bool, bool, bool, bool)
}

impl PostProcessRenderer {
    pub fn new() -> Self {
        Self {
            bloom_dual: None,
            blur_bokeh: None,
            blur_direct: None,
            blur_dual: None,
            blur_radial: None,
            color_effect: None,
            copy_intensity: None,
            filter_brightness: None,
            filter_sobel: None,
            horizon_glitch: None,
            radial_wave: None,
            final_copy_renderer: None,
            flags: (0, false, false, false, false, false, false, false, false, false, false),
        }
    }

    pub fn update(
        &mut self,
        flags: (u8, bool, bool, bool, bool, bool, bool, bool, bool, bool, bool)
    ) {

    }

    pub fn check_copy_intensity(
        &mut self,
        device: &wgpu::Device,
        geometrys: &mut PostProcessGeometryManager,
        postprocess_pipelines: &mut PostProcessMaterialMgr,
        primitive: wgpu::PrimitiveState,
        targets: &[Option<wgpu::ColorTargetState>],
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) {
        if self.copy_intensity.is_none() {
            let geometry = geometrys.check_geometry(device);
            let shader_key = EPostprocessShader::CopyIntensity;
            let vertex_buffer_key = EVertexBufferLayout::Position2D;

            postprocess_pipelines.check_pipeline(device, geometry, shader_key, &[Some(create_default_target())], primitive, None);
            postprocess_pipelines.check_pipeline(device, geometry, shader_key, targets, primitive, depth_stencil);

            self.copy_intensity = Some(CopyIntensityRenderer{ copy: renderer::get_renderer(device, shader_key) });
            self.final_copy_renderer = Some(CopyIntensityRenderer{ copy: renderer::get_renderer(device, shader_key) });
        }
    }

    pub fn check_color_effect(
        &mut self,
        device: &wgpu::Device,
        geometrys: &mut PostProcessGeometryManager,
        postprocess_pipelines: &mut PostProcessMaterialMgr,
        primitive: wgpu::PrimitiveState,
        targets: &[Option<wgpu::ColorTargetState>],
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) {
        if self.color_effect.is_none() {
            let geometry = geometrys.check_geometry(device);
            let shader_key = EPostprocessShader::ColorEffect;
            let vertex_buffer_key = EVertexBufferLayout::Position2D;

            postprocess_pipelines.check_pipeline(device, geometry, shader_key, &[Some(create_default_target())], primitive, None);
            postprocess_pipelines.check_pipeline(device, geometry, shader_key, targets, primitive, depth_stencil);

            self.color_effect = Some(ColorEffectRenderer{ effect: renderer::get_renderer(device, shader_key) });
        }
    }

    pub fn check_blur_bokeh(
        &mut self,
        device: &wgpu::Device,
        geometrys: &mut PostProcessGeometryManager,
        postprocess_pipelines: &mut PostProcessMaterialMgr,
        primitive: wgpu::PrimitiveState,
        targets: &[Option<wgpu::ColorTargetState>],
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) {
        if self.blur_bokeh.is_none() {
            let geometry = geometrys.check_geometry(device);
            let shader_key = EPostprocessShader::BlurBokeh;
            let vertex_buffer_key = EVertexBufferLayout::Position2D;

            postprocess_pipelines.check_pipeline(device, geometry, shader_key, &[Some(create_default_target())], primitive, None);
            postprocess_pipelines.check_pipeline(device, geometry, shader_key, targets, primitive, depth_stencil);

            self.blur_bokeh = Some(BlurBokehRenderer{ bokeh: renderer::get_renderer(device, shader_key) });
        }
    }
    

    pub fn check_blur_direct(
        &mut self,
        device: &wgpu::Device,
        geometrys: &mut PostProcessGeometryManager,
        postprocess_pipelines: &mut PostProcessMaterialMgr,
        primitive: wgpu::PrimitiveState,
        targets: &[Option<wgpu::ColorTargetState>],
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) {
        if self.blur_direct.is_none() {
            let geometry = geometrys.check_geometry(device);
            let shader_key = EPostprocessShader::BlurDirect;
            let vertex_buffer_key = EVertexBufferLayout::Position2D;

            postprocess_pipelines.check_pipeline(device, geometry, shader_key, &[Some(create_default_target())], primitive, None);
            postprocess_pipelines.check_pipeline(device, geometry, shader_key, targets, primitive, depth_stencil);

            self.blur_direct = Some(BlurDirectRenderer{ direct: renderer::get_renderer(device, shader_key) });
        }
    }
    

    pub fn check_blur_dual(
        &mut self,
        device: &wgpu::Device,
        geometrys: &mut PostProcessGeometryManager,
        postprocess_pipelines: &mut PostProcessMaterialMgr,
        primitive: wgpu::PrimitiveState,
        targets: &[Option<wgpu::ColorTargetState>],
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) {
        if self.blur_dual.is_none() {
            let geometry = geometrys.check_geometry(device);
            let shader_key = EPostprocessShader::BlurDual;
            let vertex_buffer_key = EVertexBufferLayout::Position2D;

            postprocess_pipelines.check_pipeline(device, geometry, shader_key, &[Some(create_default_target())], primitive, None);
            postprocess_pipelines.check_pipeline(device, geometry, shader_key, targets, primitive, depth_stencil);

            self.blur_dual = Some(BlurDualRenderer{ down_first: renderer::get_renderer(device, shader_key), down: renderer::get_renderer(device, shader_key), up: renderer::get_renderer(device, shader_key), up_final: renderer::get_renderer(device, shader_key) });
        }
    }

    pub fn check_blur_radial(
        &mut self,
        device: &wgpu::Device,
        geometrys: &mut PostProcessGeometryManager,
        postprocess_pipelines: &mut PostProcessMaterialMgr,
        primitive: wgpu::PrimitiveState,
        targets: &[Option<wgpu::ColorTargetState>],
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) {
        if self.blur_radial.is_none() {
            let geometry = geometrys.check_geometry(device);
            let shader_key = EPostprocessShader::BlurRadial;
            let vertex_buffer_key = EVertexBufferLayout::Position2D;

            postprocess_pipelines.check_pipeline(device, geometry, shader_key, &[Some(create_default_target())], primitive, None);
            postprocess_pipelines.check_pipeline(device, geometry, shader_key, targets, primitive, depth_stencil);

            self.blur_radial = Some(BlurRadialRenderer{ radial: renderer::get_renderer(device, shader_key) });
        }
    }

    pub fn check_radial_wave(
        &mut self,
        device: &wgpu::Device,
        geometrys: &mut PostProcessGeometryManager,
        postprocess_pipelines: &mut PostProcessMaterialMgr,
        primitive: wgpu::PrimitiveState,
        targets: &[Option<wgpu::ColorTargetState>],
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) {
        if self.radial_wave.is_none() {
            let geometry = geometrys.check_geometry(device);
            let shader_key = EPostprocessShader::RadialWave;
            let vertex_buffer_key = EVertexBufferLayout::Position2D;

            postprocess_pipelines.check_pipeline(device, geometry, shader_key, &[Some(create_default_target())], primitive, None);
            postprocess_pipelines.check_pipeline(device, geometry, shader_key, targets, primitive, depth_stencil);

            self.radial_wave = Some(RadialWaveRenderer{ wave: renderer::get_renderer(device, shader_key) });
        }
    }

    pub fn check_bloom_dual(
        &mut self,
        device: &wgpu::Device,
        geometrys: &mut PostProcessGeometryManager,
        postprocess_pipelines: &mut PostProcessMaterialMgr,
        primitive: wgpu::PrimitiveState,
        targets: &[Option<wgpu::ColorTargetState>],
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) {
        if self.bloom_dual.is_none() {
            let geometry = geometrys.check_geometry(device);
            let vertex_buffer_key = EVertexBufferLayout::Position2D;

            let shader_key = EPostprocessShader::FilterBrightness;
            postprocess_pipelines.check_pipeline(device, geometry, shader_key, &[Some(create_default_target())], primitive, None);

            let filter_brightness = FilterBrightnessRenderer { filter: renderer::get_renderer(device, shader_key) };
            
            let shader_key = EPostprocessShader::BlurDual;
            postprocess_pipelines.check_pipeline(device, geometry, shader_key, &[Some(create_default_target())], primitive, None);

            let target_temp = create_target(wgpu::TextureFormat::Rgba8UnormSrgb, get_blend_state(EBlend::Add), wgpu::ColorWrites::ALL);
            postprocess_pipelines.check_pipeline(device, geometry, shader_key, &[Some(target_temp.clone())], primitive, None);

            let dual = BlurDualRenderer{ down_first: renderer::get_renderer(device, shader_key), down: renderer::get_renderer(device, shader_key), up: renderer::get_renderer(device, shader_key), up_final: renderer::get_renderer(device, shader_key) };

            let shader_key = EPostprocessShader::CopyIntensity;
            postprocess_pipelines.check_pipeline(device, geometry, shader_key, &[Some(target_temp.clone())], primitive, None);
            postprocess_pipelines.check_pipeline(device, geometry, shader_key, targets, primitive, depth_stencil);

            let copy = CopyIntensityRenderer{ copy: renderer::get_renderer(device, shader_key) };
            let copy_2 = CopyIntensityRenderer{ copy: renderer::get_renderer(device, shader_key) };
    
            self.bloom_dual = Some(BloomDualRenderer{ filter_brightness, dual, copy, copy_2, filter_rt_id: None });
        }
    }

    pub fn check_sobel(
        &mut self,
        device: &wgpu::Device,
        geometrys: &mut PostProcessGeometryManager,
        postprocess_pipelines: &mut PostProcessMaterialMgr,
        primitive: wgpu::PrimitiveState,
        targets: &[Option<wgpu::ColorTargetState>],
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) {
        if self.filter_sobel.is_none() {
            let geometry = geometrys.check_geometry(device);
            let shader_key = EPostprocessShader::Sobel;
            let vertex_buffer_key = EVertexBufferLayout::Position2D;

            postprocess_pipelines.check_pipeline(device, geometry, shader_key, &[Some(create_default_target())], primitive, None);
            postprocess_pipelines.check_pipeline(device, geometry, shader_key, targets, primitive, depth_stencil);

            self.filter_sobel = Some(FilterSobelRenderer{ sobel: renderer::get_renderer(device, shader_key) });
                
        }
    }

    pub fn check_horizon_glitch(
        &mut self,
        device: &wgpu::Device,
        geometrys: &mut PostProcessGeometryManager,
        postprocess_pipelines: &mut PostProcessMaterialMgr,
        primitive: wgpu::PrimitiveState,
        targets: &[Option<wgpu::ColorTargetState>],
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) {
        if self.horizon_glitch.is_none() {
            geometrys.check_glitch_geometry(device);

            let geometry = geometrys.get_geometry();
            let glitch_geometry = geometrys.get_glitch_geometry();

            let shader_key = EPostprocessShader::CopyIntensity;
            let vertex_buffer_key = EVertexBufferLayout::Position2D;
            postprocess_pipelines.check_pipeline(device, geometry, shader_key, &[Some(create_default_target())], primitive, None);
            postprocess_pipelines.check_pipeline(device, geometry, shader_key, targets, primitive.clone(), depth_stencil.clone());

            let copy = CopyIntensityRenderer { copy: renderer::get_renderer(device, shader_key) };
            
            let shader_key = EPostprocessShader::HorizonGlitch;
            let vertex_buffer_key = EVertexBufferLayout::Position2DGlitchInstance;
            postprocess_pipelines.check_pipeline(device, glitch_geometry, shader_key, &[Some(create_default_target())], primitive.clone(), None);
            postprocess_pipelines.check_pipeline(device, glitch_geometry, shader_key, targets, primitive.clone(), depth_stencil.clone());

            let glitch = renderer::get_renderer(device, shader_key);

            self.horizon_glitch = Some(HorizonGlitchRenderer{ copy, glitch });
                
        }
    }
}