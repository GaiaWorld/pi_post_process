
use std::result;

use pi_render::{components::view::target_alloc::{SafeAtlasAllocator, ShareTargetView}, rhi::{device::{RenderDevice}}};

use crate::{effect::{
    hsb::HSB, color_balance::ColorBalance, color_scale::ColorScale, area_mask::AreaMask,
    blur_dual::BlurDual, blur_radial::BlurRadial, blur_bokeh::BlurBokeh, blur_direct::BlurDirect,
    vignette::Vignette, copy::CopyIntensity, radial_wave::RadialWave, color_filter::ColorFilter, filter_sobel::FilterSobel, bloom_dual::BloomDual, horizon_glitch::HorizonGlitch, alpha::Alpha},
    temprory_render_target::{EPostprocessTarget, TemporaryRenderTargets}, renderer::{copy_intensity::{copy_intensity_render}, blur_dual::{blur_dual_render}, blur_direct::blur_direct_render, blur_radial::blur_radial_render, radial_wave::radial_wave_render, filter_sobel::filter_sobel_render, color_effect::color_effect_render, bloom_dual::bloom_dual_render, blur_bokeh::blur_bokeh_render, horizon_glitch::horizon_glitch_render}, postprocess_geometry::PostProcessGeometryManager, material::{target_format::{ETexutureFormat, get_target_texture_format}, blend::EBlend, shader::EPostprocessShader}, geometry::{vertex_buffer_layout::EVertexBufferLayout, IDENTITY_MATRIX}, postprocess_flags::PostprocessFlags, postprocess_renderer::{PostProcessRenderer, EPostprocessRenderType}, postprocess_pipeline::PostProcessPipeline
};

pub struct PostProcess {
    // pub area_mask:          Option<AreaMask>,
    
    pub copy:               Option<CopyIntensity>,
    pub alpha:              Option<Alpha>,
    pub hsb:                Option<HSB>,
    pub color_balance:      Option<ColorBalance>,
    pub color_scale:        Option<ColorScale>,
    pub vignette:           Option<Vignette>,
    pub color_filter:       Option<ColorFilter>,

    pub blur_dual:          Option<BlurDual>,
    pub blur_direct:        Option<BlurDirect>,
    pub blur_radial:        Option<BlurRadial>,
    pub blur_bokeh:         Option<BlurBokeh>,
    
    pub bloom_dual:         Option<BloomDual>,

    pub radial_wave:        Option<RadialWave>,
    pub filter_sobel:       Option<FilterSobel>,
    pub horizon_glitch:     Option<HorizonGlitch>,

    // pub clear:              Option<(u8, u8, u8, u8)>,
    flags:                  Vec<EPostprocessRenderType>,
    renders:                PostProcessRenderer,
}

impl Default for PostProcess {
    fn default() -> Self {
        Self {
            // area_mask:          None,
            copy:               None,
            alpha:              None,
            hsb:                None,
            color_balance:      None,
            color_filter:       None,
            color_scale:        None,
            vignette:           None,

            blur_dual:          None,
            blur_direct:        None,
            blur_radial:        None,
            blur_bokeh:         None,

            bloom_dual:         None,

            radial_wave:        None,
            filter_sobel:       None,
            horizon_glitch:     None,

            // clear:              None,
            flags:              vec![],
            renders:            PostProcessRenderer::new(),
        }
    }
}

/// * 处理渲染逻辑
///   * 设置对应效果数据
///   * 调用 calc 预计算渲染所需数据
///   * 调用 draw 进行渲染
/// * PS
///   * 内部检查效果参数,如果检查结果表明不需要实际渲染则 返回 Ok(false), src 未被渲染到 dst
///     * 可以通过显式设置 copy 参数强制确保 src 转移到 dst
///   * 当 src 与 dst 指向同一个 ShareTargetView 则可以实现内容原地变换(实际是 转移到临时目标 再转移到 dst)
///     * 当 内部检查结果只有一次渲染过程 则程序会崩溃
impl PostProcess {
    /// 绘制前计算和准备
    /// * `delta_time`
    ///   * 距离上次调用的间隔时间 ms
    /// * `src_format`
    ///   * 源纹理格式
    /// * `dst_format`
    ///   * 最终结果纹理的格式
    /// * `blend`
    ///   * 渲染到目标时的混合方式
    /// * `force_final_copy`
    ///   * 是否强制增加 拷贝过程(普通拷贝)
    pub fn calc(
        &mut self,
        delta_time: u64,
        render_device: &RenderDevice,
        postprocess_pipelines: &mut PostProcessPipeline,
        geometrys: &mut PostProcessGeometryManager,
        src_format: ETexutureFormat,
        dst_format: ETexutureFormat,
        blend: EBlend,
        force_final_copy: bool,
    ) {
        self.check(render_device, delta_time, geometrys, postprocess_pipelines, src_format, dst_format, blend, force_final_copy);
    }
    /// 对源内容进行后处理 - 最后一个效果的渲染在 draw_final 接口调用
    /// * `src`
    ///   * 源纹理内容
    /// * `dst`
    ///   * 接收处理结果的纹理尺寸
    /// * `return`
    ///   * Ok(EPostprocessTarget) 执行成功
    ///     * 返回 渲染结果纹理
    ///     * 当实际没有渲染 结果时 会返回 传入的src 对应数据
    ///       * Example: 模糊后处理, 模糊半径为 0 则认为不需要渲染过程, 应当直接使用 src, 返回 false
    ///   * Err(String)
    pub fn draw_front<'a, 'b>(
        &self,
        device: &RenderDevice,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        atlas_allocator: &'a SafeAtlasAllocator,
        postprocess_pipelines: &PostProcessPipeline,
        geometrys: &PostProcessGeometryManager,
        src: EPostprocessTarget<'a>,
        dst: (u32, u32),
    ) -> Result<EPostprocessTarget<'a>, String> {
        let mut temp_targets: TemporaryRenderTargets = TemporaryRenderTargets::new(atlas_allocator);

        let blend: EBlend = EBlend::None;
        let matrix: &[f32; 16] = &IDENTITY_MATRIX;

        let result = self._draw_front(
            device, queue, encoder, postprocess_pipelines, geometrys, src, dst, blend, matrix, &mut temp_targets
        );

        result
    }
    /// 对源内容进行最后一个效果的处理
    /// * `src`
    ///   * 源纹理内容
    /// * `dst`
    ///   * 接收处理结果的纹理
    /// * `blend`
    ///   * 渲染到目标时的混合方式
    /// * `matrix`
    ///   * 渲染到目标时的网格变换
    /// * `return`
    ///   * Ok(EPostprocessTarget) 执行成功
    ///     * 返回 渲染结果纹理
    ///     * 当实际没有渲染 结果时 会返回 传入的src 对应数据
    ///       * Example: 模糊后处理, 模糊半径为 0 则认为不需要渲染过程, 应当直接使用 src, 返回 false
    ///   * Err(String)
    pub fn draw_final<'a>(
        &self,
        device: &RenderDevice,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        atlas_allocator: &SafeAtlasAllocator,
        postprocess_pipelines: &PostProcessPipeline,
        geometrys: &PostProcessGeometryManager,
        src: EPostprocessTarget<'a>,
        dst: EPostprocessTarget<'a>,
        blend: EBlend,
        matrix: &[f32; 16],
    ) -> Result<EPostprocessTarget<'a>, String> {
        let mut temp_targets: TemporaryRenderTargets = TemporaryRenderTargets::new(atlas_allocator);
        let mut resource = (src.use_w(), src.use_h(), 0, src.format());
        let mut receiver = (dst.use_w(), dst.use_h(), 0, dst.format());
        resource.2 = temp_targets.record_from_other(src.clone());
        receiver.2 = temp_targets.record_from_other(dst.clone());
        let result = self._draw_final(
            device, queue, encoder, postprocess_pipelines, geometrys, resource, receiver, blend, matrix, &mut temp_targets
        );

        let result = match result {
            Ok(result) => match result {
                true => {
                    Ok(dst)
                },
                false => {
                    Ok(src)
                },
            },
            Err(e) => Err(e),
        };

        temp_targets.reset();

        result
    }

    fn _draw_front<'a>(
        &self,
        device: &RenderDevice,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        postprocess_pipelines: &PostProcessPipeline,
        geometrys: &PostProcessGeometryManager,
        src: EPostprocessTarget<'a>,
        dst: (u32, u32),
        blend: EBlend,
        matrix: &[f32; 16],
        temp_targets: &mut TemporaryRenderTargets<'a>,
    ) -> Result<EPostprocessTarget<'a>, String>  {
        let count = self.flags.len();
        let resource = src;

        if count > 1 {
            let src_id = temp_targets.record_from_other(resource.clone());
            let mut src = (resource.use_w(), resource.use_h(), src_id, resource.format());

            for i in 0..count-1 {
                let flag = *self.flags.get(i).unwrap();
                let temp_result = self._draw_single_front(device, queue, encoder, postprocess_pipelines, geometrys, src, dst, blend, matrix, flag, temp_targets);

                temp_targets.release(src.2);

                match temp_result {
                    Ok(id) => {
                        src.2 = id;
                    },
                    Err(e) => {
                        return Err(e);
                    },
                }
            }
        
            let view = temp_targets.get_share_target_view(Some(src.2)).unwrap();
            let format = temp_targets.get_format(src.2).unwrap();
            
            temp_targets.reset();

            Ok(EPostprocessTarget::from_share_target(view, format))
        } else {
            Ok(resource)
        }
    }

    fn _draw_final(
        &self,
        device: &RenderDevice,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        postprocess_pipelines: &PostProcessPipeline,
        geometrys: &PostProcessGeometryManager,
        src: (u32, u32, usize, ETexutureFormat),
        dst: (u32, u32, usize, ETexutureFormat),
        blend: EBlend,
        matrix: &[f32; 16],
        temp_targets: &mut TemporaryRenderTargets,
    ) -> Result<bool, String> {
        let count = self.flags.len();
        if count > 0 {
            let flag = *self.flags.get(count - 1).unwrap();
            let alpha = match self.alpha {
                Some(alpha) => alpha,
                None => Alpha::default(),
            };
            let temp_result = self._draw_single(&alpha, device, queue, encoder, postprocess_pipelines, geometrys, src, dst, blend, matrix, flag, temp_targets);

            if let Err(e) = temp_result {
                return Err(e);
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn _draw_single_front(
        &self,
        device: &RenderDevice,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        postprocess_pipelines: &PostProcessPipeline,
        geometrys: &PostProcessGeometryManager,
        src: (u32, u32, usize, ETexutureFormat),
        dst: (u32, u32),
        blend: EBlend,
        matrix: &[f32; 16],
        flag: EPostprocessRenderType,
        temp_targets: &mut TemporaryRenderTargets,
    ) -> Result<usize, String> {
        let (_, _, src_id, format) = src;
        let (width, height) = dst;
        let dst_id = temp_targets.create_share_target(Some(src_id), width, height, format);

        let result = self._draw_single(&Alpha::default(), device, queue, encoder, postprocess_pipelines, geometrys, src, (width, height, dst_id, format), blend, matrix, flag, temp_targets);

        match result {
            Ok(_) => Ok(dst_id),
            Err(e) => Err(e),
        }
    }

    fn _draw_single(
        &self,
        alpha: &Alpha,
        device: &RenderDevice,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        postprocess_pipelines: &PostProcessPipeline,
        geometrys: &PostProcessGeometryManager,
        src: (u32, u32, usize, ETexutureFormat),
        dst: (u32, u32, usize, ETexutureFormat),
        blend: EBlend,
        matrix: &[f32; 16],
        flag: EPostprocessRenderType,
        temp_targets: &mut TemporaryRenderTargets,
    ) -> Result<(), String> {
        let geometry = geometrys.get_geometry();
        let ( _, _, src_id, _) = src;
        let ( _, _, dst_id, _) = dst;
        match flag {
            EPostprocessRenderType::ColorEffect => {
                let src = temp_targets.get_target(src_id).unwrap();
                let dst = temp_targets.get_target(dst_id).unwrap();
                color_effect_render(&self.hsb, &self.color_balance, &self.color_scale, &self.vignette, &self.color_filter,  device, queue, encoder, postprocess_pipelines, &self.renders.color_effect.as_ref().unwrap(), geometry, src, dst, blend, matrix);
            },
            EPostprocessRenderType::BlurDual => {let blur_dual_result = blur_dual_render(self.blur_dual.as_ref().unwrap(), device, queue, encoder, postprocess_pipelines, self.renders.blur_dual.as_ref().unwrap(), geometry,src, dst, EBlend::None, EBlend::None, blend, matrix, temp_targets);
                if let Err(e) = blur_dual_result {
                    return Err(e);
                };
            },
            EPostprocessRenderType::BlurDirect => {
                let src = temp_targets.get_target(src_id).unwrap();
                let dst = temp_targets.get_target(dst_id).unwrap();
                blur_direct_render(self.blur_direct.as_ref().unwrap(), device, queue, encoder, postprocess_pipelines, self.renders.blur_direct.as_ref().unwrap(), geometry,src, dst, blend, matrix);
            },
            EPostprocessRenderType::BlurRadial => {
                let src = temp_targets.get_target(src_id).unwrap();
                let dst = temp_targets.get_target(dst_id).unwrap();
                blur_radial_render(self.blur_radial.as_ref().unwrap(), device, queue, encoder, postprocess_pipelines, self.renders.blur_radial.as_ref().unwrap(), geometry,src, dst, blend, matrix);
            },
            EPostprocessRenderType::BlurBokeh => {
                let src = temp_targets.get_target(src_id).unwrap();
                let dst = temp_targets.get_target(dst_id).unwrap();
                blur_bokeh_render(self.blur_bokeh.as_ref().unwrap(), device, queue, encoder, postprocess_pipelines,  self.renders.blur_bokeh.as_ref().unwrap(), geometry,src, dst, blend, matrix);
            },
            EPostprocessRenderType::BloomDual => {
                let bloom_dual_result = bloom_dual_render(self.bloom_dual.as_ref().unwrap(), device, queue, encoder, postprocess_pipelines,  self.renders.bloom_dual.as_ref().unwrap(), geometry, src, dst, blend, matrix, temp_targets);
                if let Err(e) = bloom_dual_result {
                    return Err(e);
                };
            },
            EPostprocessRenderType::RadialWave => {
                let src = temp_targets.get_target(src_id).unwrap();
                let dst = temp_targets.get_target(dst_id).unwrap();
                radial_wave_render(self.radial_wave.as_ref().unwrap(), device, queue, encoder, postprocess_pipelines, self.renders.radial_wave.as_ref().unwrap(), geometry, src, dst, blend, matrix);
            },
            EPostprocessRenderType::HorizonGlitch => {
                let src = temp_targets.get_target(src_id).unwrap();
                let dst = temp_targets.get_target(dst_id).unwrap();
                horizon_glitch_render(self.horizon_glitch.as_ref().unwrap(), device, queue, encoder, postprocess_pipelines, self.renders.horizon_glitch.as_ref().unwrap(), geometry, geometrys.get_glitch_geometry(), src, dst, blend, matrix);
            },
            EPostprocessRenderType::FilterSobel => {
                let src = temp_targets.get_target(src_id).unwrap();
                let dst = temp_targets.get_target(dst_id).unwrap();
                filter_sobel_render(self.filter_sobel.as_ref().unwrap(), device, queue, encoder, postprocess_pipelines, self.renders.filter_sobel.as_ref().unwrap(), geometry, src, dst, blend, matrix);
            },
            EPostprocessRenderType::CopyIntensity => {
                let src = temp_targets.get_target(src_id).unwrap();
                let dst = temp_targets.get_target(dst_id).unwrap();
                copy_intensity_render(self.copy.as_ref().unwrap(), alpha, device, queue, encoder, postprocess_pipelines, self.renders.copy_intensity.as_ref().unwrap(), geometry, src, dst, blend, matrix);
            },
            EPostprocessRenderType::FinalCopyIntensity => {
                let src = temp_targets.get_target(src_id).unwrap();
                let dst = temp_targets.get_target(dst_id).unwrap();
                copy_intensity_render(&CopyIntensity::default(), alpha, device, queue, encoder, postprocess_pipelines, self.renders.final_copy_renderer.as_ref().unwrap(), geometry, src, dst, blend, matrix);
            },
        }
        Ok(())
    }

    fn check(
        &mut self,
        render_device: &RenderDevice,
        delta_time: u64,
        geometrys: &mut PostProcessGeometryManager,
        postprocess_pipelines: &mut PostProcessPipeline,
        src_format: ETexutureFormat,
        dst_format: ETexutureFormat,
        blend: EBlend,
        force_final_copy: bool,
    ) {
        self.flags.clear();

        // color
        let color_effect     = (self.hsb.is_some() && self.hsb.as_ref().unwrap().is_enabled()) 
            || self.color_balance.is_some() && self.color_balance.as_ref().unwrap().is_enabled()
            || self.color_scale.is_some() && self.color_scale.as_ref().unwrap().is_enabled()
            || self.vignette.is_some() && self.vignette.as_ref().unwrap().is_enabled()
            || self.color_filter.is_some() && self.color_filter.as_ref().unwrap().is_enabled()
        ;

        let blur_dual        = self.blur_dual.is_some() && self.blur_dual.as_ref().unwrap().is_enabled();
        let blur_direct      = self.blur_direct.is_some() && self.blur_direct.as_ref().unwrap().is_enabled();
        let blur_radial      = self.blur_radial.is_some() && self.blur_radial.as_ref().unwrap().is_enabled();
        let blur_bokeh       = self.blur_bokeh.is_some() && self.blur_bokeh.as_ref().unwrap().is_enabled();
        let radial_wave      = self.radial_wave.is_some() && self.radial_wave.as_ref().unwrap().is_enabled();
        let bloom_dual       = self.bloom_dual.is_some() && self.bloom_dual.as_ref().unwrap().is_enabled();
        let filter_sobel     = self.filter_sobel.is_some() && self.filter_sobel.as_ref().unwrap().is_enabled();
        let horizon_glitch   = self.horizon_glitch.is_some() && self.horizon_glitch.as_ref().unwrap().is_enabled();
        let copy_intensity   = self.copy.is_some();

        let device = render_device.wgpu_device();

        self.renders.check_copy_intensity(device, geometrys, postprocess_pipelines, src_format, dst_format, blend);

        if color_effect {
            self.renders.check_color_effect(device, geometrys, postprocess_pipelines, src_format, dst_format, blend);
            self.flags.push(EPostprocessRenderType::ColorEffect);
        }
        if blur_dual {
            self.renders.check_blur_dual(device, geometrys, postprocess_pipelines, src_format, dst_format, blend);
            self.flags.push(EPostprocessRenderType::BlurDual);
        }
        if blur_direct {
            self.renders.check_blur_direct(device, geometrys, postprocess_pipelines, src_format, dst_format, blend);
            self.flags.push(EPostprocessRenderType::BlurDirect);
        }
        if blur_radial {
            self.renders.check_blur_radial(device, geometrys, postprocess_pipelines, src_format, dst_format, blend);
            self.flags.push(EPostprocessRenderType::BlurRadial);
        }
        if blur_bokeh {
            self.renders.check_blur_bokeh(device, geometrys, postprocess_pipelines, src_format, dst_format, blend);
            self.flags.push(EPostprocessRenderType::BlurBokeh);
        }
        if bloom_dual {
            self.renders.check_bloom_dual(device, geometrys, postprocess_pipelines, src_format, dst_format, blend);
            self.flags.push(EPostprocessRenderType::BloomDual);
        }
        if radial_wave {
            self.renders.check_radial_wave(device, geometrys, postprocess_pipelines, src_format, dst_format, blend);
            self.flags.push(EPostprocessRenderType::RadialWave);
        }
        if horizon_glitch {
            self.renders.check_horizon_glitch(device, geometrys, postprocess_pipelines, src_format, dst_format, blend);
            self.horizon_glitch.as_mut().unwrap().update(delta_time);
            self.flags.push(EPostprocessRenderType::HorizonGlitch);
        }
        if filter_sobel {
            self.renders.check_sobel(device, geometrys, postprocess_pipelines, src_format, dst_format, blend);
            self.flags.push(EPostprocessRenderType::FilterSobel);
        }
        if copy_intensity {
            self.flags.push(EPostprocessRenderType::CopyIntensity);
        }
        if force_final_copy {
            self.flags.push(EPostprocessRenderType::FinalCopyIntensity);
        }
        if self.alpha.is_some() && !copy_intensity && !force_final_copy {
            self.flags.push(EPostprocessRenderType::FinalCopyIntensity);
        }
    }
}