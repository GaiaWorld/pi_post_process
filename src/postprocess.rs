

use std::fmt::Debug;

use pi_assets::mgr::AssetMgr;
use pi_render::{
    components::view::target_alloc::{SafeAtlasAllocator, TargetType},
    rhi::{device::RenderDevice, pipeline::RenderPipeline, asset::RenderRes, RenderQueue},
    renderer::{
        pipeline::DepthStencilState,
        vertices::RenderVertices,
        draw_obj::DrawObj
    }
};
use pi_share::Share;

use crate::{
    effect::*,
    temprory_render_target::PostprocessTexture,
    renderer::{ bloom_dual::bloom_dual_render, horizon_glitch::{horizon_glitch_render, horizon_glitch_render_calc}, blur_gauss::blur_gauss_render},
    error::EPostprocessError,
    image_effect::*,
    material::create_default_target,
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
    pub blur_gauss:         Option<BlurGauss>,
    
    pub bloom_dual:         Option<BloomDual>,

    pub radial_wave:        Option<RadialWave>,
    pub filter_sobel:       Option<FilterSobel>,
    pub horizon_glitch:     Option<HorizonGlitch>,
    pub image_mask:         Option<ImageMask>,
    pub clip_sdf:           Option<ClipSdf>,

    pub flags:              Vec<EPostprocessRenderType>,
    /// 源内容是否为预乘内容
    pub src_preimultiplied: bool,
    horizon_glitch_instance:Option<RenderVertices>,

    pub(crate) renderer_copy: Option<CopyIntensityRenderer>,
    pub(crate) renderer_copyintensity: Option<CopyIntensityRenderer>,
    pub(crate) renderer_coloreffect: Option<ColorEffectRenderer>,
    pub(crate) renderer_blur_dual: Option<BlurDualRendererList>,
    pub(crate) renderer_blur_direct: Option<BlurDirectRenderer>,
    pub(crate) renderer_blur_radial: Option<BlurRadialRenderer>,
    pub(crate) renderer_blur_bokeh: Option<BlurBokehRenderer>,
    pub(crate) renderer_blur_gauss: Option<(BlurGaussRenderer, BlurGaussRenderer)>,
    pub(crate) renderer_bloom_dual: Option<BloomDualRenderer>,
    pub(crate) renderer_radial_wave: Option<RadialWaveRenderer>,
    pub(crate) renderer_filter_sobel: Option<FilterSobelRenderer>,
    pub(crate) renderer_horizon_glitch: Option<HorizonGlitchRenderer>,
    pub(crate) renderer_image_mask: Option<ImageMaskRenderer>,
    pub(crate) renderer_clip_sdf: Option<ClipSdfRenderer>,
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
            blur_gauss:         None,

            bloom_dual:         None,

            radial_wave:        None,
            filter_sobel:       None,
            horizon_glitch:     None,
            image_mask:         None,
            clip_sdf:           None,

            flags:              vec![],
            src_preimultiplied:  true,
            horizon_glitch_instance: None,
            
            renderer_copy: None,
            renderer_copyintensity: None,
            renderer_coloreffect: None,
            renderer_blur_dual: None,
            renderer_blur_direct: None,
            renderer_blur_radial: None,
            renderer_blur_bokeh: None,
            renderer_blur_gauss: None,
            renderer_bloom_dual: None,
            renderer_radial_wave: None,
            renderer_filter_sobel: None,
            renderer_horizon_glitch: None,
            renderer_image_mask: None,
            renderer_clip_sdf: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ETarget {
    Temp(u32, u32),
    Final(u32, u32),
}
impl ETarget {
    pub fn size(&self) -> (u32, u32) {
        match self {
            ETarget::Temp(w, h) => (*w, *h),
            ETarget::Final(w, h) => (*w, *h),
        }
    }
}

pub struct TempResult {
    finaldraw: Option<DrawObj>,
    // draw: Option<PostProcessDraw>,
    target: Option<PostprocessTexture>,
}
// impl TempResult {
//     fn target(&mut self) -> Option<PostprocessTexture> {
//         replace(&mut self.target, None) 
//     }
// }

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
        device: &RenderDevice,
        queue: &RenderQueue,
        src: PostprocessTexture,
        _dst_size: (u32, u32),
        safeatlas: &SafeAtlasAllocator,
        resources: &SingleImageEffectResource,
        pipelines: &Share<AssetMgr<RenderRes<RenderPipeline>>>,
        target_type: TargetType,
        target_format: wgpu::TextureFormat,
    ) -> Result<(Vec<PostProcessDraw>, PostprocessTexture), EPostprocessError> {
        if self.renderer_copy.is_none() {
            self.renderer_copy = Some(CopyIntensityRenderer { param: CopyIntensity::default(), uniform: resources.uniform_buffer() })
        }
        self.check(delta_time, true, device, queue, resources);

        // let matrix: &[f32] = &IDENTITY_MATRIX;

        let drawlist: Vec<PostProcessDraw> = vec![];
        
        let src_premultiplied: bool = self.src_preimultiplied;
        // if src.use_w() != dst_size.0 || src.use_h() != dst_size.1 {
        //     let mut result_use_once_innext = true;
        //     if 0 < self.flags.len() {
        //         let next = *self.flags.get(0).unwrap();
        //         match next {
        //             EPostprocessRenderType::HorizonGlitch => result_use_once_innext = false,
        //             EPostprocessRenderType::BloomDual => result_use_once_innext = false,
        //             _ => result_use_once_innext = true,
        //         }
        //     }

        //     let result = EffectCopy::get_target(None, &src, dst_size, safeatlas, target_type, target_format, result_use_once_innext);

        //     let draw = EffectCopy::ready(
        //         self.renderer_copy.as_ref().unwrap(), resources, device, queue,
        //         0, dst_size,
        //         &matrix,
        //         1., 0.,
        //         &src,
        //         safeatlas, target_type, pipelines,
        //         create_default_target(target_format), None, false,
        //         src_premultiplied, false
        //     ).unwrap();
            
        //     log::warn!("First {:?}", result.get_rect());

        //     src_premultiplied = false;
            
        //     let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
        //     drawlist.push(draw);
        //     src = result;
        // }

        let result = self._draw_front(
            device, queue, src, &IDENTITY_MATRIX, safeatlas, resources, pipelines, target_type, target_format, src_premultiplied, drawlist
        );
        
        result
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
        encoder: &mut wgpu::CommandEncoder,
        draws: &Vec<PostProcessDraw>,
    ) {
        draws.iter().for_each(|draw| {
            draw.draw(Some(encoder), None);
        });
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
        safeatlas: &SafeAtlasAllocator,
        source: &PostprocessTexture,
        target_size: (u32, u32),
        resources: &SingleImageEffectResource,
        pipelines: &Share<AssetMgr<RenderRes<RenderPipeline>>>,
        color_state: wgpu::ColorTargetState,
        depth_stencil: Option<DepthStencilState>,
        target_type: TargetType,
        target_format: wgpu::TextureFormat,
    ) -> Option<DrawObj> {

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
                let mut tempresult = TempResult { target: None, finaldraw: None };
                let src_premultiplied = if count == 1 { self.src_preimultiplied } else { false };
                let dst_premultiply = self.src_preimultiplied;
                self._draw_single_simple(device, queue, matrix, extends, flag, safeatlas, source, ETarget::Final(target_size.0, target_size.1), &mut draws, resources, pipelines, color_state, depth_stencil, target_type, target_format, &mut tempresult, src_premultiplied, dst_premultiply, true);

                if let Some(finaldraw) = tempresult.finaldraw {
                    Some(finaldraw)
                } else {
                    None
                }
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
        src: PostprocessTexture,
        _: &[f32],
        safeatlas: &SafeAtlasAllocator,
        resources: &SingleImageEffectResource,
        pipelines: &Share<AssetMgr<RenderRes<RenderPipeline>>>,
        target_type: TargetType,
        target_format: wgpu::TextureFormat,
        mut src_premultiplied: bool,
        mut drawlist: Vec<PostProcessDraw>,
    ) -> Result<(Vec<PostProcessDraw>, PostprocessTexture), EPostprocessError>  {
        let count = self.flags.len();

        if count <= 1 {
            Ok((drawlist, src))
        } else {
            let mut source = src;
            let target = ETarget::Temp(source.use_w(), source.use_h());
            // let format = wgpu::TextureFormat::Rgba8UnormSrgb;
            for i in 0..count-1 {
                let flag = *self.flags.get(i).unwrap();

                let mut temp_result = TempResult { target: None, finaldraw: None };

                let mut result_use_once_innext = true;
                if i < count - 1 {
                    let next = *self.flags.get(i+1).unwrap();
                    match next {
                        EPostprocessRenderType::HorizonGlitch => result_use_once_innext = false,
                        EPostprocessRenderType::BloomDual => result_use_once_innext = false,
                        _ => result_use_once_innext = true,
                    }
                }

                if i > 0 { src_premultiplied = false; }

                self._draw_single_simple(
                    device, queue,
                    &IDENTITY_MATRIX, SimpleRenderExtendsData::default(), flag, safeatlas,
                    &source, target,
                    &mut drawlist, resources, pipelines,
                    create_default_target(target_format), None, target_type, target_format, &mut temp_result, src_premultiplied, false, result_use_once_innext
                );
                source = temp_result.target.unwrap();
                temp_result.target = None;
            }

            Ok((drawlist, source))
        }
    }

    fn _draw_single_simple<'a>(
        &'a self,
        device: & RenderDevice,
        queue: & RenderQueue,
        matrix: & [f32],
        extends: SimpleRenderExtendsData,
        flag: EPostprocessRenderType,
        safeatlas: &SafeAtlasAllocator,
        source: &PostprocessTexture,
        target: ETarget,
        draws: &mut Vec<PostProcessDraw>,
        resources: &SingleImageEffectResource,
        pipelines: &Share<AssetMgr<RenderRes<RenderPipeline>>>,
        color_state: wgpu::ColorTargetState,
        depth_stencil: Option<DepthStencilState>,
        target_type: TargetType,
        target_format: wgpu::TextureFormat,
        temp_result: &mut TempResult,
        mut src_premultiplied: bool,
        dst_premultiply: bool,
        result_target_useonce: bool
    ) {
        let dst_size = target.size();
        let force_nearest_filter = source.size_eq_2(&dst_size);
        match flag {
            EPostprocessRenderType::ColorEffect => {
                let param = self.renderer_coloreffect.as_ref().unwrap();
                match target {
                    ETarget::Temp(_, _) => {
                        let result = EffectCopy::get_target(None, &source, dst_size, safeatlas, target_type, target_format, result_target_useonce); 
                        let draw = EffectColorEffect::ready(
                            param, resources, device, queue, 0, dst_size, &matrix, extends.alpha, extends.depth,
                            source, safeatlas, target_type, pipelines, color_state, depth_stencil, force_nearest_filter, src_premultiplied, dst_premultiply
                        ).unwrap();
                        let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
                        draws.push(draw);
                        temp_result.target = Some(result);
                    },
                    ETarget::Final(_, _) => {
                        let draw = EffectColorEffect::ready(
                            param, resources, device, queue, 0, dst_size, &matrix, extends.alpha, extends.depth,
                            source, safeatlas, target_type, pipelines, color_state, depth_stencil, force_nearest_filter, src_premultiplied, dst_premultiply
                        ).unwrap();
                        temp_result.finaldraw = Some(draw);
                    },
                }
            },
            EPostprocessRenderType::BlurDirect => {
                let param = self.renderer_blur_direct.as_ref().unwrap();
                match target {
                    ETarget::Temp(_, _) => {
                        let result = EffectCopy::get_target(None, &source, dst_size, safeatlas, target_type, target_format, result_target_useonce); 
                        let draw = EffectBlurDirect::ready(
                            param, resources, device, queue,
                            0, dst_size, &matrix, 1., 1., source, safeatlas, target_type, pipelines, color_state, depth_stencil, src_premultiplied, dst_premultiply
                        ).unwrap();
                        let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
                        draws.push(draw);
                        temp_result.target = Some(result);
                    },
                    ETarget::Final(_, _) => {
                        let draw = EffectBlurDirect::ready(
                            param, resources, device, queue,
                            0, dst_size, &matrix, 1., 1., source, safeatlas, target_type, pipelines, color_state, depth_stencil, src_premultiplied, dst_premultiply
                        ).unwrap();
                        temp_result.finaldraw = Some(draw);
                    },
                }
            },
            EPostprocessRenderType::BlurRadial => {
                let param = self.renderer_blur_radial.as_ref().unwrap();
                match target {
                    ETarget::Temp(_, _) => {
                        let result = EffectCopy::get_target(None, &source, dst_size, safeatlas, target_type, target_format, result_target_useonce); 
                        let draw = EffectBlurRadial::ready(
                            param, resources, device, queue,
                            0, dst_size, &matrix, extends.alpha, extends.depth, source, safeatlas, target_type, pipelines,  color_state, depth_stencil, src_premultiplied, dst_premultiply
                        ).unwrap();
                        let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
                        draws.push(draw);
                        temp_result.target = Some(result);
                    },
                    ETarget::Final(_, _) => {
                        let draw = EffectBlurRadial::ready(
                            param, resources, device, queue,
                            0, dst_size, &matrix, extends.alpha, extends.depth, source, safeatlas, target_type, pipelines,  color_state, depth_stencil, src_premultiplied, dst_premultiply
                        ).unwrap();
                        temp_result.finaldraw = Some(draw);
                    },
                }
            },
            EPostprocessRenderType::BlurBokeh => {
                let param = self.renderer_blur_bokeh.as_ref().unwrap();
                match target {
                    ETarget::Temp(_, _) => {
                        let result = EffectCopy::get_target(None, &source, dst_size, safeatlas, target_type, target_format, result_target_useonce); 
                        let draw = EffectBlurBokeh::ready(
                            param, resources, device, queue,
                            0, dst_size, &matrix, extends.alpha, extends.depth, &source, safeatlas, target_type, pipelines, color_state, depth_stencil, src_premultiplied, dst_premultiply
                        ).unwrap();
                        let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
                        draws.push(draw);
                        temp_result.target = Some(result);
                    },
                    ETarget::Final(_, _) => {
                        let draw = EffectBlurBokeh::ready(
                            param, resources, device, queue,
                            0, dst_size, &matrix, extends.alpha, extends.depth, &source, safeatlas, target_type, pipelines, color_state, depth_stencil, src_premultiplied, dst_premultiply
                        ).unwrap();
                        temp_result.finaldraw = Some(draw);
                    },
                }
            },
            EPostprocessRenderType::RadialWave => {
                let param = self.renderer_radial_wave.as_ref().unwrap();
                match target {
                    ETarget::Temp(_, _) => {
                        let result = EffectCopy::get_target(None, &source, dst_size, safeatlas, target_type, target_format, result_target_useonce); 
                        let draw = EffectRadialWave::ready(
                            param, resources, device, queue,
                            0, dst_size, &matrix,  extends.alpha, extends.depth, source, safeatlas, target_type, pipelines, color_state, depth_stencil, src_premultiplied, dst_premultiply
                        ).unwrap();
                        let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
                        draws.push(draw);
                        temp_result.target = Some(result);
                    },
                    ETarget::Final(_, _) => {
                        let draw = EffectRadialWave::ready(
                            param, resources, device, queue,
                            0, dst_size, &matrix,  extends.alpha, extends.depth, source, safeatlas, target_type, pipelines, color_state, depth_stencil, src_premultiplied, dst_premultiply
                        ).unwrap();
                        temp_result.finaldraw = Some(draw);
                    },
                }
            },
            EPostprocessRenderType::FilterSobel => {
                let param = self.renderer_filter_sobel.as_ref().unwrap();
                match target {
                    ETarget::Temp(_, _) => {
                        let result = EffectCopy::get_target(None, &source, dst_size, safeatlas, target_type, target_format, result_target_useonce); 
                        let draw = EffectFilterSobel::ready(
                            param, resources, device, queue,
                            0, dst_size, &matrix,  extends.alpha, extends.depth, source, safeatlas, target_type, pipelines, color_state, depth_stencil, src_premultiplied, dst_premultiply
                        ).unwrap();
                        let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
                        draws.push(draw);
                        temp_result.target = Some(result);
                    },
                    ETarget::Final(_, _) => {
                        let draw = EffectFilterSobel::ready(
                            param, resources, device, queue,
                            0, dst_size, &matrix,  extends.alpha, extends.depth, source, safeatlas, target_type, pipelines, color_state, depth_stencil, src_premultiplied, dst_premultiply
                        ).unwrap();
                        temp_result.finaldraw = Some(draw);
                    },
                }
            },
            EPostprocessRenderType::CopyIntensity => {
                let param = self.renderer_copyintensity.as_ref().unwrap();
                match target {
                    ETarget::Temp(_, _) => {
                        let result = EffectCopy::get_target(None, &source, dst_size, safeatlas, target_type, target_format, result_target_useonce); 
                        let draw = EffectCopy::ready(
                            param, resources, device, queue,
                            0, dst_size, &matrix,  extends.alpha, extends.depth, source, safeatlas, target_type, pipelines, color_state, depth_stencil, force_nearest_filter, src_premultiplied, dst_premultiply
                        ).unwrap();
                        let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
                        draws.push(draw);
                        temp_result.target = Some(result);
                    },
                    ETarget::Final(_, _) => {
                        let draw = EffectCopy::ready(
                            param, resources, device, queue,
                            0, dst_size, &matrix,  extends.alpha, extends.depth, source, safeatlas, target_type, pipelines, color_state, depth_stencil, force_nearest_filter, src_premultiplied, dst_premultiply
                        ).unwrap();
                        temp_result.finaldraw = Some(draw);
                    },
                }
            },
            EPostprocessRenderType::FinalCopyIntensity => {
                let param = self.renderer_copy.as_ref().unwrap();
                match target {
                    ETarget::Temp(_, _) => {
                        let result = EffectCopy::get_target(None, &source, dst_size, safeatlas, target_type, target_format, result_target_useonce); 
                        let draw = EffectCopy::ready(
                            param, resources, device, queue,
                            0, dst_size, &matrix,  extends.alpha, extends.depth, source, safeatlas, target_type, pipelines, color_state, depth_stencil, force_nearest_filter, src_premultiplied, dst_premultiply
                        ).unwrap();
                        let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
                        draws.push(draw);
                        temp_result.target = Some(result);
                    },
                    ETarget::Final(_, _) => {
                        let draw = EffectCopy::ready(
                            param, resources, device, queue,
                            0, dst_size, &matrix,  extends.alpha, extends.depth, source, safeatlas, target_type, pipelines, color_state, depth_stencil, force_nearest_filter, src_premultiplied, dst_premultiply
                        ).unwrap();
                        temp_result.finaldraw = Some(draw);
                    },
                }
            },
            EPostprocessRenderType::ImageMask => {
                let param = self.renderer_image_mask.as_ref().unwrap();
                match target {
                    ETarget::Temp(_, _) => {
                        let result = EffectImageMask::get_target(None, &source, dst_size, safeatlas, target_type, target_format, result_target_useonce); 
                        let draw = EffectImageMask::ready(
                            param, resources, device, queue,
                            0, dst_size, &matrix,  extends.alpha, extends.depth, source, safeatlas, target_type, pipelines, color_state, depth_stencil, force_nearest_filter, src_premultiplied, dst_premultiply
                        ).unwrap();
                        let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
                        draws.push(draw);
                        temp_result.target = Some(result);
                    },
                    ETarget::Final(_, _) => {
                        let draw = EffectImageMask::ready(
                            param, resources, device, queue,
                            0, dst_size, &matrix,  extends.alpha, extends.depth, source, safeatlas, target_type, pipelines, color_state, depth_stencil, force_nearest_filter, src_premultiplied, dst_premultiply
                        ).unwrap();
                        temp_result.finaldraw = Some(draw);
                    },
                }
            },
            EPostprocessRenderType::ClipSdf => {
                let param = self.renderer_clip_sdf.as_ref().unwrap();
                match target {
                    ETarget::Temp(_, _) => {
                        let result = EffectClipSdf::get_target(None, &source, dst_size, safeatlas, target_type, target_format, result_target_useonce); 
                        let draw = EffectClipSdf::ready(
                            param, resources, device, queue,
                            0, dst_size, &matrix,  extends.alpha, extends.depth, source, safeatlas, target_type, pipelines, color_state, depth_stencil, force_nearest_filter, src_premultiplied, dst_premultiply
                        ).unwrap();
                        let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
                        draws.push(draw);
                        temp_result.target = Some(result);
                    },
                    ETarget::Final(_, _) => {
                        let draw = EffectClipSdf::ready(
                            param, resources, device, queue,
                            0, dst_size, &matrix,  extends.alpha, extends.depth, source, safeatlas, target_type, pipelines, color_state, depth_stencil, force_nearest_filter, src_premultiplied, dst_premultiply
                        ).unwrap();
                        temp_result.finaldraw = Some(draw);
                    },
                }
            },
            EPostprocessRenderType::BlurDual => {
                let param = self.renderer_blur_dual.as_ref().unwrap();
                match target {
                    ETarget::Temp(_, _) => {
                        let mut realiter = 0;
                        let _blur_dual = self.blur_dual.as_ref().unwrap();
                        let fromw = dst_size.0;
                        let fromh = dst_size.1;
                        let mut tow = fromw;
                        let mut toh = fromh;
                        let mut tempresult = source.clone();
                        for idx in 0..param.iteration {
                            if tow / 2 >= 2 && toh / 2 >= 2 {
                                tow = tow / 2;
                                toh = toh / 2;
                                realiter += 1;
                                let result = EffectCopy::get_target(None, &tempresult, (tow, toh), safeatlas, target_type, target_format, true); 
                                let draw = EffectBlurDual::ready(
                                    param.downs.get(idx).unwrap(), resources, device, queue,
                                    0, (tow, toh),
                                    &matrix, 
                                    1., 0.,
                                    tempresult,
                                    safeatlas, target_type, pipelines,
                                    create_default_target(target_format), None, src_premultiplied, false
                                ).unwrap();
                                
                                let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
                                draws.push(draw);
                                tempresult = result;
                                // log::warn!("Down: {:?}", (tow, toh));
                                src_premultiplied = false;
                            }
                        }
        
                        if realiter >= 1 {
                            for idx in 1..realiter {
                                tow = tow * 2;
                                toh = toh * 2;
                                let result = EffectCopy::get_target(None, &tempresult, (tow, toh), safeatlas, target_type, target_format, true); 
                                let draw = EffectBlurDual::ready(
                                    param.ups.get(idx).unwrap(), resources, device, queue,
                                    0, (tow, toh),
                                    &matrix, 
                                    1., 0.,
                                    tempresult,
                                    safeatlas, target_type, pipelines,
                                    create_default_target(target_format), None, src_premultiplied, false
                                ).unwrap();
                                let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
                                draws.push(draw);
                                tempresult = result;
                                // log::warn!("Up: {:?}", (tow, toh));
                                
                                src_premultiplied = false;
                            }
                        }
        
                        tow = fromw;
                        toh = fromh;
                        let param = param.ups.get(0).unwrap();
                        let result = EffectCopy::get_target(None, &tempresult, (tow, toh), safeatlas, target_type, target_format, true); 
                        let draw = EffectBlurDual::ready(
                            param, resources, device, queue,
                            0, (tow, toh),
                            &matrix, 
                            extends.alpha, extends.depth,
                            tempresult,
                            safeatlas, target_type, pipelines,
                            color_state, depth_stencil, src_premultiplied, dst_premultiply
                        ).unwrap();
                        let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
                        draws.push(draw);
                        tempresult = result;

                        temp_result.target = Some(tempresult);
                    },
                    _ => {
                        return;
                    },
                }
                return;
            },
            EPostprocessRenderType::BloomDual => {
                let param = self.renderer_bloom_dual.as_ref().unwrap();
                match target {
                    ETarget::Temp(_, _) => {
                        let result = bloom_dual_render(
                            param,
                            device, queue, matrix, extends,
                            safeatlas, source.clone(), draws, resources, pipelines, depth_stencil, target_type, target_format, src_premultiplied, dst_premultiply
                        );
                        temp_result.target = Some(result);
                        return;
                    },
                    _ => {
                        return;
                    },
                }
            },
            EPostprocessRenderType::HorizonGlitch => {
                let param = self.renderer_horizon_glitch.as_ref().unwrap();
                match target {
                    ETarget::Temp(_, _) => {
                        let result = horizon_glitch_render(
                            param,
                            device, queue, self.horizon_glitch_instance.clone(), matrix,
                            safeatlas, source, None, draws, resources, pipelines, color_state, depth_stencil, target_type, target_format, src_premultiplied, dst_premultiply
                        );
                        temp_result.target = Some(result);
                        return;
                    },
                    _ => {
                        return;
                    },
                }
            },
            EPostprocessRenderType::BlurGauss => {
                let (hparam, vparam) = self.renderer_blur_gauss.as_ref().unwrap();
                match target {
                    ETarget::Temp(_, _) => {
                        let result = blur_gauss_render(
                            hparam, vparam,
                            device, queue, matrix,
                            safeatlas, source, None, draws, resources, pipelines, color_state, depth_stencil, target_type, target_format, src_premultiplied, dst_premultiply
                        );
                        temp_result.target = Some(result);
                        return;
                    },
                    _ => {
                        return;
                    },
                }
            },
        };

    }

    fn check(
        &mut self,
        delta_time: u64,
        final_step_by_draw_final: bool,
        device: & RenderDevice,
        queue: & RenderQueue,
        resources: &SingleImageEffectResource,
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
        let blur_gauss       = self.blur_gauss.is_some() && self.blur_gauss.as_ref().unwrap().is_enabled();
        let image_mask       = self.image_mask.is_some();
        let clip_sdf         = self.clip_sdf.is_some();
        let copy_intensity   = self.copy.is_some();

        let mut final_is_multi_render_steps = true;

        if color_effect {
            self.flags.push(EPostprocessRenderType::ColorEffect);
            if self.renderer_coloreffect.is_none() { self.renderer_coloreffect = Some(ColorEffectRenderer { hsb: None, balance: None, vignette: None, scale: None, filter: None, uniform: resources.uniform_buffer() } ) }
            if let Some(item) = self.renderer_coloreffect.as_mut() {
                item.balance = self.color_balance.clone();
                item.hsb = self.hsb.clone();
                item.vignette = self.vignette.clone();
                item.scale = self.color_scale.clone();
                item.filter = self.color_filter.clone();
            }
            final_is_multi_render_steps = false;
        }
        if blur_dual {
            self.flags.push(EPostprocessRenderType::BlurDual);
            if let Some(item) = self.renderer_blur_dual.as_mut() {
                item.update(self.blur_dual.as_ref().unwrap());
            } else {
                self.renderer_blur_dual = Some(BlurDualRendererList::new(self.blur_dual.as_ref().unwrap(), resources));
            }
            final_is_multi_render_steps = true;
        }
        if blur_direct {
            self.flags.push(EPostprocessRenderType::BlurDirect);
            if let Some(item) = self.renderer_blur_direct.as_mut() {
                item.param = self.blur_direct.as_ref().unwrap().clone();
            } else {
                self.renderer_blur_direct = Some(BlurDirectRenderer::new(self.blur_direct.as_ref().unwrap(), resources));
            }
            final_is_multi_render_steps = false;
        }
        if blur_radial {
            self.flags.push(EPostprocessRenderType::BlurRadial);
            if let Some(item) = self.renderer_blur_radial.as_mut() {
                item.param = self.blur_radial.as_ref().unwrap().clone();
            } else {
                self.renderer_blur_radial = Some(BlurRadialRenderer::new(self.blur_radial.as_ref().unwrap(), resources));
            }
            final_is_multi_render_steps = false;
        }
        if blur_bokeh {
            self.flags.push(EPostprocessRenderType::BlurBokeh);
            if let Some(item) = self.renderer_blur_bokeh.as_mut() {
                item.param = self.blur_bokeh.as_ref().unwrap().clone();
            } else {
                self.renderer_blur_bokeh = Some(BlurBokehRenderer::new(self.blur_bokeh.as_ref().unwrap(), resources));
            }
            final_is_multi_render_steps = false;
        }
        if bloom_dual {
            self.flags.push(EPostprocessRenderType::BloomDual);
            if let Some(item) = self.renderer_bloom_dual.as_mut() {
                item.update(self.bloom_dual.as_ref().unwrap());
            } else {
                self.renderer_bloom_dual = Some(BloomDualRenderer::new(self.bloom_dual.as_ref().unwrap(), resources));
            }
            final_is_multi_render_steps = true;
        }
        if blur_gauss {
            self.flags.push(EPostprocessRenderType::BlurGauss);
            if let Some((h, v)) = self.renderer_blur_gauss.as_mut() {
                h.param = self.blur_gauss.as_ref().unwrap().clone();
                v.param = h.param.clone();
            } else {
                let param = self.blur_gauss.as_ref().unwrap().clone();
                self.renderer_blur_gauss = Some((
                    BlurGaussRenderer { param: param.clone(), ishorizon: true, uniform: resources.uniform_buffer()  },
                    BlurGaussRenderer { param, ishorizon: false, uniform: resources.uniform_buffer() }
                ));
            }
            final_is_multi_render_steps = true;
        }
        if radial_wave {
            self.flags.push(EPostprocessRenderType::RadialWave);
            if let Some(item) = self.renderer_radial_wave.as_mut() {
                item.param = self.radial_wave.as_ref().unwrap().clone();
            } else {
                self.renderer_radial_wave = Some(RadialWaveRenderer::new(self.radial_wave.as_ref().unwrap(), resources));
            }
            final_is_multi_render_steps = false;
        }
        if horizon_glitch {
            self.horizon_glitch.as_mut().unwrap().update(delta_time);
            if let Some(item) = self.renderer_horizon_glitch.as_mut() {
                item.strength = self.horizon_glitch.as_ref().unwrap().strength;
                item.fade = self.horizon_glitch.as_ref().unwrap().fade;
            } else {
                self.renderer_horizon_glitch = Some(HorizonGlitchRenderer::new(self.horizon_glitch.as_ref().unwrap(), resources));
            }
            self.horizon_glitch_instance = horizon_glitch_render_calc(self.horizon_glitch.as_ref().unwrap(), self.renderer_horizon_glitch.as_ref().unwrap(), device, queue, resources);
            if self.horizon_glitch_instance.is_some() {
                self.flags.push(EPostprocessRenderType::HorizonGlitch);
                final_is_multi_render_steps = true;
            }
        }
        if filter_sobel {
            self.flags.push(EPostprocessRenderType::FilterSobel);
            if let Some(item) = self.renderer_filter_sobel.as_mut() {
                item.param = self.filter_sobel.as_ref().unwrap().clone();
            } else {
                self.renderer_filter_sobel = Some(FilterSobelRenderer::new(self.filter_sobel.as_ref().unwrap(), resources));
            }
            final_is_multi_render_steps = false;
        }
        if image_mask {
            self.flags.push(EPostprocessRenderType::ImageMask);
            if let Some(item) = self.renderer_image_mask.as_mut() {
                item.param = self.image_mask.as_ref().unwrap().clone();
            } else {
                self.renderer_image_mask = Some(ImageMaskRenderer::new(self.image_mask.as_ref().unwrap(), resources));
            }
            final_is_multi_render_steps = false;
        }
        if clip_sdf {
            self.flags.push(EPostprocessRenderType::ClipSdf);
            if let Some(item) = self.renderer_clip_sdf.as_mut() {
                item.param = self.clip_sdf.as_ref().unwrap().clone();
            } else {
                self.renderer_clip_sdf = Some(ClipSdfRenderer::new(self.clip_sdf.as_ref().unwrap(), resources));
            }
            final_is_multi_render_steps = false;
        }
        if copy_intensity {
            self.flags.push(EPostprocessRenderType::CopyIntensity);
            if let Some(item) = self.renderer_copyintensity.as_mut() {
                item.param = self.copy.as_ref().unwrap().clone();
            } else {
                self.renderer_copyintensity = Some(CopyIntensityRenderer::new(self.copy.as_ref().unwrap(), resources));
            }
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