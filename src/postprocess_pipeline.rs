use pi_hash::XHashMap;

use crate::{material::{shader::{Shader, EPostprocessShader, MOVE_E_POSTPROCESS_SHADER, get_shader}, blend::{EBlend, MOVE_E_BLEND}, target_format::{ETexutureFormat, MOVE_E_TARGET_FORMAT}, pipeline::Pipeline}, geometry::{vertex_buffer_layout::{EVertexBufferLayout, MOVE_E_VERTEX_BUFFER_LAYOUT, get_vertex_buffer_layouts}, Geometry}, renderer::renderer};


pub struct PostProcessPipeline {
    shaders: XHashMap<u128, Shader>,
    pipelines: XHashMap<u128, Pipeline>,
}

impl PostProcessPipeline {
    pub fn new() -> Self {
        Self {
            shaders: XHashMap::default(),
            pipelines: XHashMap::default(),
        }
    }
    pub fn gen_key(
        shader: EPostprocessShader,
        vertex_buffer_key: EVertexBufferLayout,
        blend: EBlend,
        target_format: ETexutureFormat,
    ) -> u128 {
        let mut move_number: u128 = 1;
        let mut result = 0;

        result += shader as u128 * move_number;
        move_number *= MOVE_E_POSTPROCESS_SHADER;

        result += vertex_buffer_key as u128 * move_number;
        move_number *= MOVE_E_VERTEX_BUFFER_LAYOUT;

        result += blend as u128 * move_number;
        move_number *= MOVE_E_BLEND;
        
        result += target_format as u128 * move_number;
        move_number *= MOVE_E_TARGET_FORMAT;

        result
    }

    pub fn check_pipeline(
        &mut self,
        device: &wgpu::Device,
        geometry: &Geometry,
        shader_key: EPostprocessShader,
        vertex_buffer_key: EVertexBufferLayout,
        blend: EBlend,
        target_format: ETexutureFormat,
    ) -> &Pipeline {
        let shader = self.shaders.get(&(shader_key as u128));
        if shader.is_none() {
            let shader = get_shader(device, shader_key);
            self.shaders.insert(shader_key as u128, shader);
        }
        let shader = self.shaders.get(&(shader_key as u128)).unwrap();

        let render_key = PostProcessPipeline::gen_key(shader_key, vertex_buffer_key, blend, target_format);
        let pipeline = self.pipelines.get(&render_key);
        if pipeline.is_none() {
            let pipeline = renderer::get_pipeline(
                render_key,
                &get_vertex_buffer_layouts(vertex_buffer_key, geometry),
                device,
                shader_key,
                shader,
                blend,
                target_format,
            );
            self.pipelines.insert(render_key as u128, pipeline);
        }

        self.pipelines.get(&(render_key as u128)).unwrap()
    }

    pub fn get_pipeline(
        &self,
        shader_key: EPostprocessShader,
        vertex_buffer_key: EVertexBufferLayout,
        blend: EBlend,
        target_format: ETexutureFormat,
    ) -> &Pipeline {
        let render_key = PostProcessPipeline::gen_key(shader_key, vertex_buffer_key, blend, target_format);
        let pipeline = self.pipelines.get(&(render_key as u128)).unwrap();

        pipeline
    }

}