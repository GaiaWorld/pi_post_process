
use std::{time::SystemTime, sync::Arc, mem::size_of};

use image::{GenericImageView};
use pi_assets::{mgr::AssetMgr, asset::{GarbageEmpty, Handle}, homogeneous::HomogeneousMgr};
use pi_postprocess::{
    postprocess::{PostProcess}, 
    effect::*,
    material::{blend::{get_blend_state, EBlend}, create_target},
    temprory_render_target::{PostprocessTexture},
    image_effect::*,
};
use pi_render::{components::view::target_alloc::{SafeAtlasAllocator, UnuseTexture, TargetDescriptor, TextureDescriptor, TargetType}, rhi::{device::RenderDevice, asset::{RenderRes, TextureRes, }, pipeline::RenderPipeline, RenderQueue, }, renderer::{texture::*, sampler::SamplerRes, vertex_buffer::VertexBufferAllocator, draw_obj::DrawObj}, asset::TAssetKeyU64};
use pi_share::Share;
use smallvec::SmallVec;
use winit::{window::Window, event::WindowEvent};

pub struct State {
    pub surface: wgpu::Surface,
    pub renderdevice: RenderDevice,
    pub queue: RenderQueue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub pipelines: Share<AssetMgr<RenderRes<RenderPipeline>>>,
    pub asset_samplers: Share<AssetMgr::<SamplerRes>>,
    pub resources: SingleImageEffectResource,
    pub postprocess: PostProcess,
    pub value_test: u8,
    pub asset_tex: Share<AssetMgr<TextureRes>>,
    pub diffuse_texture: Handle<TextureRes>,
    pub diffuse_size: wgpu::Extent3d,
    // pub diffuse_buffer: wgpu::Buffer,
    pub lasttime: SystemTime,
    atlas: SafeAtlasAllocator,
    target_type: TargetType,
    vballocator: VertexBufferAllocator,
    draws: Vec<DrawObj>,
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            dx12_shader_compiler: wgpu::Dx12Compiler ::default(),
        });
        let surface = unsafe { instance.create_surface(window).unwrap() };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            }
        )
        .await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::DEPTH_CLIP_CONTROL,
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
            },
            None
        )
        .await.unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        println!("Size: {:?}", size);
        surface.configure(&device, &config);

        ///// 
        let postprocess = PostProcess::default();

        //// Texture
        let diffuse_bytes = include_bytes!("../dialog_bg.png");
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_image.as_bytes();
        let dimensions = diffuse_image.dimensions();
        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let diffuse_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                // All textures are stored as 3D, we represent our 2D texture
                // by setting depth to 1.
                size: texture_size,
                mip_level_count: 1, // We'll talk about this a little later
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                // Most images are stored using sRGB so we need to reflect that here.
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                // COPY_DST means that we want to copy data to this texture
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
                label: Some("diffuse_texture"),
                view_formats: &vec![],
            }
        );
        queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            &diffuse_rgba,
            // The layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * dimensions.0),
                rows_per_image: std::num::NonZeroU32::new(dimensions.1),
            },
            texture_size,
        );
        let texture_view = diffuse_texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );

        let asset_tex = AssetMgr::<TextureRes>::new(GarbageEmpty(), false, 1024, 10000);   
        let key_img = KeyImageTexture::from("../dialog_bg.png");
        let diffuse_texture = asset_tex.insert(key_img.asset_u64(), TextureRes::new(texture_size.width, texture_size.height, (texture_size.width * texture_size.height * 4) as usize, texture_view, true)).unwrap();

        let texture_assets_mgr = AssetMgr::<RenderRes<wgpu::TextureView>>::new(
            GarbageEmpty(), 
            false,
            60 * 1024 * 1024, 
            3 * 60 * 1000
        );
        let unusetexture_assets_mgr = HomogeneousMgr::<RenderRes<UnuseTexture>>::new(
            pi_assets::homogeneous::GarbageEmpty(), 
            10 * size_of::<UnuseTexture>(),
            size_of::<UnuseTexture>(),
            3 * 60 * 1000,
        );
        let renderdevice = RenderDevice::from(Arc::new(device));
        let queue = RenderQueue::from(queue);
        let mut atlas = SafeAtlasAllocator::new(renderdevice.clone(), texture_assets_mgr, unusetexture_assets_mgr);
        let mut colors_descriptor = SmallVec::<[TextureDescriptor;1]>::new();
        colors_descriptor.push(
            TextureDescriptor {
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
                base_mip_level: 0,
                base_array_layer: 0,
                array_layer_count: None,
                view_dimension: Some(wgpu::TextureViewDimension::D2),
            }
        );
        let target_type = atlas.create_type(TargetDescriptor {
            colors_descriptor: colors_descriptor,
            need_depth: false,
            default_width: 2048,
            default_height: 2048,
            depth_descriptor: None
        });
        let mut vballocator = VertexBufferAllocator::new();
        let pipelines = AssetMgr::<RenderRes<RenderPipeline>>::new(GarbageEmpty(), false, 1024, 10000);
        let mut resources = SingleImageEffectResource::new(&renderdevice, &queue, &mut vballocator);
        let asset_samplers = AssetMgr::<SamplerRes>::new(GarbageEmpty(), false, 1024, 10000);
        
        EffectBlurBokeh::setup(&renderdevice, &mut resources, &asset_samplers);
        EffectBlurDirect::setup(&renderdevice, &mut resources, &asset_samplers);
        EffectBlurDual::setup(&renderdevice, &mut resources, &asset_samplers);
        EffectBlurRadial::setup(&renderdevice, &mut resources, &asset_samplers);
        EffectColorEffect::setup(&renderdevice, &mut resources, &asset_samplers);
        EffectCopy::setup(&renderdevice, &mut resources, &asset_samplers);
        EffectFilterBrightness::setup(&renderdevice, &mut resources, &asset_samplers);
        EffectFilterSobel::setup(&renderdevice, &mut resources, &asset_samplers);
        EffectHorizonGlitch::setup(&renderdevice, &mut resources, &asset_samplers);
        EffectRadialWave::setup(&renderdevice, &mut resources, &asset_samplers);

        Self {
            surface,
            renderdevice,
            queue,
            config,
            size,
            pipelines,
            asset_samplers,
            resources,
            postprocess,
            value_test: 0,
            asset_tex,
            diffuse_size: texture_size,
            diffuse_texture: diffuse_texture,
            lasttime: std::time::SystemTime::now(),
            atlas,
            target_type,
            vballocator,
            draws: vec![],
        }
    }

    pub fn resize(
        &mut self,
        new_size: winit::dpi::PhysicalSize<u32>
    ) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            // self.surface.configure(&self.renderdevice.wgpu_device(), &self.config);
        }
    }

    pub fn input(
        &mut self,
        event: &WindowEvent,
    ) -> bool {
        false
    }

    pub fn update(
        &mut self,
    ) {
        let mut r = self.value_test;
        if r == 200 {
            r = 0;
        } else {
            r = r + 1;
        }
        self.value_test = r;
        // self.postprocess.color_balance = Some(ColorBalance { r: r, g: 255 - r, b: 255 });
        // self.postprocess.color_filter = Some(ColorFilter { r: r, g: 0, b: 0 });
        self.postprocess.vignette = Some(Vignette { r: r, g: 0, b: 0, begin: 0.5, end: 1.5, scale: 1.0 });
        // self.postprocess.hsb = Some(HSB { hue: 0, brightness: 0, saturate: (r as i16 - 100) as i8 });
        self.postprocess.blur_dual = Some(BlurDual { radius: 1, iteration: 2, intensity: 1.0f32, simplified_up: false });
        self.postprocess.blur_direct = Some(BlurDirect { radius: 4, iteration: 10, direct_x: r as f32 / 255.0 * 2.0 - 1.0, direct_y: 1.0 });
        self.postprocess.blur_radial = Some(BlurRadial { radius: 4, iteration: 10, center_x: 0., center_y: 0., start: 0.1, fade: 0.2  });
        self.postprocess.blur_bokeh = Some(BlurBokeh { radius: 0.5, iteration: 8, center_x: 0., center_y: 0., start: 0.0, fade: 0.0  });

        if self.postprocess.horizon_glitch.is_none() {
            let mut hg = HorizonGlitch::default();
            hg.probability = 0.8;
            hg.max_count = 200;
            hg.min_count = 50;
            hg.max_size = 0.05;
            hg.min_size = 0.01;
            hg.strength = 0.2;
            self.postprocess.horizon_glitch = Some(hg);
        }

        self.postprocess.bloom_dual = Some(BloomDual { radius: 1, iteration: 1, intensity: 1.0f32, threshold: r as f32 / 255.0, threshold_knee: 0.5 });

        self.postprocess.radial_wave = Some(RadialWave { aspect_ratio: true, start: r as f32 / 255.0, end: r as f32 / 255.0 + 0.5, center_x: 0., center_y: 0., cycle: 2, weight: 0.2  });
        
        self.postprocess.filter_sobel = Some(FilterSobel{ size: 1, clip: r as f32 / 255.0, color: (255, 0, 0, 255), bg_color: (0, 0, 0, 125)  });

        self.postprocess.copy = Some(CopyIntensity { intensity: 2.0f32, polygon: r / 10, radius: r as f32 / 255.0, angle: r as f32, bg_color: (0, 0, 0, 125) });
        // self.postprocess.alpha = Some(Alpha { a: r as f32 / 255.0 });
    }

    pub fn render(
        &mut self,
    ) -> Result<(), wgpu::SurfaceError> {
        // let last_time = SystemTime::now();
        let output = match self.surface.get_current_texture() {
            Ok(output) => {
                output
            },
            Err(e) => {
                println!("Err {:?}", e);
                return Ok(());
            }
        };

        // BGRASrgb
        let ouput_format = self.config.format;

        let view = output.texture.create_view(
            &wgpu::TextureViewDescriptor {
                label: None,
                format: Some(ouput_format),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            }
        );

        let mut encoder = self.renderdevice.wgpu_device().create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Ender Encoder")
            }
        );

        self.clear(&mut encoder, &view);
        
        // let format = wgpu::TextureFormat::Rgba8UnormSrgb;

        let src_texture = PostprocessTexture {
            use_x: 0, // self.diffuse_size.width / 4,
            use_y: 0, //self.diffuse_size.height / 4,
            use_w: self.diffuse_size.width, // / 2,
            use_h: self.diffuse_size.height, // / 2,
            width: self.diffuse_size.width,
            height: self.diffuse_size.height,
            view: ETextureViewUsage::Tex(self.diffuse_texture.clone()),
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
        };

        let receive_w = self.size.width - 200 as u32;
        let receive_h = self.size.height - 200 as u32;
        let receive_width = self.size.width;
        let receive_height = self.size.height;

        let dst = PostprocessTexture {
            use_x: self.value_test as u32,
            use_y: self.value_test as u32,
            use_w: receive_w,
            use_h: receive_h,
            width: receive_width,
            height: receive_height,
            view: ETextureViewUsage::Temp(Arc::new(view), 0),
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
        };

        let final_targets = create_target(ouput_format, get_blend_state(EBlend::Combine), wgpu::ColorWrites::ALL);
        let final_depth_and_stencil = None;
        
        self.postprocess.calc(
            16,
            &self.renderdevice, 
            &self.queue,
            &mut self.vballocator,
        );

        let result = self.postprocess.draw_front(
            &self.renderdevice, 
            &self.queue,
            &mut encoder,
            src_texture,
            (receive_w, receive_h),
            & self.atlas,
            &self.resources,
            & self.pipelines,
            self.target_type.clone()
        );

        self.draws.clear();
        let _ = match result {
            Ok(result) => {
                let matrix = [0.3535533845424652, 0.3535533845424652, 0., 0., -0.3535533845424652, 0.3535533845424652, 0., 0., 0., 0., 0.5, 0., 0., 0., 0., 1.];
                // renderpass.set_viewport(dst.use_x as f32, dst.use_y as f32, dst.use_w as f32, dst.use_h as f32, 0., 1.);
                if let Some(draw) = self.postprocess.draw_final(
                    &self.renderdevice, 
                    &mut self.queue,
                    &matrix,
                    1.,
                    &self.atlas,
                    &result,
                    (dst.use_w(), dst.use_h()),
                    &self.resources,
                    // &IDENTITY_MATRIX,
                    &self.pipelines,
                    final_targets,
                    final_depth_and_stencil,
                    self.target_type.clone(),
                ) {
                    self.draws.push(draw);
                }

                let mut renderpass = encoder.begin_render_pass(
                    &wgpu::RenderPassDescriptor {
                        label: Some("ToScreen"),
                        color_attachments: &[
                            Some(
                                wgpu::RenderPassColorAttachment {
                                    view: dst.view(),
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Load,
                                        store: true,
                                    }
                                }
                            )
                        ],
                        depth_stencil_attachment: None,
                    }
                );

                self.draws.iter().for_each(|v| {
                    v.draw(&mut renderpass)
                });
            },
            Err(_) => {
                
            },
        };

        self.queue.submit(std::iter::once(encoder.finish()));

        // output.present();

        // let new_time = SystemTime::now();
        // println!("{:?}", new_time.duration_since(last_time));
        Ok(())
    }

    fn clear(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView
    ) {
        let _ = encoder.begin_render_pass(
            &wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[
                    Some(
                        wgpu::RenderPassColorAttachment {
                            view: view,
                            resolve_target: None,
                            ops:wgpu::Operations {
                                load: wgpu::LoadOp::Clear(
                                    wgpu::Color {
                                        r: self.value_test as f64 / 255.0, 
                                        g: 0.21, 
                                        b: 0.41, 
                                        a: 1.0, 
                                    }
                                ),
                                store: true
                            }
                        }
                    )
                ],
                depth_stencil_attachment: None,
            }
        );
    }
}
