

use pi_assets::mgr::AssetMgr;
use pi_render::{components::view::target_alloc::{SafeAtlasAllocator, TargetType}, rhi::{device::{RenderDevice}, pipeline::RenderPipeline, asset::RenderRes, RenderQueue}, renderer::{pipeline::DepthStencilState, vertex_buffer::VertexBufferAllocator}};
use pi_share::Share;

use crate::{
    effect::*,
    temprory_render_target::{PostprocessTexture},
    renderer::{ bloom_dual::bloom_dual_render, horizon_glitch::horizon_glitch_render},
    error::EPostprocessError,
    image_effect::*,
    material::{create_default_target},
    geometry::IDENTITY_MATRIX, postprocess_flags::EPostprocessRenderType, SimpleRenderExtendsData
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
    ) {
        self.check(delta_time, true);
        // println!("{:?}", self.flags);
    }
    /// 对源内容进行后处理 - 最后一个效果的渲染在 draw_final 接口调用
    /// * `src`
    ///   * 源纹理内容
    /// * `dst_size`
    ///   * 接收处理结果的纹理尺寸
    /// * `return`
    ///   * Ok(PostprocessTexture) 执行成功
    ///     * 返回 渲染结果纹理
    ///     * 当实际没有渲染 结果时 会返回 传入的src 对应数据
    ///       * Example: 模糊后处理, 模糊半径为 0 则认为不需要渲染过程, 应当直接使用 src, 返回 false
    ///   * Err(String)
    pub fn draw_front(
        &self,
        device: &RenderDevice,
        queue: &RenderQueue,
        encoder: &mut wgpu::CommandEncoder,
        mut src: PostprocessTexture,
        dst_size: (u32, u32),
        vballocator: &mut VertexBufferAllocator,
        safeatlas: &SafeAtlasAllocator,
        resources: &SingleImageEffectResource,
        pipelines: &Share<AssetMgr<RenderRes<RenderPipeline>>>,
        target_type: TargetType,
    ) -> Result<PostprocessTexture, EPostprocessError> {

        let matrix: &[f32] = &IDENTITY_MATRIX;
        
        if src.use_w() != dst_size.0 || src.use_h() != dst_size.1 {
            let mut templist = vec![];
            let target = if let Some(temp) = src.get_share_target() {
                templist.push(temp);
                let target = safeatlas.allocate(dst_size.0, dst_size.1, target_type, templist.iter());
                PostprocessTexture::from_share_target(target, src.format())
            } else {
                let target = safeatlas.allocate(dst_size.0, dst_size.1, target_type, templist.iter());
                PostprocessTexture::from_share_target(target, src.format())
            };

            let (draw, result) = EffectCopy::ready(
                CopyIntensity::default(), resources, device, queue,
                0, (target.use_w(), target.use_h()),
                &matrix, src.get_tilloff(),
                1., 0.,
                src, Some(target),
                safeatlas, target_type, pipelines,
                create_default_target(), None
            ).unwrap();
            draw.draw(Some(encoder), None);
            src = result;
        }

        let result = self._draw_front(
            device, queue, encoder, src, &IDENTITY_MATRIX, vballocator, safeatlas, resources, pipelines, target_type
        );

        result
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
        queue: & RenderQueue,
        matrix: &[f32],
        depth: f32,
        vballocator: &mut VertexBufferAllocator,
        safeatlas: &SafeAtlasAllocator,
        source: PostprocessTexture,
        target: PostprocessTexture,
        resources: &SingleImageEffectResource,
        pipelines: &Share<AssetMgr<RenderRes<RenderPipeline>>>,
        color_state: wgpu::ColorTargetState,
        depth_stencil: Option<DepthStencilState>,
        target_type: TargetType,
    ) -> Option<PostProcessDraw> {

        if matrix.len() == 16 {
            let count = self.flags.len();
            if count > 0 {
                let extends = if let Some(alpha) = self.alpha {
                    SimpleRenderExtendsData { alpha: alpha.a, depth }
                } else {
                    SimpleRenderExtendsData { alpha: 1., depth }
                };
                let flag = *self.flags.get(count - 1).unwrap();
                let mut draws = vec![];
                self._draw_single_simple(device, queue, None, matrix, extends, flag, vballocator, safeatlas, source, Some(target), &mut draws, resources, pipelines, color_state, depth_stencil, target_type);
    
                // let flag = *self.flags.get(count - 1).unwrap();
                // self._draw_single_simple(device, queue, renderpass, postprocess_pipelines, geometrys, texture_scale_offset, texture_bind_group, targets[0].as_ref().unwrap(), depth_stencil, matrix, extends, flag);
                Some(draws.pop().unwrap())
            } else {
                None
            }
        } else {
            None
        }
    }

    fn _draw_front<'a>(
        &self,
        device: &RenderDevice,
        queue: &RenderQueue,
        encoder: &mut wgpu::CommandEncoder,
        src: PostprocessTexture,
        matrix: &[f32],
        vballocator: &mut VertexBufferAllocator,
        safeatlas: &SafeAtlasAllocator,
        resources: &SingleImageEffectResource,
        pipelines: &Share<AssetMgr<RenderRes<RenderPipeline>>>,
        target_type: TargetType,
    ) -> Result<PostprocessTexture, EPostprocessError>  {
        let count = self.flags.len();

        if count <= 1 {
            Ok(src)
        } else {
            let format = wgpu::TextureFormat::Rgba8UnormSrgb;
            let mut temp_result = src;
            for i in 0..count-1 {
                let flag = *self.flags.get(i).unwrap();

                let mut draws = vec![];
                temp_result = self._draw_single_front(
                    device, queue, encoder,
                    &IDENTITY_MATRIX, flag,
                    vballocator, safeatlas, temp_result, None, &mut draws, resources, pipelines, None, target_type
                );
                draws.iter().for_each(|v| {
                    v.draw(Some(encoder), None);
                });
            }

            Ok(temp_result)
        }
    }

    fn _draw_single_front<'a>(
        &self,
        device: & RenderDevice,
        queue: & RenderQueue,
        encoder: &mut wgpu::CommandEncoder,
        matrix: & [f32],
        flag: EPostprocessRenderType,
        vballocator: &mut VertexBufferAllocator,
        safeatlas: &SafeAtlasAllocator,
        source: PostprocessTexture,
        target: Option<PostprocessTexture>,
        draws: &mut Vec<PostProcessDraw>,
        resources: &SingleImageEffectResource,
        pipelines: &Share<AssetMgr<RenderRes<RenderPipeline>>>,
        depth_stencil: Option<DepthStencilState>,
        target_type: TargetType,
    ) -> PostprocessTexture {

        self._draw_single_simple(device, queue, Some(encoder), matrix, SimpleRenderExtendsData::default(), flag, vballocator, safeatlas, source, target, draws, resources, pipelines, create_default_target(), depth_stencil, target_type)

    }

    fn _draw_single_simple<'a>(
        &'a self,
        device: & RenderDevice,
        queue: & RenderQueue,
        encoder: Option<&mut wgpu::CommandEncoder>,
        matrix: & [f32],
        extends: SimpleRenderExtendsData,
        flag: EPostprocessRenderType,
        vballocator: &mut VertexBufferAllocator,
        safeatlas: &SafeAtlasAllocator,
        source: PostprocessTexture,
        target: Option<PostprocessTexture>,
        draws: &mut Vec<PostProcessDraw>,
        resources: &SingleImageEffectResource,
        pipelines: &Share<AssetMgr<RenderRes<RenderPipeline>>>,
        color_state: wgpu::ColorTargetState,
        depth_stencil: Option<DepthStencilState>,
        target_type: TargetType,
    ) -> PostprocessTexture {
        let dst_size = if let Some(target) = &target {
            (target.use_w(), target.use_h())
        } else {
            (source.use_w(), source.use_h())
        };
        match flag {
            EPostprocessRenderType::ColorEffect => {
                let param = ColorEffect {
                    hsb: self.hsb.clone(),
                    balance: self.color_balance.clone(),
                    vignette: self.vignette.clone(),
                    scale: self.color_scale.clone(),
                    filter: self.color_filter.clone(),
                };
                let (draw, result) = EffectColorEffect::ready(
                    param, resources, device, queue,
                    0, dst_size,
                    &matrix, source.get_tilloff(),
                    extends.alpha, extends.depth,
                    source, target,
                    safeatlas, target_type, pipelines,
                    color_state, depth_stencil
                ).unwrap();
                draws.push(draw);
                return result; 
            },
            EPostprocessRenderType::BlurDirect => {
                let (draw, result) = EffectBlurDirect::ready(
                    self.blur_direct.as_ref().unwrap().clone(), resources, device, queue,
                    0, dst_size,
                    &matrix, source.get_tilloff(),
                    1., 1.,
                    source, target,
                    safeatlas, target_type, pipelines,
                    color_state, depth_stencil
                ).unwrap();
                draws.push(draw);
                return result; 
            },
            EPostprocessRenderType::BlurRadial => {
                let (draw, result) = EffectBlurRadial::ready(
                    self.blur_radial.as_ref().unwrap().clone(), resources, device, queue,
                    0, dst_size,
                    &matrix, source.get_tilloff(),
                    extends.alpha, extends.depth,
                    source, target,
                    safeatlas, target_type, pipelines,
                    color_state, depth_stencil
                ).unwrap();
                draws.push(draw);
                return result; 
            },
            EPostprocessRenderType::BlurBokeh => {
                let (draw, result) = EffectBlurBokeh::ready(
                    self.blur_bokeh.as_ref().unwrap().clone(), resources, device, queue,
                    0, dst_size,
                    &matrix, source.get_tilloff(),
                    extends.alpha, extends.depth,
                    source, target,
                    safeatlas, target_type, pipelines,
                    color_state, depth_stencil
                ).unwrap();
                draws.push(draw);
                return result; 
            },
            EPostprocessRenderType::RadialWave => {
                let (draw, result) = EffectRadialWave::ready(
                    self.radial_wave.as_ref().unwrap().clone(), resources, device, queue,
                    0, dst_size,
                    &matrix, source.get_tilloff(),
                    extends.alpha, extends.depth,
                    source, target,
                    safeatlas, target_type, pipelines,
                    color_state, depth_stencil
                ).unwrap();
                draws.push(draw);
                return result; 
            },
            EPostprocessRenderType::FilterSobel => {
                let (draw, result) = EffectFilterSobel::ready(
                    self.filter_sobel.as_ref().unwrap().clone(), resources, device, queue,
                    0, dst_size,
                    &matrix, source.get_tilloff(),
                    extends.alpha, extends.depth,
                    source, target,
                    safeatlas, target_type, pipelines,
                    color_state, depth_stencil
                ).unwrap();
                draws.push(draw);
                return result; 
            },
            EPostprocessRenderType::CopyIntensity => {
                let (draw, result) = EffectCopy::ready(
                    self.copy.as_ref().unwrap().clone(), resources, device, queue,
                    0, dst_size,
                    &matrix, source.get_tilloff(),
                    extends.alpha, extends.depth,
                    source, target,
                    safeatlas, target_type, pipelines,
                    color_state, depth_stencil
                ).unwrap();
                draws.push(draw);
                return result; 
            },
            EPostprocessRenderType::FinalCopyIntensity => {
                let (draw, result) = EffectCopy::ready(
                    CopyIntensity::default(), resources, device, queue,
                    0, dst_size,
                    &matrix, source.get_tilloff(),
                    extends.alpha, extends.depth,
                    source, target,
                    safeatlas, target_type, pipelines,
                    color_state, depth_stencil
                ).unwrap();
                draws.push(draw);
                return result; 
            },
            EPostprocessRenderType::BlurDual => {
                let mut realiter = 0;
                let blur_dual = self.blur_dual.as_ref().unwrap();
                let fromw = dst_size.0;
                let fromh = dst_size.1;
                let mut tow = fromw;
                let mut toh = fromh;
                let mut tempresult = source.clone();
                for i in 0..blur_dual.iteration {
                    if tow / 2 >= 2 && toh / 2 >= 2 {
                        tow = tow / 2;
                        toh = toh / 2;
                        realiter += 1;
                        let param = BlurDualForBuffer { param: blur_dual.clone(), isup: false };
                        let (draw, result) = EffectBlurDual::ready(
                            param, resources, device, queue,
                            0, (tow, toh),
                            &matrix, tempresult.get_tilloff(),
                            1., 0.,
                            tempresult, None,
                            safeatlas, target_type, pipelines,
                            create_default_target(), None
                        ).unwrap();
                        draws.push(draw);
                        tempresult = result;
                        // log::warn!("Down: {:?}", (tow, toh));
                    }
                }

                if realiter >= 1 {
                    for i in 1..realiter {
                        tow = tow * 2;
                        toh = toh * 2;
                        let param = BlurDualForBuffer { param: blur_dual.clone(), isup: true };
                        let (draw, result) = EffectBlurDual::ready(
                            param, resources, device, queue,
                            0, (tow, toh),
                            &matrix, tempresult.get_tilloff(),
                            1., 0.,
                            tempresult, None,
                            safeatlas, target_type, pipelines,
                            create_default_target(), None
                        ).unwrap();
                        draws.push(draw);
                        tempresult = result;
                        // log::warn!("Up: {:?}", (tow, toh));
                    }
                }

                tow = fromw;
                toh = fromh;
                let param = BlurDualForBuffer { param: blur_dual.clone(), isup: true };
                let (draw, result) = EffectBlurDual::ready(
                    param, resources, device, queue,
                    0, (tow, toh),
                    &matrix, tempresult.get_tilloff(),
                    extends.alpha, extends.depth,
                    tempresult, target,
                    safeatlas, target_type, pipelines,
                    color_state, depth_stencil
                ).unwrap();
                draws.push(draw);
                // log::warn!("Final: {:?}", (tow, toh));
                // log::warn!("Final {:?}", result.get_rect());

                return result; 
            },
            EPostprocessRenderType::BloomDual => {
                if let Some(encoder) = encoder {
                    bloom_dual_render(
                        self.bloom_dual.as_ref().unwrap(),
                        device, queue, encoder, matrix, extends,
                        safeatlas, source, draws, resources, pipelines, depth_stencil, target_type
                    )
                } else {
                    source
                }
            },
            EPostprocessRenderType::HorizonGlitch => {
                // log::warn!("HorizonGlitch {:?}", source.get_rect());
                horizon_glitch_render(
                    self.horizon_glitch.as_ref().unwrap(),
                    device, queue, vballocator, matrix,
                    safeatlas, source, target, draws, resources, pipelines, color_state, depth_stencil, target_type
                )
            },
        }
    }

    fn check(
        &mut self,
        delta_time: u64,
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

        let mut final_is_multi_render_steps = false;

        if color_effect {            self.flags.push(EPostprocessRenderType::ColorEffect);
            final_is_multi_render_steps = false;
        }
        if blur_dual {
            self.flags.push(EPostprocessRenderType::BlurDual);
            final_is_multi_render_steps = true;
        }
        if blur_direct {
            self.flags.push(EPostprocessRenderType::BlurDirect);
            final_is_multi_render_steps = false;
        }
        if blur_radial {
            self.flags.push(EPostprocessRenderType::BlurRadial);
            final_is_multi_render_steps = false;
        }
        if blur_bokeh {
            self.flags.push(EPostprocessRenderType::BlurBokeh);
            final_is_multi_render_steps = false;
        }
        if bloom_dual {
            self.flags.push(EPostprocessRenderType::BloomDual);
            final_is_multi_render_steps = true;
        }
        if radial_wave {
            self.flags.push(EPostprocessRenderType::RadialWave);
            final_is_multi_render_steps = false;
        }
        if horizon_glitch {
            self.horizon_glitch.as_mut().unwrap().update(delta_time);
            self.flags.push(EPostprocessRenderType::HorizonGlitch);
            final_is_multi_render_steps = true;
        }
        if filter_sobel {
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