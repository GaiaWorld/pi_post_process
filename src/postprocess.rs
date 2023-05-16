

use std::mem::replace;

use pi_assets::mgr::AssetMgr;
use pi_render::{
    components::view::target_alloc::{SafeAtlasAllocator, TargetType},
    rhi::{device::{RenderDevice}, pipeline::RenderPipeline, asset::RenderRes, RenderQueue},
    renderer::{
        pipeline::DepthStencilState,
        vertex_buffer::VertexBufferAllocator,
        vertices::RenderVertices,
        texture::*,
        draw_obj::DrawObj
    }
};
use pi_share::Share;

use crate::{
    effect::*,
    temprory_render_target::{PostprocessTexture},
    renderer::{ bloom_dual::bloom_dual_render, horizon_glitch::{horizon_glitch_render, horizon_glitch_render_calc}},
    error::EPostprocessError,
    image_effect::*,
    material::{create_default_target},
    geometry::IDENTITY_MATRIX, postprocess_flags::EPostprocessRenderType, SimpleRenderExtendsData
};

#[derive(Debug)]
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
    horizon_glitch_instance:Option<RenderVertices>,
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
            horizon_glitch_instance: None,
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
impl TempResult {
    fn target(&mut self) -> Option<PostprocessTexture> {
        replace(&mut self.target, None) 
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
        device: &RenderDevice,
        queue: &RenderQueue,
        vballocator: &mut VertexBufferAllocator,
    ) {
        self.check(delta_time, true, device, queue, vballocator);
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
        safeatlas: &SafeAtlasAllocator,
        resources: &SingleImageEffectResource,
        pipelines: &Share<AssetMgr<RenderRes<RenderPipeline>>>,
        target_type: TargetType,
    ) -> Result<PostprocessTexture, EPostprocessError> {

        let matrix: &[f32] = &IDENTITY_MATRIX;
        
        if src.use_w() != dst_size.0 || src.use_h() != dst_size.1 {
            let result = EffectCopy::get_target(None, &src, dst_size, safeatlas, target_type); 

            let draw = EffectCopy::ready(
                CopyIntensity::default(), resources, device, queue,
                0, dst_size,
                &matrix,
                1., 0.,
                &src,
                safeatlas, target_type, pipelines,
                create_default_target(), None, false
            ).unwrap();
            
            let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
            draw.draw(Some(encoder), None);
            src = result;
        }



        let result = self._draw_front(
            device, queue, encoder, src, &IDENTITY_MATRIX, safeatlas, resources, pipelines, target_type
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
        safeatlas: &SafeAtlasAllocator,
        source: &PostprocessTexture,
        target_size: (u32, u32),
        resources: &SingleImageEffectResource,
        pipelines: &Share<AssetMgr<RenderRes<RenderPipeline>>>,
        color_state: wgpu::ColorTargetState,
        depth_stencil: Option<DepthStencilState>,
        target_type: TargetType,
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
                self._draw_single_simple(device, queue, None, matrix, extends, flag, safeatlas, source, ETarget::Final(target_size.0, target_size.1), &mut draws, resources, pipelines, color_state, depth_stencil, target_type, &mut tempresult);

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
        encoder: &mut wgpu::CommandEncoder,
        src: PostprocessTexture,
        _: &[f32],
        safeatlas: &SafeAtlasAllocator,
        resources: &SingleImageEffectResource,
        pipelines: &Share<AssetMgr<RenderRes<RenderPipeline>>>,
        target_type: TargetType,
    ) -> Result<PostprocessTexture, EPostprocessError>  {
        let count = self.flags.len();

        if count <= 1 {
            Ok(src)
        } else {
            let mut source = src;
            let target = ETarget::Temp(source.use_w(), source.use_h());
            // let format = wgpu::TextureFormat::Rgba8UnormSrgb;
            for i in 0..count-1 {
                let flag = *self.flags.get(i).unwrap();

                let mut draws = vec![];
                
                let mut temp_result = TempResult { target: None, finaldraw: None };

                self._draw_single_simple(
                    device, queue, Some(encoder),
                    &IDENTITY_MATRIX, SimpleRenderExtendsData::default(), flag, safeatlas,
                    &source, target,
                    &mut draws, resources, pipelines,
                    create_default_target(), None, target_type, &mut temp_result
                );

                source = temp_result.target.unwrap();
                temp_result.target = None;
                draws.iter().for_each(|v| {
                    v.draw(Some(encoder), None);
                });
            }

            Ok(source)
        }
    }

    // fn _draw_single_front<'a>(
    //     &self,
    //     device: & RenderDevice,
    //     queue: & RenderQueue,
    //     encoder: &mut wgpu::CommandEncoder,
    //     matrix: & [f32],
    //     flag: EPostprocessRenderType,
    //     safeatlas: &SafeAtlasAllocator,
    //     source: PostprocessTexture,
    //     target: Option<PostprocessTexture>,
    //     draws: &mut Vec<PostProcessDraw>,
    //     resources: &SingleImageEffectResource,
    //     pipelines: &Share<AssetMgr<RenderRes<RenderPipeline>>>,
    //     depth_stencil: Option<DepthStencilState>,
    //     target_type: TargetType,
    // ) -> PostprocessTexture {

    //     // self._draw_single_simple(device, queue, Some(encoder), matrix, SimpleRenderExtendsData::default(), flag, safeatlas, source, target, draws, resources, pipelines, create_default_target(), depth_stencil, target_type)

    // }

    fn _draw_single_simple<'a>(
        &'a self,
        device: & RenderDevice,
        queue: & RenderQueue,
        encoder: Option<&mut wgpu::CommandEncoder>,
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
        temp_result: &mut TempResult,
    ) {
        let dst_size = target.size();
        let force_nearest_filter = source.size_eq_2(&dst_size);
        match flag {
            EPostprocessRenderType::ColorEffect => {
                let param = ColorEffect {
                    hsb: self.hsb.clone(),
                    balance: self.color_balance.clone(),
                    vignette: self.vignette.clone(),
                    scale: self.color_scale.clone(),
                    filter: self.color_filter.clone()
                };
                match target {
                    ETarget::Temp(_, _) => {
                        let result = EffectCopy::get_target(None, &source, dst_size, safeatlas, target_type); 
                        let draw = EffectColorEffect::ready(
                            param, resources, device, queue, 0, dst_size, &matrix, extends.alpha, extends.depth,
                            source, safeatlas, target_type, pipelines, color_state, depth_stencil, force_nearest_filter
                        ).unwrap();
                        let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
                        draws.push(draw);
                        temp_result.target = Some(result);
                    },
                    ETarget::Final(_, _) => {
                        let draw = EffectColorEffect::ready(
                            param, resources, device, queue, 0, dst_size, &matrix, extends.alpha, extends.depth,
                            source, safeatlas, target_type, pipelines, color_state, depth_stencil, force_nearest_filter
                        ).unwrap();
                        temp_result.finaldraw = Some(draw);
                    },
                }
            },
            EPostprocessRenderType::BlurDirect => {
                match target {
                    ETarget::Temp(_, _) => {
                        let result = EffectCopy::get_target(None, &source, dst_size, safeatlas, target_type); 
                        let draw = EffectBlurDirect::ready(
                            self.blur_direct.as_ref().unwrap().clone(), resources, device, queue,
                            0, dst_size, &matrix, 1., 1., source, safeatlas, target_type, pipelines, color_state, depth_stencil
                        ).unwrap();
                        let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
                        draws.push(draw);
                        temp_result.target = Some(result);
                    },
                    ETarget::Final(_, _) => {
                        let draw = EffectBlurDirect::ready(
                            self.blur_direct.as_ref().unwrap().clone(), resources, device, queue,
                            0, dst_size, &matrix, 1., 1., source, safeatlas, target_type, pipelines, color_state, depth_stencil
                        ).unwrap();
                        temp_result.finaldraw = Some(draw);
                    },
                }
            },
            EPostprocessRenderType::BlurRadial => {
                match target {
                    ETarget::Temp(_, _) => {
                        let result = EffectCopy::get_target(None, &source, dst_size, safeatlas, target_type); 
                        let draw = EffectBlurRadial::ready(
                            self.blur_radial.as_ref().unwrap().clone(), resources, device, queue,
                            0, dst_size, &matrix, extends.alpha, extends.depth, source, safeatlas, target_type, pipelines,  color_state, depth_stencil
                        ).unwrap();
                        let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
                        draws.push(draw);
                        temp_result.target = Some(result);
                    },
                    ETarget::Final(_, _) => {
                        let draw = EffectBlurRadial::ready(
                            self.blur_radial.as_ref().unwrap().clone(), resources, device, queue,
                            0, dst_size, &matrix, extends.alpha, extends.depth, source, safeatlas, target_type, pipelines,  color_state, depth_stencil
                        ).unwrap();
                        temp_result.finaldraw = Some(draw);
                    },
                }
            },
            EPostprocessRenderType::BlurBokeh => {
                match target {
                    ETarget::Temp(_, _) => {
                        let result = EffectCopy::get_target(None, &source, dst_size, safeatlas, target_type); 
                        let draw = EffectBlurBokeh::ready(
                            self.blur_bokeh.as_ref().unwrap().clone(), resources, device, queue,
                            0, dst_size, &matrix, extends.alpha, extends.depth, &source, safeatlas, target_type, pipelines, color_state, depth_stencil
                        ).unwrap();
                        let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
                        draws.push(draw);
                        temp_result.target = Some(result);
                    },
                    ETarget::Final(_, _) => {
                        let draw = EffectBlurBokeh::ready(
                            self.blur_bokeh.as_ref().unwrap().clone(), resources, device, queue,
                            0, dst_size, &matrix, extends.alpha, extends.depth, &source, safeatlas, target_type, pipelines, color_state, depth_stencil
                        ).unwrap();
                        temp_result.finaldraw = Some(draw);
                    },
                }
            },
            EPostprocessRenderType::RadialWave => {
                match target {
                    ETarget::Temp(_, _) => {
                        let result = EffectCopy::get_target(None, &source, dst_size, safeatlas, target_type); 
                        let draw = EffectRadialWave::ready(
                            self.radial_wave.as_ref().unwrap().clone(), resources, device, queue,
                            0, dst_size, &matrix,  extends.alpha, extends.depth, source, safeatlas, target_type, pipelines, color_state, depth_stencil
                        ).unwrap();
                        let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
                        draws.push(draw);
                        temp_result.target = Some(result);
                    },
                    ETarget::Final(_, _) => {
                        let draw = EffectRadialWave::ready(
                            self.radial_wave.as_ref().unwrap().clone(), resources, device, queue,
                            0, dst_size, &matrix,  extends.alpha, extends.depth, source, safeatlas, target_type, pipelines, color_state, depth_stencil
                        ).unwrap();
                        temp_result.finaldraw = Some(draw);
                    },
                }
            },
            EPostprocessRenderType::FilterSobel => {
                match target {
                    ETarget::Temp(_, _) => {
                        let result = EffectCopy::get_target(None, &source, dst_size, safeatlas, target_type); 
                        let draw = EffectFilterSobel::ready(
                            self.filter_sobel.as_ref().unwrap().clone(), resources, device, queue,
                            0, dst_size, &matrix,  extends.alpha, extends.depth, source, safeatlas, target_type, pipelines, color_state, depth_stencil
                        ).unwrap();
                        let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
                        draws.push(draw);
                        temp_result.target = Some(result);
                    },
                    ETarget::Final(_, _) => {
                        let draw = EffectFilterSobel::ready(
                            self.filter_sobel.as_ref().unwrap().clone(), resources, device, queue,
                            0, dst_size, &matrix,  extends.alpha, extends.depth, source, safeatlas, target_type, pipelines, color_state, depth_stencil
                        ).unwrap();
                        temp_result.finaldraw = Some(draw);
                    },
                }
            },
            EPostprocessRenderType::CopyIntensity => {
                match target {
                    ETarget::Temp(_, _) => {
                        let result = EffectCopy::get_target(None, &source, dst_size, safeatlas, target_type); 
                        let draw = EffectCopy::ready(
                            self.copy.as_ref().unwrap().clone(), resources, device, queue,
                            0, dst_size, &matrix,  extends.alpha, extends.depth, source, safeatlas, target_type, pipelines, color_state, depth_stencil, force_nearest_filter
                        ).unwrap();
                        let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
                        draws.push(draw);
                        temp_result.target = Some(result);
                    },
                    ETarget::Final(_, _) => {
                        let draw = EffectCopy::ready(
                            self.copy.as_ref().unwrap().clone(), resources, device, queue,
                            0, dst_size, &matrix,  extends.alpha, extends.depth, source, safeatlas, target_type, pipelines, color_state, depth_stencil, force_nearest_filter
                        ).unwrap();
                        temp_result.finaldraw = Some(draw);
                    },
                }
            },
            EPostprocessRenderType::FinalCopyIntensity => {
                match target {
                    ETarget::Temp(_, _) => {
                        let result = EffectCopy::get_target(None, &source, dst_size, safeatlas, target_type); 
                        let draw = EffectCopy::ready(
                            CopyIntensity::default(), resources, device, queue,
                            0, dst_size, &matrix,  extends.alpha, extends.depth, source, safeatlas, target_type, pipelines, color_state, depth_stencil, force_nearest_filter
                        ).unwrap();
                        let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
                        draws.push(draw);
                        temp_result.target = Some(result);
                    },
                    ETarget::Final(_, _) => {
                        let draw = EffectCopy::ready(
                            CopyIntensity::default(), resources, device, queue,
                            0, dst_size, &matrix,  extends.alpha, extends.depth, source, safeatlas, target_type, pipelines, color_state, depth_stencil, force_nearest_filter
                        ).unwrap();
                        temp_result.finaldraw = Some(draw);
                    },
                }
            },
            EPostprocessRenderType::BlurDual => {
                match target {
                    ETarget::Temp(_, _) => {
                        let mut realiter = 0;
                        let blur_dual = self.blur_dual.as_ref().unwrap();
                        let fromw = dst_size.0;
                        let fromh = dst_size.1;
                        let mut tow = fromw;
                        let mut toh = fromh;
                        let mut tempresult = source.clone();
                        for _ in 0..blur_dual.iteration {
                            if tow / 2 >= 2 && toh / 2 >= 2 {
                                tow = tow / 2;
                                toh = toh / 2;
                                realiter += 1;
                                let param = BlurDualForBuffer { param: blur_dual.clone(), isup: false };
                                let result = EffectCopy::get_target(None, &tempresult, (tow, toh), safeatlas, target_type); 
                                let draw = EffectBlurDual::ready(
                                    param, resources, device, queue,
                                    0, (tow, toh),
                                    &matrix, 
                                    1., 0.,
                                    tempresult,
                                    safeatlas, target_type, pipelines,
                                    create_default_target(), None
                                ).unwrap();
                                
                                let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
                                draws.push(draw);
                                tempresult = result;
                                // log::warn!("Down: {:?}", (tow, toh));
                            }
                        }
        
                        if realiter >= 1 {
                            for _ in 1..realiter {
                                tow = tow * 2;
                                toh = toh * 2;
                                let param = BlurDualForBuffer { param: blur_dual.clone(), isup: true };
                                let result = EffectCopy::get_target(None, &tempresult, (tow, toh), safeatlas, target_type); 
                                let draw = EffectBlurDual::ready(
                                    param, resources, device, queue,
                                    0, (tow, toh),
                                    &matrix, 
                                    1., 0.,
                                    tempresult,
                                    safeatlas, target_type, pipelines,
                                    create_default_target(), None
                                ).unwrap();
                                let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
                                draws.push(draw);
                                tempresult = result;
                                // log::warn!("Up: {:?}", (tow, toh));
                            }
                        }
        
                        tow = fromw;
                        toh = fromh;
                        let param = BlurDualForBuffer { param: blur_dual.clone(), isup: true };
                        let result = EffectCopy::get_target(None, &tempresult, (tow, toh), safeatlas, target_type); 
                        let draw = EffectBlurDual::ready(
                            param, resources, device, queue,
                            0, (tow, toh),
                            &matrix, 
                            extends.alpha, extends.depth,
                            tempresult,
                            safeatlas, target_type, pipelines,
                            color_state, depth_stencil
                        ).unwrap();

                        let draw = PostProcessDraw::Temp(result.get_rect(), draw, result.view.clone() );
                        draws.push(draw);
                        temp_result.target = Some(result);
                    },
                    _ => {
                        return;
                    },
                }
                return;
            },
            EPostprocessRenderType::BloomDual => {
                match target {
                    ETarget::Temp(_, _) => {
                        if let Some(encoder) = encoder {
                            let result = bloom_dual_render(
                                self.bloom_dual.as_ref().unwrap(),
                                device, queue, encoder, matrix, extends,
                                safeatlas, source.clone(), draws, resources, pipelines, depth_stencil, target_type
                            );
                            temp_result.target = Some(result);
                        } else {
                            temp_result.target = Some(source.clone());
                        };
                        return;
                    },
                    _ => {
                        return;
                    },
                }
            },
            EPostprocessRenderType::HorizonGlitch => {
                match target {
                    ETarget::Temp(_, _) => {
                        if let Some(encoder) = encoder {
                            let result = horizon_glitch_render(
                                self.horizon_glitch.as_ref().unwrap(),
                                device, queue, self.horizon_glitch_instance.clone(), matrix,
                                safeatlas, source, None, draws, resources, pipelines, color_state, depth_stencil, target_type
                            );
                            temp_result.target = Some(result);
                        } else {
                            temp_result.target = Some(source.clone());
                        };
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
        vballocator: &mut VertexBufferAllocator,
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
            self.horizon_glitch_instance = horizon_glitch_render_calc(self.horizon_glitch.as_ref().unwrap(), device, queue, vballocator);
            if self.horizon_glitch_instance.is_some() {
                self.flags.push(EPostprocessRenderType::HorizonGlitch);
                final_is_multi_render_steps = true;
            }
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