

use pi_render::{components::view::target_alloc::{SafeAtlasAllocator}, rhi::{device::{RenderDevice}}};

use crate::{effect::{
    hsb::HSB, color_balance::ColorBalance, color_scale::ColorScale,
    blur_dual::BlurDual, blur_radial::BlurRadial, blur_bokeh::BlurBokeh, blur_direct::BlurDirect,
    vignette::Vignette, copy::CopyIntensity, radial_wave::RadialWave, color_filter::ColorFilter, filter_sobel::FilterSobel, bloom_dual::BloomDual, horizon_glitch::HorizonGlitch, alpha::Alpha},
    temprory_render_target::{EPostprocessTarget, TemporaryRenderTargets}, renderer::{copy_intensity::{copy_intensity_render}, blur_dual::{blur_dual_render}, blur_direct::blur_direct_render, blur_radial::blur_radial_render, radial_wave::radial_wave_render, filter_sobel::filter_sobel_render, color_effect::{color_effect_render, ColorEffectRenderer}, bloom_dual::bloom_dual_render, blur_bokeh::blur_bokeh_render, horizon_glitch::horizon_glitch_render, renderer::ERenderParam}, postprocess_geometry::PostProcessGeometryManager, material::{ blend::EBlend, shader::EPostprocessShader, tools::{get_texture_binding_group, SimpleRenderExtendsData, TextureScaleOffset}, fragment_state::create_default_target}, geometry::{IDENTITY_MATRIX}, postprocess_renderer::{PostProcessRenderer, EPostprocessRenderType}, postprocess_pipeline::PostProcessMaterialMgr, error::EPostprocessError
};

pub struct PostProcess {
    // pub area_mask:          Option<AreaMask>,
    
    pub alpha:              Option<Alpha>,
    pub copy:               Option<CopyIntensity>,
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
    /// * `target`
    ///   * 最终结果的 ColorTarget
    /// * `depth_stencil`
    ///   * 最终结果的 DepthStencil
    pub fn calc(
        &mut self,
        delta_time: u64,
        render_device: &RenderDevice,
        postprocess_pipelines: &mut PostProcessMaterialMgr,
        geometrys: &mut PostProcessGeometryManager,
        targets: &[Option<wgpu::ColorTargetState>],
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) {
        self.check(render_device, delta_time, geometrys, postprocess_pipelines, targets, depth_stencil, true);
        // println!("{:?}", self.flags);
    }
    /// 对源内容进行后处理 - 最后一个效果的渲染在 draw_final 接口调用
    /// * `src`
    ///   * 源纹理内容
    /// * `dst_size`
    ///   * 接收处理结果的纹理尺寸
    /// * `return`
    ///   * Ok(EPostprocessTarget) 执行成功
    ///     * 返回 渲染结果纹理
    ///     * 当实际没有渲染 结果时 会返回 传入的src 对应数据
    ///       * Example: 模糊后处理, 模糊半径为 0 则认为不需要渲染过程, 应当直接使用 src, 返回 false
    ///   * Err(String)
    pub fn draw_front<'a>(
        &self,
        device: &RenderDevice,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        atlas_allocator: &'a SafeAtlasAllocator,
        postprocess_pipelines: &PostProcessMaterialMgr,
        geometrys: &PostProcessGeometryManager,
        src: EPostprocessTarget<'a>,
        dst_size: (u32, u32),
    ) -> Result<EPostprocessTarget<'a>, EPostprocessError> {

        let matrix: &[f32] = &IDENTITY_MATRIX;

        let result = self._draw_front(
            device, queue, atlas_allocator, encoder, postprocess_pipelines, geometrys, src, dst_size, matrix, 
        );

        result
    }
    /// 获取 draw_final 需要的 texture_bind_group 
    /// * 获取失败, 则不能调用 draw_final
    /// * `target`
    ///   * 最终结果的 ColorTarget
    /// * `depth_stencil`
    ///   * 最终结果的 DepthStencil
    pub fn get_final_texture_bind_group<'a>(
        &'a self,
        device: &'a RenderDevice,
        postprocess_pipelines: &'a PostProcessMaterialMgr,
        src: &'a EPostprocessTarget,
        targets: &[Option<wgpu::ColorTargetState>],
        depth_stencil: &Option<wgpu::DepthStencilState>,
    ) -> Option<wgpu::BindGroup> {
        
        let primitive: wgpu::PrimitiveState = wgpu::PrimitiveState::default();

        match self.flags.last() {
            Some(flag) => {
                self.get_texture_bind_group(device, postprocess_pipelines, src, targets[0].as_ref().unwrap(), &primitive, depth_stencil, *flag)
            },
            None => {
                None
            },
        }
    }
    fn get_texture_bind_group<'a>(
        &'a self,
        device: &'a RenderDevice,
        postprocess_pipelines: &'a PostProcessMaterialMgr,
        src: &'a EPostprocessTarget,
        target: &wgpu::ColorTargetState,
        primitive: &wgpu::PrimitiveState,
        depth_stencil: &Option<wgpu::DepthStencilState>,
        flag: EPostprocessRenderType,
    ) -> Option<wgpu::BindGroup> {
        let pipeline = match flag {
            EPostprocessRenderType::ColorEffect => {
                Some(postprocess_pipelines.get_material( EPostprocessShader::ColorEffect).get_pipeline(target, primitive, depth_stencil))
            },
            EPostprocessRenderType::BlurDirect => {
                Some(postprocess_pipelines.get_material( EPostprocessShader::BlurDirect).get_pipeline(target, primitive, depth_stencil))
            },
            EPostprocessRenderType::BlurRadial => {
                Some(postprocess_pipelines.get_material( EPostprocessShader::BlurRadial).get_pipeline(target, primitive, depth_stencil))
            },
            EPostprocessRenderType::BlurBokeh => {
                Some(postprocess_pipelines.get_material( EPostprocessShader::BlurBokeh).get_pipeline(target, primitive, depth_stencil))
            },
            EPostprocessRenderType::RadialWave => {
                Some(postprocess_pipelines.get_material( EPostprocessShader::RadialWave).get_pipeline(target, primitive, depth_stencil))
            },
            EPostprocessRenderType::HorizonGlitch => {
                Some(postprocess_pipelines.get_material( EPostprocessShader::HorizonGlitch).get_pipeline(target, primitive, depth_stencil))
            },
            EPostprocessRenderType::FilterSobel => {
                Some(postprocess_pipelines.get_material( EPostprocessShader::Sobel).get_pipeline(target, primitive, depth_stencil))
            },
            EPostprocessRenderType::CopyIntensity => {
                Some(postprocess_pipelines.get_material( EPostprocessShader::CopyIntensity).get_pipeline(target, primitive, depth_stencil))
            },
            EPostprocessRenderType::FinalCopyIntensity => {
                Some(postprocess_pipelines.get_material( EPostprocessShader::CopyIntensity).get_pipeline(target, primitive, depth_stencil))
            },
            _ => {
                None
            }
        };
        match pipeline {
            Some(pipeline) => Some(
                get_texture_binding_group(&pipeline.texture_bind_group_layout, device, src.view())),
            None => None,
        }
    }
    /// 对源内容进行最后一个效果的处理
    /// * 调用前必须的操作:
    ///   * 通过 get_final_texture_bind_group 获取 texture_bind_group
    ///   * 创建 renderpass
    /// * `src`
    ///   * 源纹理内容
    /// * `target`
    ///   * 最终结果的 ColorTarget
    /// * `depth_stencil`
    ///   * 最终结果的 DepthStencil
    /// * `matrix`
    ///   * 渲染到目标时的网格变换
    /// * `depth`
    ///   * 渲染到目标时的深度值
    /// * `return`
    ///   * Ok(true) 执行成功
    ///   * Ok(false) 当实际没有渲染时
    ///       * Example: 模糊后处理, 模糊半径为 0 则认为不需要渲染过程, 应当直接使用 src, 返回 false
    ///   * Err(String)
    pub fn draw_final<'a>(
        &'a self,
        device: & RenderDevice,
        queue: & wgpu::Queue,
        postprocess_pipelines: &'a PostProcessMaterialMgr,
        geometrys: &'a PostProcessGeometryManager,
        renderpass: & mut wgpu::RenderPass<'a>,
        texture: EPostprocessTarget<'a>,
        texture_bind_group: &'a wgpu::BindGroup,
        targets: &[Option<wgpu::ColorTargetState>],
        depth_stencil: &Option<wgpu::DepthStencilState>,
        matrix: &[f32],
        depth: f32,
    ) -> Result<bool, EPostprocessError> {
        let alpha = match self.alpha {
            Some(alpha) => alpha.a,
            None => 1.0,
        };

        if matrix.len() == 16 {
            let texture_scale_offset = TextureScaleOffset::from_rect(texture.use_x(), texture.use_y(), texture.use_w(), texture.use_h(), texture.width(), texture.height());
            self._draw_final(
                device, queue, renderpass, postprocess_pipelines, geometrys, &texture_scale_offset, texture_bind_group, targets, depth_stencil, matrix, SimpleRenderExtendsData { alpha, depth }
            )
        } else {
            Err(EPostprocessError::ParamMatrixSizeError)
        }
    }

    fn _draw_front<'a>(
        &self,
        device: &RenderDevice,
        queue: &wgpu::Queue,
        atlas_allocator: &'a SafeAtlasAllocator,
        encoder: &mut wgpu::CommandEncoder,
        postprocess_pipelines: &PostProcessMaterialMgr,
        geometrys: & PostProcessGeometryManager,
        src: EPostprocessTarget<'a>,
        dst_size: (u32, u32),
        matrix: &[f32],
    ) -> Result<EPostprocessTarget<'a>, EPostprocessError>  {
        let count = self.flags.len();

        if count <= 1 {
            Ok(src)
        } else {
            let format = wgpu::TextureFormat::Rgba8UnormSrgb;
            let mut temp_targets: TemporaryRenderTargets = TemporaryRenderTargets::new(atlas_allocator);
            let mut src_info = (src.use_w(), src.use_h(), 0, format);
            src_info.2 = temp_targets.record_from_other(src);

            let targets = [Some(create_default_target())];
            let primitive: wgpu::PrimitiveState = wgpu::PrimitiveState::default();
            let depth_and_stencil = None;

            for i in 0..count-1 {
                let flag = *self.flags.get(i).unwrap();

                let temp_result = self._draw_single_front(device, queue, encoder, postprocess_pipelines, geometrys, src_info, dst_size, &targets, &primitive, &depth_and_stencil, matrix, flag, &mut temp_targets);

                temp_targets.release(src_info.2);

                match temp_result {
                    Ok(id) => {
                        src_info.2 = id;
                    },
                    Err(e) => {
                        return Err(e);
                    },
                }
            }
        
            let view = temp_targets.get_share_target_view(Some(src_info.2)).unwrap();
            
            temp_targets.reset();

            Ok(EPostprocessTarget::from_share_target(view, format))
        }
    }

    fn _draw_final<'a>(
        &'a self,
        device: & RenderDevice,
        queue: & wgpu::Queue,
        renderpass: & mut wgpu::RenderPass<'a>,
        postprocess_pipelines: &'a PostProcessMaterialMgr,
        geometrys: &'a PostProcessGeometryManager,
        texture_scale_offset: &TextureScaleOffset,
        texture_bind_group: &'a wgpu::BindGroup,
        targets: &[Option<wgpu::ColorTargetState>],
        depth_stencil: &Option<wgpu::DepthStencilState>,
        matrix: & [f32],
        extends: SimpleRenderExtendsData,
    ) -> Result<bool, EPostprocessError> {
        let count = self.flags.len();
        if count > 0 {
            let flag = *self.flags.get(count - 1).unwrap();
            self._draw_single_simple(device, queue, renderpass, postprocess_pipelines, geometrys, texture_scale_offset, texture_bind_group, targets[0].as_ref().unwrap(), depth_stencil, matrix, extends, flag);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn _draw_single_front<'a>(
        &self,
        device: & RenderDevice,
        queue: & wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        postprocess_pipelines: &PostProcessMaterialMgr,
        geometrys: &PostProcessGeometryManager,
        src: (u32, u32, usize, wgpu::TextureFormat),
        dst: (u32, u32),
        targets: &[Option<wgpu::ColorTargetState>],
        primitive: &wgpu::PrimitiveState,
        depth_stencil: &Option<wgpu::DepthStencilState>,
        matrix: & [f32],
        flag: EPostprocessRenderType,
        temp_targets: & mut TemporaryRenderTargets<'a>,
    ) -> Result<usize, EPostprocessError> {
        let (_, _, src_id, format) = src;
        let (width, height) = dst;
        let dst_id = temp_targets.create_share_target(Some(src_id), width, height, format);

        let result = self._draw_single(device, queue, encoder, postprocess_pipelines, geometrys, src, (width, height, dst_id, format), targets, primitive, depth_stencil, matrix, flag, temp_targets, SimpleRenderExtendsData::default());

        match result {
            Ok(_) => Ok(dst_id),
            Err(e) => Err(e),
        }
    }

    fn _draw_single<'a>(
        &'a self,
        device: & RenderDevice,
        queue: & wgpu::Queue,
        encoder: &'a mut wgpu::CommandEncoder,
        postprocess_pipelines: &'a PostProcessMaterialMgr,
        geometrys: &'a PostProcessGeometryManager,
        src: (u32, u32, usize, wgpu::TextureFormat),
        dst: (u32, u32, usize, wgpu::TextureFormat),
        targets: &[Option<wgpu::ColorTargetState>],
        primitive: &wgpu::PrimitiveState,
        depth_stencil: &Option<wgpu::DepthStencilState>,
        matrix: & [f32],
        flag: EPostprocessRenderType,
        temp_targets: & mut TemporaryRenderTargets<'_>,
        extends: SimpleRenderExtendsData,
    ) -> Result<(), EPostprocessError> {
        let ( _, _, src_id, _) = src;
        let ( _, _, dst_id, _) = dst;
        match flag {
            EPostprocessRenderType::BlurDual => {
                let geometry = geometrys.get_geometry();
                let blur_dual_result = blur_dual_render(self.blur_dual.as_ref().unwrap(), device, queue, encoder, postprocess_pipelines, self.renders.blur_dual.as_ref().unwrap(), geometry,src, dst, &IDENTITY_MATRIX, SimpleRenderExtendsData::default(), temp_targets);
                if let Err(e) = blur_dual_result {
                    return Err(e);
                };
            },
            EPostprocessRenderType::BloomDual => {
                let geometry = geometrys.get_geometry();
                let bloom_dual_result = bloom_dual_render(self.bloom_dual.as_ref().unwrap(), device, queue, encoder, postprocess_pipelines,  self.renders.bloom_dual.as_ref().unwrap(), geometry, src, dst, &IDENTITY_MATRIX, SimpleRenderExtendsData::default(), temp_targets);
                if let Err(e) = bloom_dual_result {
                    return Err(e);
                };
            },
            EPostprocessRenderType::HorizonGlitch => {
                let geometry = geometrys.get_geometry();
                let src = temp_targets.get_target(src_id).unwrap();
                let dst = temp_targets.get_target(dst_id).unwrap();
                horizon_glitch_render(self.horizon_glitch.as_ref().unwrap(), device, queue, encoder, postprocess_pipelines, self.renders.horizon_glitch.as_ref().unwrap(), geometry, geometrys.get_glitch_geometry(), src, dst, &IDENTITY_MATRIX, SimpleRenderExtendsData::default());
            },
            _ => {
                let src = temp_targets.get_target(src_id).unwrap();
                let receiver = temp_targets.get_target(dst_id).unwrap();

                let texture_scale_offset: TextureScaleOffset = TextureScaleOffset::from_rect(src.use_x(), src.use_y(), src.use_w(), src.use_h(), src.width(), src.height());
                let texture_bind_group = self.get_texture_bind_group(device, postprocess_pipelines, src, targets[0].as_ref().unwrap(), primitive, depth_stencil, flag).unwrap();
                let mut renderpass = encoder.begin_render_pass(
                    &wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[
                            Some(wgpu::RenderPassColorAttachment {
                                view: receiver.view(),
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
                renderpass.set_viewport(
                    receiver.use_x() as f32,
                    receiver.use_y() as f32,
                    receiver.use_w() as f32,
                    receiver.use_h() as f32,
                    0.,
                    1.
                );
                renderpass.set_scissor_rect(
                    receiver.use_x(),
                    receiver.use_y(),
                    receiver.use_w(),
                    receiver.use_h(),
                );
                self._draw_single_simple(device, queue, &mut renderpass, postprocess_pipelines, geometrys, &texture_scale_offset, &texture_bind_group, targets[0].as_ref().unwrap(), depth_stencil, matrix, extends, flag);
            },
        }
        Ok(())
    }

    fn _draw_single_simple<'a>(
        &'a self,
        device: & RenderDevice,
        queue: & wgpu::Queue,
        renderpass: & mut wgpu::RenderPass<'a>,
        postprocess_pipelines: &'a PostProcessMaterialMgr,
        geometrys: &'a PostProcessGeometryManager,
        texture_scale_offset: & TextureScaleOffset,
        texture_bind_group: &'a wgpu::BindGroup,
        target: &wgpu::ColorTargetState,
        depth_stencil: &Option<wgpu::DepthStencilState>,
        matrix: & [f32],
        extends: SimpleRenderExtendsData,
        flag: EPostprocessRenderType,
    ) {
        let geometry = geometrys.get_geometry();

        match flag {
            EPostprocessRenderType::ColorEffect => {
                color_effect_render(&self.hsb, &self.color_balance, &self.color_scale, &self.vignette, &self.color_filter,  device, queue, renderpass, postprocess_pipelines, &self.renders.color_effect.as_ref().unwrap(), geometry, texture_scale_offset, texture_bind_group, target, depth_stencil, matrix, extends);
            },
            EPostprocessRenderType::BlurDirect => {
                blur_direct_render(self.blur_direct.as_ref().unwrap(), device, queue, renderpass, postprocess_pipelines, self.renders.blur_direct.as_ref().unwrap(), geometry, texture_scale_offset, texture_bind_group, target, depth_stencil, matrix, extends);
            },
            EPostprocessRenderType::BlurRadial => {
                blur_radial_render(self.blur_radial.as_ref().unwrap(), device, queue, renderpass, postprocess_pipelines, self.renders.blur_radial.as_ref().unwrap(), geometry, texture_scale_offset, texture_bind_group, target, depth_stencil, matrix, extends);
            },
            EPostprocessRenderType::BlurBokeh => {
                blur_bokeh_render(self.blur_bokeh.as_ref().unwrap(), device, queue, renderpass, postprocess_pipelines,  self.renders.blur_bokeh.as_ref().unwrap(), geometry, texture_scale_offset, texture_bind_group, target, depth_stencil, matrix, extends);
            },
            EPostprocessRenderType::RadialWave => {
                radial_wave_render(self.radial_wave.as_ref().unwrap(), device, queue, renderpass, postprocess_pipelines, self.renders.radial_wave.as_ref().unwrap(), geometry, texture_scale_offset, texture_bind_group, target, depth_stencil, matrix, extends);
            },
            EPostprocessRenderType::FilterSobel => {
                filter_sobel_render(self.filter_sobel.as_ref().unwrap(), device, queue, renderpass, postprocess_pipelines, self.renders.filter_sobel.as_ref().unwrap(), geometry, texture_scale_offset, texture_bind_group, target, depth_stencil, matrix, extends);
            },
            EPostprocessRenderType::CopyIntensity => {
                copy_intensity_render(self.copy.as_ref().unwrap(), device, queue, renderpass, postprocess_pipelines, self.renders.copy_intensity.as_ref().unwrap(), geometry, texture_scale_offset, texture_bind_group, target, depth_stencil, matrix, extends);
            },
            EPostprocessRenderType::FinalCopyIntensity => {
                copy_intensity_render(&CopyIntensity::default(), device, queue, renderpass, postprocess_pipelines, self.renders.final_copy_renderer.as_ref().unwrap(), geometry, texture_scale_offset, texture_bind_group, target, depth_stencil, matrix, extends);
            },
            _ => {

            }
        }
    }

    fn check(
        &mut self,
        render_device: &RenderDevice,
        delta_time: u64,
        geometrys: &mut PostProcessGeometryManager,
        postprocess_pipelines: &mut PostProcessMaterialMgr,
        targets: &[Option<wgpu::ColorTargetState>],
        depth_stencil: Option<wgpu::DepthStencilState>,
        final_step_by_draw_final: bool,
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
        
        let primitive: wgpu::PrimitiveState = wgpu::PrimitiveState::default();

        self.renders.check_copy_intensity(device, geometrys, postprocess_pipelines, primitive.clone(), targets, depth_stencil.clone());

        let mut final_is_multi_render_steps = false;

        if color_effect {
            self.renders.check_color_effect(device, geometrys, postprocess_pipelines, primitive.clone(), targets, depth_stencil.clone());
            self.flags.push(EPostprocessRenderType::ColorEffect);
            final_is_multi_render_steps = false;
        }
        if blur_dual {
            self.renders.check_blur_dual(device, geometrys, postprocess_pipelines, primitive.clone(), targets, depth_stencil.clone());
            self.flags.push(EPostprocessRenderType::BlurDual);
            final_is_multi_render_steps = true;
        }
        if blur_direct {
            self.renders.check_blur_direct(device, geometrys, postprocess_pipelines, primitive.clone(), targets, depth_stencil.clone());
            self.flags.push(EPostprocessRenderType::BlurDirect);
            final_is_multi_render_steps = false;
        }
        if blur_radial {
            self.renders.check_blur_radial(device, geometrys, postprocess_pipelines, primitive.clone(), targets, depth_stencil.clone());
            self.flags.push(EPostprocessRenderType::BlurRadial);
            final_is_multi_render_steps = false;
        }
        if blur_bokeh {
            self.renders.check_blur_bokeh(device, geometrys, postprocess_pipelines, primitive.clone(), targets, depth_stencil.clone());
            self.flags.push(EPostprocessRenderType::BlurBokeh);
            final_is_multi_render_steps = false;
        }
        if bloom_dual {
            self.renders.check_bloom_dual(device, geometrys, postprocess_pipelines, primitive.clone(), targets, depth_stencil.clone());
            self.flags.push(EPostprocessRenderType::BloomDual);
            final_is_multi_render_steps = true;
        }
        if radial_wave {
            self.renders.check_radial_wave(device, geometrys, postprocess_pipelines, primitive.clone(), targets, depth_stencil.clone());
            self.flags.push(EPostprocessRenderType::RadialWave);
            final_is_multi_render_steps = false;
        }
        if horizon_glitch {
            self.renders.check_horizon_glitch(device, geometrys, postprocess_pipelines, primitive.clone(), targets, depth_stencil.clone());
            self.horizon_glitch.as_mut().unwrap().update(delta_time);
            self.flags.push(EPostprocessRenderType::HorizonGlitch);
            final_is_multi_render_steps = true;
        }
        if filter_sobel {
            self.renders.check_sobel(device, geometrys, postprocess_pipelines, primitive.clone(), targets, depth_stencil.clone());
            self.flags.push(EPostprocessRenderType::FilterSobel);
            final_is_multi_render_steps = false;
        }
        if copy_intensity {
            self.flags.push(EPostprocessRenderType::CopyIntensity);
            final_is_multi_render_steps = false;
        } else {
            if self.alpha.is_some() {
                self.flags.push(EPostprocessRenderType::FinalCopyIntensity);
                final_is_multi_render_steps = false;
            }
        }

        if final_step_by_draw_final && final_is_multi_render_steps {
            self.flags.push(EPostprocessRenderType::FinalCopyIntensity);
        }
    }
}