
use std::{time::SystemTime, sync::Arc};

pub use bevy::{
    app::{ prelude::*, PluginGroupBuilder }, core::prelude::*, ecs::prelude::*, hierarchy::prelude::*, input::{prelude::*, InputPlugin},
    log::prelude::*, math::prelude::*, reflect::prelude::*, time::prelude::*,
    utils::prelude::*, window::prelude::*,
    ecs::system::{CommandQueue, EntityCommands, SystemState, SystemParam}, prelude::{Deref, DerefMut},
    a11y::*,
    // winit::*,
};
use image::GenericImageView;
use pi_assets::{asset::Handle, mgr::AssetMgr};
pub use pi_atom::Atom;
use pi_bevy_render_plugin::SimpleInOut;
pub use pi_bevy_winit_window::*;
pub use pi_bevy_ecs_extend::prelude::*;
pub use pi_bevy_asset::{AssetMgrConfigs, AssetCapacity, ShareAssetMgr};
pub use pi_bevy_render_plugin::{
    PiRenderDevice, PiRenderQueue, PiRenderGraph, PiRenderWindow, PiRenderOptions, PiSafeAtlasAllocator, PiScreenTexture, PiRenderPlugin,
    node::*, RenderContext, GraphError, constant::{ render_state::*, texture_sampler::* }, 
};
use pi_null::Null;
use pi_postprocess::prelude::*;
use pi_share::{ShareRefCell, Share};
use pi_render::{
    asset::*,
    renderer::{
        vertex_buffer::*,
        sampler::*,
        texture::*,
        draw_obj::*,
    },
    rhi::{
        asset::*,
        pipeline::*,
        device::RenderDevice,
        RenderQueue,
    },
    components::view::target_alloc::*,
};
pub use pi_assets::asset::GarbageEmpty;
use pi_futures::BoxFuture;
use smallvec::SmallVec;
use wgpu::Extent3d;
use wgpu1::Backends;


#[derive(SystemParam)]
pub struct QueryParam<'w> (
    Res<'w, PiRenderWindow>,
    Res<'w, PiRenderDevice>,
    Res<'w, PiRenderQueue>,
    Res<'w, PiScreenTexture>,
    Res<'w, PiSafeAtlasAllocator>,
    ResMut<'w, TestPostprocess>,
    Res<'w, ResImageEffectResource>,
);


#[derive(SystemParam)]
pub struct RunQueryParam<'w> (
    Res<'w, PiRenderWindow>,
    Res<'w, PiRenderDevice>,
    Res<'w, PiRenderQueue>,
    Res<'w, PiScreenTexture>,
    Res<'w, PiSafeAtlasAllocator>,
    Res<'w, TestPostprocess>,
    Res<'w, ResImageEffectResource>,
);

#[derive(Resource, Deref, DerefMut)]
pub struct TestVB(pub VertexBufferAllocator);

#[derive(Resource)]
pub struct TestPostprocess {
    pub postprocess: PostProcess,
    pub asset_samplers: Share<AssetMgr::<SamplerRes>>,
    pub pipelines: Share<AssetMgr<RenderRes<RenderPipeline>>>,
    pub value_test: u8,
    pub lasttime: SystemTime,
    pub target_type: TargetType,
    pub draws: Vec<DrawObj>,
    pub diffuse_texture: Handle<TextureRes>,
    pub diffuse_size: wgpu::Extent3d,
    pub mask_texture: Handle<TextureRes>,
    pub mask_size: wgpu::Extent3d,
    pub asset_tex: Share<AssetMgr<TextureRes>>,
    pub frontdraws: Vec<PostProcessDraw>,
    pub resulttarget: Result<PostprocessTexture, ()>,

    pub viewport: (f32, f32, f32, f32),
}

#[derive(Resource, Deref)]
pub struct ResImageEffectResource(pub SingleImageEffectResource);

pub struct RenderNode;
impl Node for RenderNode {
    type Input = SimpleInOut;

    type Output = SimpleInOut;

    type BuildParam = QueryParam<'static>;
    type RunParam = RunQueryParam<'static>;

    fn build<'a>(
        &'a mut self,
        world: &'a mut World,
        param: &'a mut SystemState<Self::BuildParam>,
        context: RenderContext,
        input: &'a Self::Input,
        usage: &'a ParamUsage,
        id: NodeId,
        from: &'a [NodeId],
        to: &'a [NodeId],
    ) -> Result<Self::Output, String> {
        let time = pi_time::Instant::now();

            let param: QueryParam = param.get_mut(world);
            let (window, device, queue, final_render_target, atlas_allocator, mut postprocess, resources) = (param.0, param.1, param.2, param.3, param.4, param.5, param.6);

        if let Some(screen) = &final_render_target.0 {
            let view = if let Some(view) = &screen.view {
                view
            } else {
                return Err(String::from(""));
            };
            let size = if let Some(tex) = &screen.texture() {
                tex.texture.size()
            } else {
                return Err(String::from(""));
            };

            // calc
            {
                let src_texture = PostprocessTexture {
                    use_x: 0, // self.diffuse_size.width / 4,
                    use_y: 0, //self.diffuse_size.height / 4,
                    use_w: postprocess.diffuse_size.width, // / 2,
                    use_h: postprocess.diffuse_size.height, // / 2,
                    width: postprocess.diffuse_size.width,
                    height: postprocess.diffuse_size.height,
                    view: ETextureViewUsage::Tex(postprocess.diffuse_texture.clone()),
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                };
    
                let receive_width = size.width;
                let receive_height = size.height;
    
                let pipelines = postprocess.pipelines.clone();
                let delta_time = 16;
                let target_type = postprocess.target_type.clone();
                let target_format = wgpu::TextureFormat::Rgba8Unorm;
                let dst_size = (receive_width, receive_height);
    
                let draws = postprocess.postprocess.calc(delta_time, &device, &queue, src_texture, dst_size, &atlas_allocator, &resources, &pipelines, target_type, target_format);
                match draws {
                    Ok((draws, restarget)) => { postprocess.frontdraws = draws; postprocess.resulttarget = Ok(restarget) },
                    Err(_) => { postprocess.frontdraws.clear(); postprocess.resulttarget = Err(()) },
                }
            }
        }

        Ok(input.clone())
    }

    fn run<'a>(
        &'a mut self,
        world: &'a World,
        param: &'a mut SystemState<RunQueryParam<'static>>,
        _: RenderContext,
        mut commands: ShareRefCell<wgpu::CommandEncoder>,
        input: &'a Self::Input,
        _: &'a ParamUsage,
		_id: NodeId,
		_from: &[NodeId],
		_to: &[NodeId],
    ) -> BoxFuture<'a, Result<(), String>> {
        let time = pi_time::Instant::now();

        let param: RunQueryParam = param.get(world);
        let (window, device, queue, final_render_target, atlas_allocator, postprocess, resources) = (param.0, param.1, param.2, param.3, param.4, param.5, param.6);

        if let Some(screen) = &final_render_target.0 {
            let view = if let Some(view) = &screen.view {
                view
            } else {
                return Box::pin(
                    async move {
                        Err(String::from(""))
                    }
                );
            };
            let mut finalcolorformat = wgpu::TextureFormat::Rgba8Unorm;
            let size = if let Some(tex) = &screen.texture() {
                finalcolorformat = tex.texture.format();
                tex.texture.size()
            } else {
                return Box::pin(
                    async move {
                        Err(String::from(""))
                    }
                );
            };

            // draw_front
            {
        
                postprocess.postprocess.draw_front(
                    &mut commands,
                    &postprocess.frontdraws
                );
            }

            // draw_final
            {
                let receive_w = size.width - 200 as u32;
                let receive_h = size.height - 200 as u32;
                let receive_width = size.width;
                let receive_height = size.height;
        
                let dst = PostprocessTexture {
                    use_x: postprocess.value_test as u32,
                    use_y: postprocess.value_test as u32,
                    use_w: receive_w,
                    use_h: receive_h,
                    width: receive_width,
                    height: receive_height,
                    view: ETextureViewUsage::Temp(view.clone(), 0),
                    format: finalcolorformat,
                };
            
                let final_targets = create_target(finalcolorformat, get_blend_state(EBlend::Combine), wgpu::ColorWrites::ALL);
                let final_depth_and_stencil = None;

                let result = postprocess.resulttarget.clone();
        
                // postprocess.draws.clear();
                let _ = match result {
                    Ok(result) => {
                        // let matrix = [0.3535533845424652, 0.3535533845424652, 0., 0., -0.3535533845424652, 0.3535533845424652, 0., 0., 0., 0., 0.5, 0., 0., 0., 0., 1.];
                        let matrix = [1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.];
                        // renderpass.set_viewport(dst.use_x as f32, dst.use_y as f32, dst.use_w as f32, dst.use_h as f32, 0., 1.);
                        // println!("result {}, {}, {}, {}", result.use_x(), result.use_y(), result.use_w(), result.use_h());
                        if let Some(draw) = postprocess.postprocess.draw_final(
                            &device, 
                            &queue,
                            &matrix,
                            1.,
                            &atlas_allocator,
                            &result,
                            (dst.use_w(), dst.use_h()),
                            &resources,
                            // &IDENTITY_MATRIX,
                            &postprocess.pipelines,
                            final_targets,
                            final_depth_and_stencil,
                            postprocess.target_type.clone(),
                            finalcolorformat
                        ) {
                            let mut renderpass = commands.begin_render_pass(
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
            
                            let (x, y, w, h) = postprocess.viewport;
                            renderpass.set_viewport(x, y, w, h, 0., 1.);
                            draw.draw(&mut renderpass);
                        }
                    },
                    Err(_) => {
                        
                    },
                };
            }
        }

        return Box::pin(
            async move {
                Ok(())
            }
        );
    }
}


pub trait AddEvent {
	// 添加事件， 该实现每帧清理一次
	fn add_frame_event<T: Event>(&mut self) -> &mut Self;
}

impl AddEvent for App {
	fn add_frame_event<T: Event>(&mut self) -> &mut Self {
		if !self.world.contains_resource::<Events<T>>() {
			self.init_resource::<Events<T>>()
				.add_systems(Update, Events::<T>::update_system);
		}
		self
	}
}

pub struct PluginTest;
impl Plugin for PluginTest {
    fn build(&self, app: &mut App) {
        let mut colors_descriptor = SmallVec::<[TextureDescriptor;1]>::new();
        colors_descriptor.push(
            TextureDescriptor {
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
                base_mip_level: 0,
                base_array_layer: 0,
                array_layer_count: None,
                view_dimension: Some(wgpu::TextureViewDimension::D2),
            }
        );
        let mut atlas = app.world.get_resource_mut::<PiSafeAtlasAllocator>().unwrap().clone();
        let target_type = atlas.create_type(TargetDescriptor {
            colors_descriptor: colors_descriptor,
            need_depth: false,
            default_width: 2048,
            default_height: 2048,
            depth_descriptor: None
        });
        log::warn!("Create TargetType: {:?}", target_type);

        let renderdevice = app.world.get_resource::<PiRenderDevice>().unwrap().clone();
        let queue = app.world.get_resource::<PiRenderQueue>().unwrap().clone();
        

        let mut vballocator = VertexBufferAllocator::new(1024, 1000);
        let mut resources = SingleImageEffectResource::new(&renderdevice, &queue, &mut vballocator);
        let asset_samplers = AssetMgr::<SamplerRes>::new(GarbageEmpty(), false, 1024, 10000);
        let pipelines = AssetMgr::<RenderRes<RenderPipeline>>::new(GarbageEmpty(), false, 1024, 10000);

        let asset_tex = AssetMgr::<TextureRes>::new(GarbageEmpty(), false, 1024, 10000);  

        //// Texture
        let (diffuse_texture, diffuse_size) = texture(include_bytes!("./dialog_bg.png"), "./dialog_bg.png", &renderdevice, &queue, &asset_tex);
        let (mask_texture, mask_size) = texture(include_bytes!("./effgezi.png"), "./effgezi.png", &renderdevice, &queue, &asset_tex);


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
        EffectBlurGauss::setup(&renderdevice, &mut resources, &asset_samplers);
        EffectImageMask::setup(&renderdevice, &mut resources, &asset_samplers);
        EffectClipSdf::setup(&renderdevice, &mut resources, &asset_samplers);

        app.insert_resource(ResImageEffectResource(resources));
        app.insert_resource(TestPostprocess {
            postprocess: PostProcess::default(),
            value_test: 0,
            lasttime: std::time::SystemTime::now(),
            target_type,
            draws: vec![],
            asset_samplers,
            pipelines,
            diffuse_texture,
            diffuse_size,
            asset_tex,
            mask_texture,
            mask_size,
            viewport: (400. - 100., 300. - 100., 200., 200.),
            frontdraws: vec![],
            resulttarget: Err(()),
        });

        app.insert_resource(TestVB(VertexBufferAllocator::new(1024, 1000)));

        app.add_systems(Update, sys);

        let mut graphic = app.world.get_resource_mut::<PiRenderGraph>().unwrap();
        if let Ok(node) = graphic.add_node("TEST", RenderNode, NodeId::null()) {
            // graphic.add_depend(WindowRenderer::CLEAR_KEY, "TEST");
            // graphic.add_depend("TEST", WindowRenderer::KEY);
            graphic.set_finish("TEST", true);
        }
    }
}

pub fn texture(data: &[u8], key: &str, renderdevice: &RenderDevice, queue: &RenderQueue,  asset_tex: &Share<AssetMgr<TextureRes>>) -> (Handle<TextureRes>, Extent3d) {
    //// Texture
    let diffuse_bytes = data;
    let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
    let diffuse_rgba = diffuse_image.as_bytes();
    let dimensions = diffuse_image.dimensions();
    let texture_size = wgpu::Extent3d {
        width: dimensions.0,
        height: dimensions.1,
        depth_or_array_layers: 1,
    };
    let diffuse_texture = (**renderdevice).create_texture(
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
            bytes_per_row: Some(4 * dimensions.0),
            rows_per_image: Some(dimensions.1),
        },
        texture_size,
    );
    let texture_view = diffuse_texture.create_view(
        &wgpu::TextureViewDescriptor::default()
    ); 
    let key_img = KeyImageTexture::File(Atom::from(key), true);
    let diffuse_texture = asset_tex.insert(key_img.asset_u64(), TextureRes::new(texture_size.width, texture_size.height, (texture_size.width * texture_size.height * 4) as usize, texture_view, true, wgpu::TextureFormat::Rgba8Unorm)).unwrap();

    (diffuse_texture, texture_size)
}

pub fn sys(
    mut test: ResMut<TestPostprocess>,
) {
    let mut r = test.value_test;
    if r == 255 {
        r = 0;
    } else {
        r = r + 1;
    }
    test.value_test = r;
    // self.postprocess.color_balance = Some(ColorBalance { r: r, g: 255 - r, b: 255 });
    // self.postprocess.color_filter = Some(ColorFilter { r: r, g: 0, b: 0 });
    // test.postprocess.vignette = Some(Vignette { r: r, g: 0, b: 0, begin: 0.5, end: 1.5, scale: 1.0 });
    // self.postprocess.hsb = Some(HSB { hue: 0, brightness: 0, saturate: (r as i16 - 100) as i8 });
    // test.postprocess.blur_dual = Some(BlurDual { radius: 1, iteration: 2, intensity: 1.0f32, simplified_up: false });
    // test.postprocess.blur_direct = Some(BlurDirect { radius: 4, iteration: 10, direct_x: r as f32 / 255.0 * 2.0 - 1.0, direct_y: 1.0 });
    // test.postprocess.blur_radial = Some(BlurRadial { radius: 4, iteration: 10, center_x: 0., center_y: 0., start: 0.1, fade: 0.2  });
    // test.postprocess.blur_bokeh = Some(BlurBokeh { radius: 0.5, iteration: 8, center_x: 0., center_y: 0., start: 0.0, fade: 0.0  });

    test.postprocess.src_preimultiplied = true;
    if test.postprocess.horizon_glitch.is_none() {
        let mut hg = HorizonGlitch::default();
        hg.probability = 0.8;
        hg.max_count = 200;
        hg.min_count = 50;
        hg.max_size = 0.05;
        hg.min_size = 0.01;
        hg.strength = 0.2;
        test.postprocess.horizon_glitch = Some(hg);
    }

    // test.postprocess.bloom_dual = Some(BloomDual { radius: 1, iteration: 1, intensity: 1.0f32, threshold: r as f32 / 255.0, threshold_knee: 0.5 });

    test.postprocess.radial_wave = Some(RadialWave { aspect_ratio: true, start: r as f32 / 255.0, end: r as f32 / 255.0 + 0.5, center_x: 0., center_y: 0., cycle: 2, weight: 0.2  });
    
    // test.postprocess.filter_sobel = Some(FilterSobel{ size: 1, clip: r as f32 / 255.0, color: (255, 0, 0, 255), bg_color: (0, 0, 0, 125)  });

    // test.postprocess.copy = Some(CopyIntensity { intensity: 2.0f32, polygon: r / 10, radius: r as f32 / 255.0, angle: r as f32, bg_color: (0, 0, 0, 125) });

    test.postprocess.blur_gauss = Some(BlurGauss { radius: 3. });
    test.postprocess.blur_radial = Some(BlurRadial { radius: 5, iteration: 10, center_x: 0., center_y: 0., start: 0.1, fade: 0.4 });
    test.postprocess.blur_bokeh = Some(BlurBokeh { radius: 3., iteration: 5, center_x: 0., center_y: 0., start: 0.5, fade: 0.2 });

    // let src_texture = PostprocessTexture {
    //     use_x: 0, // self.diffuse_size.width / 4,
    //     use_y: 0, //self.diffuse_size.height / 4,
    //     use_w: test.mask_size.width, // / 2,
    //     use_h: test.mask_size.height, // / 2,
    //     width: test.mask_size.width,
    //     height: test.mask_size.height,
    //     view: ETextureViewUsage::Tex(test.mask_texture.clone()),
    //     format: wgpu::TextureFormat::Rgba8UnormSrgb,
    // };
    // test.postprocess.image_mask = Some(ImageMask { image: src_texture, factor: (r as f32 * 1.2) / 255.0, mode: EMaskMode::Clip, nearest_filter: false });

    let diff = 45.;
    let angle = 90.;
    let center_axis = angle * 0.5 + diff;
    let context = ClipSdf::cacl_context_rect(0., 0., 100., 100., 50., 50., 50., 50.);
    // let clip_sdf = ClipSdf::sector((0.5, 0.5), 0.5, (f32::sin(center_axis / 180. * 3.1415926), f32::cos(center_axis / 180. * 3.1415926)), (f32::sin(angle * 0.5 / 180. * 3.1415926), f32::cos(angle * 0.5 / 180. * 3.1415926)), context);
    // let clip_sdf = ClipSdf::circle((400., 300.), 50., (200., 200., 300., 200.));
    // test.postprocess.clip_sdf = Some(clip_sdf);

}

pub fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    let mut app = App::default();

    let width = 800;
    let height = 600;

    let mut opt = PiRenderOptions::default();
    opt.backends = Backends::VULKAN;
    app.insert_resource(opt);

	let mut window_plugin = WindowPlugin::default();
    if let Some(primary_window) = &mut window_plugin.primary_window {
        primary_window.resolution.set_physical_resolution(width, height);
    }
	let (w, eventloop) = {
		use pi_winit::platform::windows::EventLoopBuilderExtWindows;
		let event_loop = pi_winit::event_loop::EventLoopBuilder::new().with_any_thread(true).build();
		let window = pi_winit::window::Window::new(&event_loop).unwrap();
		(window, event_loop)
	};

    app.insert_resource(AssetMgrConfigs::default());
    app.add_plugins(
        (
            InputPlugin::default(),
            window_plugin,
        )
    );
    app.add_plugins(
        (
            AccessibilityPlugin,
            pi_bevy_winit_window::WinitPlugin::new(Arc::new(w)).with_size(width, height),
            pi_bevy_asset::PiAssetPlugin::default(),
            PiRenderPlugin::default(),
        )
    );

    app.add_plugins(PluginTest);
    
    loop { app.update(); }

}