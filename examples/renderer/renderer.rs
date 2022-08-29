
use std::{num::NonZeroU32, time::SystemTime, sync::Arc};

use image::{GenericImageView};
use pi_assets::{mgr::AssetMgr, asset::GarbageEmpty};
use pi_postprocess::{postprocess::{PostProcess}, effect::{color_balance::ColorBalance, hsb::HSB, blur_dual::BlurDual, copy::CopyIntensity, blur_direct::BlurDirect, radial_wave::RadialWave, blur_radial::BlurRadial, vignette::Vignette, color_filter::ColorFilter, filter_sobel::FilterSobel, bloom_dual::BloomDual, blur_bokeh::BlurBokeh, horizon_glitch::HorizonGlitch, alpha::Alpha}, material::{fragment_state::create_target, blend::{get_blend_state, EBlend}}, postprocess_geometry::PostProcessGeometryManager, postprocess_pipeline::PostProcessMaterialMgr, geometry::IDENTITY_MATRIX, temprory_render_target::{EPostprocessTarget, PostprocessTexture}};
use pi_render::{components::view::target_alloc::{SafeAtlasAllocator, ShareTargetView}, rhi::{device::RenderDevice, asset::{RenderRes, }, }};
use winit::{window::Window, event::WindowEvent};

pub struct State {
    pub surface: wgpu::Surface,
    pub renderdevice: RenderDevice,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub pipelines: PostProcessMaterialMgr,
    pub geometrys: PostProcessGeometryManager,
    pub postprocess: PostProcess,
    pub value_test: u8,
    pub diffuse_texture: wgpu::Texture,
    pub diffuse_size: wgpu::Extent3d,
    // pub diffuse_buffer: wgpu::Buffer,
    pub lasttime: SystemTime,
    atlas: SafeAtlasAllocator
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);
        let surface = unsafe { instance.create_surface(window) };
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
                features: wgpu::Features::empty(),
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
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        ///// 
        let pipelines = PostProcessMaterialMgr::new();
        let geometrys = PostProcessGeometryManager::new();
        let postprocess = PostProcess::default();

        //// Texture
        let diffuse_bytes = include_bytes!("../happy-tree.png");
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
                format: wgpu::TextureFormat::Bgra8Unorm,
                // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                // COPY_DST means that we want to copy data to this texture
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
                label: Some("diffuse_texture"),
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
        

        let texture_assets_mgr = AssetMgr::<RenderRes<wgpu::TextureView>>::new(
            GarbageEmpty(), 
            false,
            60 * 1024 * 1024, 
            3 * 60 * 1000
        );
        let renderdevice = RenderDevice::from(Arc::new(device));
        let atlas = SafeAtlasAllocator::new(renderdevice.clone(), texture_assets_mgr.clone());

        Self {
            surface,
            renderdevice,
            queue,
            config,
            size,
            pipelines,
            geometrys,
            postprocess,
            value_test: 0,
            diffuse_size: texture_size,
            diffuse_texture: diffuse_texture,
            lasttime: std::time::SystemTime::now(),
            atlas
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
            self.surface.configure(&self.renderdevice.wgpu_device(), &self.config);
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
        if r == 255 {
            r = 0;
        } else {
            r = r + 1;
        }
        self.value_test = r;
        // self.postprocess.color_balance = Some(ColorBalance { r: r, g: 255 - r, b: 255 });
        // self.postprocess.color_filter = Some(ColorFilter { r: r, g: 0, b: 0 });
        // self.postprocess.vignette = Some(Vignette { r: r, g: 0, b: 0, begin: 0.5, end: 1.5, scale: 1.0 });
        self.postprocess.hsb = Some(HSB { hue: self.value_test as i16, brightness: 1, saturate: 1 });
        // self.postprocess.blur_dual = Some(BlurDual { radius: 1, iteration: 4, intensity: 1.0f32, simplified_up: false });
        // self.postprocess.blur_direct = Some(BlurDirect { radius: 4, iteration: 10, direct_x: r as f32 / 255.0 * 2.0 - 1.0, direct_y: 1.0 });
        // self.postprocess.blur_radial = Some(BlurRadial { radius: 4, iteration: 10, center_x: 0., center_y: 0., start: 0.1, fade: 0.2  });
        // self.postprocess.blur_bokeh = Some(BlurBokeh { radius: 0.5, iteration: 10, center_x: 0., center_y: 0., start: 0.0, fade: 0.0  });

        // if self.postprocess.horizon_glitch.is_none() {
        //     let mut hg = HorizonGlitch::default();
        //     hg.probability = 0.8;
        //     hg.max_count = 200;
        //     hg.min_count = 50;
        //     hg.max_size = 0.05;
        //     hg.min_size = 0.01;
        //     hg.strength = 0.2;
        //     self.postprocess.horizon_glitch = Some(hg);
        // }

        // self.postprocess.bloom_dual = Some(BloomDual { radius: 1, iteration: 1, intensity: 1.0f32, threshold: r as f32 / 255.0, threshold_knee: 0.5 });

        self.postprocess.radial_wave = Some(RadialWave { aspect_ratio: true, start: r as f32 / 255.0, end: r as f32 / 255.0 + 0.5, center_x: 0., center_y: 0., cycle: 2, weight: 0.2  });
        
        // self.postprocess.filter_sobel = Some(FilterSobel{ size: 1, clip: r as f32 / 255.0, color: (255, 0, 0, 255), bg_color: (0, 0, 0, 125)  });

        // self.postprocess.copy = Some(CopyIntensity { intensity: 2.0f32, polygon: r / 10, radius: r as f32 / 255.0, angle: r as f32, bg_color: (0, 0, 0, 125) });
        self.postprocess.alpha = Some(Alpha { a: r as f32 / 255.0 });
    }

    pub fn render(
        &mut self,
    ) -> Result<(), wgpu::SurfaceError> {
        let last_time = SystemTime::now();
        let output = self.surface.get_current_texture()?;

        // BGRASrgb
        let ouput_format = self.config.format;

        let view = output.texture.create_view(
            &wgpu::TextureViewDescriptor {
                label: None,
                format: Some(ouput_format),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count:  NonZeroU32::new(0),
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
        
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let texture_view = self.diffuse_texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );

        let src_texture = PostprocessTexture {
            use_x: 0,
            use_y: 0,
            use_w: self.diffuse_size.width,
            use_h: self.diffuse_size.height,
            width: self.diffuse_size.width,
            height: self.diffuse_size.height,
            view: &texture_view,
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
            view: &view,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
        };

        let final_targets = [create_target(ouput_format, get_blend_state(EBlend::Combine), wgpu::ColorWrites::ALL)];
        let final_depth_and_stencil = None;
        
        self.postprocess.calc(
            16,
            &self.renderdevice, &mut self.pipelines, &mut self.geometrys,
            &final_targets,
            final_depth_and_stencil.clone(),
        );

        let result = self.postprocess.draw_front(
            &self.renderdevice, 
            &mut self.queue,
            &mut encoder,
            &self.atlas,
            & self.pipelines,
            & self.geometrys,
            EPostprocessTarget::TextureView(src_texture),
            (receive_w, receive_h),
        );

        let result = match result {
            Ok(result) => {
                let texture = result;
                match self.postprocess.get_final_texture_bind_group(&self.renderdevice, &self.pipelines, &texture, &final_targets, &final_depth_and_stencil) {
                    Some(texture_bind_group) => {
                        let mut renderpass = encoder.begin_render_pass(
                            &wgpu::RenderPassDescriptor {
                                label: Some("ToScreen"),
                                color_attachments: &[
                                    wgpu::RenderPassColorAttachment {
                                        view: dst.view,
                                        resolve_target: None,
                                        ops: wgpu::Operations {
                                            load: wgpu::LoadOp::Load,
                                            store: true,
                                        }
                                    }
                                ],
                                depth_stencil_attachment: None,
                            }
                        );
                        renderpass.set_viewport(dst.use_x as f32, dst.use_y as f32, dst.use_w as f32, dst.use_h as f32, 0., 1.);
                        self.postprocess.draw_final(
                            &self.renderdevice, 
                            &mut self.queue,
                            & self.pipelines,
                            & self.geometrys,
                            &mut renderpass,
                            texture,
                            &texture_bind_group,
                            &final_targets,
                            &final_depth_and_stencil,
                            // &IDENTITY_MATRIX,
                            &[0.3535533845424652, 0.3535533845424652, 0., 0., -0.3535533845424652, 0.3535533845424652, 0., 0., 0., 0., 0.5, 0., 0., 0., 0., 1.],
                            0.0
                        )
                    },
                    None => Ok(false),
                }
            },
            Err(e) => {
                println!("{:?}", e);
                Err(e)
            },
        };

        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();

        // let new_time = SystemTime::now();
        // println!("{:?}", new_time.duration_since(last_time));
        Ok(())
    }

    fn clear(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView
    ) {
        let renderpass = encoder.begin_render_pass(
            &wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[
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
                ],
                depth_stencil_attachment: None,
            }
        );
    }
}
