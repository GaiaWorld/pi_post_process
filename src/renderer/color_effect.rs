use crate::{geometry::{Geometry, vertex_buffer_layout::EVertexBufferLayout}, material::{target_format::{get_target_texture_format, ETexutureFormat}, blend::{get_blend_state, EBlend}, shader::{Shader, EPostprocessShader}, tools::{ effect_render, get_uniform_bind_group, VERTEX_MATERIX_SIZE, DIFFUSE_MATERIX_SIZE, SimpleRenderExtendsData}, pipeline::{Pipeline, UniformBufferInfo}}, effect::{hsb::HSB, color_balance::ColorBalance, color_scale::ColorScale, vignette::Vignette, color_filter::ColorFilter}, postprocess_pipeline::PostProcessPipeline, temprory_render_target:: EPostprocessTarget };

use super::{renderer::{Renderer, ERenderParam}};

pub const COLOR_EFFECT_VS: &'static str = "ColorEffect_vs";
pub const COLOR_EFFECT_FS: &'static str = "ColorEffect_fs";
pub const COLOR_EFFECT_PIPELINE: &'static str = "ColorEffect_pipeline";

const UNIFORM_PARAM_SIZE: u64 = 25 * 4;

pub struct ColorEffectRenderer {
    pub effect: Renderer,
}

pub fn get_pipeline(
    key: u128,
    vertex_layouts: &Vec<wgpu::VertexBufferLayout>,
    device: &wgpu::Device,
    shader: &Shader,
    blend: EBlend,
    format: ETexutureFormat,
) -> Pipeline {
    
    let vs_state = wgpu::VertexState {
        module: &shader.vs_module,
        entry_point: "main",
        buffers: &vertex_layouts,
    };

    let fs_state = wgpu::FragmentState {
        module: &shader.fs_module,
        entry_point: "main",
        targets: &[
            wgpu::ColorTargetState {
                format: get_target_texture_format(format),
                blend: get_blend_state(blend),
                // blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            }
        ],
    };

    Pipeline::new(
        key,
        "ColorEffect",
        vs_state,
        fs_state,
        device,
        wgpu::ShaderStages::FRAGMENT,
    )
}

pub fn get_renderer(
    device: &wgpu::Device,
    pipeline: &Pipeline,
) -> Renderer {
    let ubo_info: UniformBufferInfo = UniformBufferInfo {
        offset_vertex_matrix: 0,
        size_vertex_matrix: VERTEX_MATERIX_SIZE,
        offset_param: device.limits().min_uniform_buffer_offset_alignment as u64,
        size_param: UNIFORM_PARAM_SIZE,
        offset_diffuse_matrix: device.limits().min_uniform_buffer_offset_alignment as u64 * 2,
        size_diffuse_matrix: DIFFUSE_MATERIX_SIZE,
        uniform_size: device.limits().min_uniform_buffer_offset_alignment as u64 * 3,
    };

    let (uniform_buffer, uniform_bind_group) = get_uniform_bind_group(
        device,
        &pipeline.uniform_bind_group_layout,
        &ubo_info
    );

    Renderer {
        pipeline_key: pipeline.key,
        uniform_buffer,
        uniform_bind_group,
        ubo_info,
    }
}

pub fn update_uniform(
    renderer: &Renderer,
    queue: &wgpu::Queue,
    hsb: &HSB,
    color_balance: &ColorBalance,
    color_scale: &ColorScale,
    vignette: &Vignette,
    color_filter: &ColorFilter,
    flags: (bool, bool, bool, bool, bool)
) {
    // println!("{}, {}, {}", color_balance.r, color_balance.g, color_balance.b);
    // println!("{}, {}, {}", color_filter.r, color_filter.g, color_filter.b);

    let balance_center = u8::max(u8::min(color_balance.r, color_balance.g), u8::min(u8::max(color_balance.r, color_balance.g), color_balance.b));
    let balance_r = f32::powf(0.5, (color_balance.r as f32 - balance_center as f32) / 100.0);
    let balance_g = f32::powf(0.5, (color_balance.g as f32 - balance_center as f32) / 100.0);
    let balance_b = f32::powf(0.5, (color_balance.b as f32 - balance_center as f32) / 100.0);

    // println!("{}", balance_center);
    // println!("{}, {}, {}", balance_r, balance_g, balance_b);

    let flag1 = if flags.0 { 1.0 } else { 0.0 };
    let flag2 = if flags.1 { 1.0 } else { 0.0 };
    let flag3 = if flags.2 { 1.0 } else { 0.0 };
    let flag4 = if flags.3 { 1.0 } else { 0.0 };
    let flag5 = if flags.4 { 1.0 } else { 0.0 };

    queue.write_buffer(&renderer.uniform_buffer, renderer.ubo_info.offset_param, bytemuck::cast_slice(&[
        flag1,
        flag2,
        flag3,
        flag4,
        flag5,
        hsb.hue as f32 / 360.0,
        hsb.saturate as f32 / 100.0,
        hsb.brightness as f32 / 100.0,
        balance_r,
        balance_g,
        balance_b,
        vignette.begin,
        vignette.end,
        vignette.scale,
        vignette.r as f32 / 255.0,
        vignette.g as f32 / 255.0,
        vignette.b as f32 / 255.0,
        color_scale.shadow_in as f32 / 255.0,
        color_scale.shadow_out as f32 / 255.0,
        color_scale.mid,
        color_scale.highlight_in as f32 / 255.0,
        color_scale.highlight_out as f32 / 255.0,
        color_filter.r as f32 / 255.0,
        color_filter.g as f32 / 255.0,
        color_filter.b as f32 / 255.0,
    ]));
}

pub fn color_effect_render<'a>(
    hsb: & Option<HSB>,
    color_balance: & Option<ColorBalance>,
    color_scale: & Option<ColorScale>,
    vignette: & Option<Vignette>,
    color_filter: & Option<ColorFilter>,
    device: & wgpu::Device,
    queue: &  wgpu::Queue,
    renderpass: & mut wgpu::RenderPass<'a>, 
    format: ETexutureFormat,
    postprocess_pipelines: &'a PostProcessPipeline,
    renderer: &'a ColorEffectRenderer,
    texture_bind_group: &'a wgpu::BindGroup,
    image_effect_geo: &'a Geometry,
    resource: & EPostprocessTarget,
    blend: EBlend,
    matrix: & [f32; 16],
    extends: SimpleRenderExtendsData,
) {
    let renderer = &renderer.effect;

    let _hsb: HSB = if hsb.is_some() {
        hsb.unwrap()
    } else {
        HSB::default()
    };
    let _color_balance: ColorBalance = if color_balance.is_some() {
        color_balance.unwrap()
    } else {
        ColorBalance::default()
    };
    let _color_scale: ColorScale = if color_scale.is_some() {
        color_scale.unwrap()
    } else {
        ColorScale::default()
    };
    let _vignette: Vignette = if vignette.is_some() {
        vignette.unwrap()
    } else {
        Vignette::default()
    };
    let _color_filter: ColorFilter = if color_filter.is_some() {
        color_filter.unwrap()
    } else {
        ColorFilter::default()
    };

    // let image_effect_geo = postprocess_renderer.get_geometry(device);
    let pipeline = postprocess_pipelines.get_pipeline(
        EPostprocessShader::ColorEffect,
        EVertexBufferLayout::Position2D,
        blend,
        format,
    );

    let mut data = matrix.to_vec(); data.push(extends.depth); data.push(extends.alpha); 
    queue.write_buffer(&renderer.uniform_buffer, renderer.ubo_info.offset_vertex_matrix, bytemuck::cast_slice( &data ));
    update_uniform(
        renderer,
        &queue,
        &_hsb,
        &_color_balance,
        &_color_scale,
        &_vignette,
        &_color_filter,
        (
            (hsb.is_some() && hsb.as_ref().unwrap().is_enabled()) 
            , color_balance.is_some() && color_balance.as_ref().unwrap().is_enabled()
            , color_scale.is_some() && color_scale.as_ref().unwrap().is_enabled()
            , vignette.is_some() && vignette.as_ref().unwrap().is_enabled()
            , color_filter.is_some() && color_filter.as_ref().unwrap().is_enabled()
        )
    );

    effect_render(
        device,
        queue,
        renderpass,
        image_effect_geo,
        resource,
        texture_bind_group,
        &pipeline.texture_bind_group_layout,
        &renderer.uniform_buffer,
        renderer.ubo_info.offset_diffuse_matrix,
        &renderer.uniform_bind_group,
        &pipeline.pipeline,
        Some("ColorEffect")
    )
}
