use crate::{geometry::{Geometry, vertex_buffer_layout::{EVertexBufferLayout, get_vertex_buffer_layouts}}, material::{blend::{get_blend_state, EBlend}, shader::{Shader, EPostprocessShader}, tools::{ effect_render, get_uniform_bind_group, VERTEX_MATERIX_SIZE, DIFFUSE_MATERIX_SIZE, SimpleRenderExtendsData, UniformBufferInfo, TextureScaleOffset}}, effect::{hsb::HSB, color_balance::ColorBalance, color_scale::ColorScale, vignette::Vignette, color_filter::ColorFilter}, postprocess_pipeline::{PostProcessMaterialMgr, PostprocessMaterial, PostprocessPipeline}, temprory_render_target:: PostprocessTexture };

use super::{renderer::{Renderer, ERenderParam}};

pub const COLOR_EFFECT_VS: &'static str = "ColorEffect_vs";
pub const COLOR_EFFECT_FS: &'static str = "ColorEffect_fs";
pub const COLOR_EFFECT_PIPELINE: &'static str = "ColorEffect_pipeline";

const UNIFORM_PARAM_SIZE: u64 = 25 * 4;

pub struct ColorEffectRenderer {
    pub effect: Renderer,
}

impl ColorEffectRenderer {
    const UNIFORM_BIND_0_VISIBILITY: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT;
    pub fn ubo_info(device: &wgpu::Device) -> UniformBufferInfo {
        let o1 = UniformBufferInfo::calc(device, VERTEX_MATERIX_SIZE);
        let o2 = UniformBufferInfo::calc(device, UNIFORM_PARAM_SIZE);
        let o3 = UniformBufferInfo::calc(device, DIFFUSE_MATERIX_SIZE);
        let ubo_info: UniformBufferInfo = UniformBufferInfo {
            offset_vertex_matrix: 0,
            size_vertex_matrix: o1,
            offset_param: 0 + o1,
            size_param: o2,
            offset_diffuse_matrix: 0 + o1 + o2,
            size_diffuse_matrix: o3,
            uniform_size: 0 + o1 + o2 + o3,
        };
        ubo_info
    }
    pub fn check_pipeline(
        device: &wgpu::Device,
        material: &mut PostprocessMaterial,
        geometry: & Geometry,
        targets: &[Option<wgpu::ColorTargetState>],
        primitive: wgpu::PrimitiveState,
        depth_stencil: Option<wgpu::DepthStencilState>
    ) {
        let vertex_layouts = get_vertex_buffer_layouts(EVertexBufferLayout::Position2D, geometry);

        material.check_pipeline(
            "ColorEffect", device,
            &vertex_layouts,
            targets,
            Self::UNIFORM_BIND_0_VISIBILITY,
            primitive, depth_stencil,
            &Self::ubo_info(device),
        );
    }

    pub fn get_renderer(
        device: &wgpu::Device,
    ) -> Renderer {
        let ubo_info = Self::ubo_info(device);
        // println!("{:?}", ubo_info);

        let uniform_bind_group_layout = PostprocessPipeline::uniform_bind_group_layout(
            device, 
            Self::UNIFORM_BIND_0_VISIBILITY,
            &ubo_info,
        );
    
        let (uniform_buffer, uniform_bind_group) = get_uniform_bind_group(
            device,
            &uniform_bind_group_layout,
            &ubo_info
        );
    
        Renderer {
            uniform_buffer,
            uniform_bind_group,
            ubo_info,
        }
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
    postprocess_pipelines: &'a PostProcessMaterialMgr,
    renderer: &'a ColorEffectRenderer,
    geometry: &'a Geometry,
    texture_scale_offset: &TextureScaleOffset,
    texture_bind_group: &'a wgpu::BindGroup,
    target: &wgpu::ColorTargetState,
    depth_stencil: &Option<wgpu::DepthStencilState>,
    matrix: & [f32],
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

    let primitive: wgpu::PrimitiveState = wgpu::PrimitiveState::default();
    let pipeline = postprocess_pipelines.get_material(EPostprocessShader::ColorEffect).get_pipeline(target, &primitive, depth_stencil);

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
            color_balance.is_some() && color_balance.as_ref().unwrap().is_enabled()
            , (hsb.is_some() && hsb.as_ref().unwrap().is_enabled()) 
            , color_scale.is_some() && color_scale.as_ref().unwrap().is_enabled()
            , vignette.is_some() && vignette.as_ref().unwrap().is_enabled()
            , color_filter.is_some() && color_filter.as_ref().unwrap().is_enabled()
        )
    );

    effect_render(
        queue,
        renderpass,
        geometry,
        texture_scale_offset,
        texture_bind_group,
        &renderer.uniform_buffer,
        renderer.ubo_info.offset_diffuse_matrix,
        &renderer.uniform_bind_group,
        &pipeline.pipeline,
    )
}
