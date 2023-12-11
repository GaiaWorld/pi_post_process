
use std::sync::Arc;

use pi_wy_rng::WyRng;
use rand::{Rng, SeedableRng};

use crate::prelude::{ImageEffectUniformBuffer, SingleImageEffectResource, ImageEffectInstanceBufferOffset};

use super::{CopyIntensityRenderer, CopyIntensity};

#[derive(Clone, Debug)]
/// 水平故障纹
pub struct HorizonGlitch {
    /// 一次故障最大条纹尺寸 - [0, 1]
    pub max_size: f32,
    /// 一次故障最小条尺寸 - [0, 1]
    pub min_size: f32,
    /// 一次故障最多条纹数 - [0, 32]
    pub max_count: u8,
    /// 一次故障最少条纹数 - [0, 255]
    pub min_count: u8,
    /// 一次故障持续时间 - ms - u16 最大 65000 ms
    pub lifetime: u16,
    /// 故障生成几率 - [0, 1]
    pub probability: f32,
    /// 故障扭曲强度 - [0, 1]
    pub strength: f32,
    /// 故障边界过渡因子
    pub fade: f32,
    /// 是否向上
    pub is_up: bool,
    /// 当前年龄
    life: u16,
    items: Vec<(f32, f32)>,
}

impl HorizonGlitch {
    pub fn is_enabled(
        &self
    ) -> bool {
        (self.max_size > 0. || self.min_size > 0.) && (self.max_count > 0 || self.min_count > 0) && self.lifetime > 0 && self.probability > 0. && self.strength > 0.
    }

    pub fn update(&mut self, delta_time: u64) {

        if self.life > self.lifetime {
            self.life = 0;
        }

        if self.life == 0 {
            self.items.clear();
            
            let mut rng = WyRng::seed_from_u64(1000);

            let scale = if self.is_up { -1.0 } else { 1.0 };
            
            let count = (self.max_count - self.min_count) as f32 * f32::max(0., self.probability - rng.gen_range(0.0..1.0f32) );
            let count = self.min_count as usize + count as usize;
            for _ in 0..count {
                let value = rng.gen_range(0.0..1.0f32) - scale;
                let size = self.min_size + (self.max_size - self.min_size) * rng.gen_range(0.0..1.0f32);
                self.items.push((value, size));
            }
        }

        // println!(">>>>>> {}", self.life);
        self.life += delta_time as u16;
    }

    pub fn get_items(&self) -> Vec<(f32, f32)> {
        let mut result: Vec<(f32, f32)> = Vec::default();

        let count = self.items.len();

        let scale = if self.is_up { -1.0 } else { 1.0 };

        let v_distance = self.life as f32 / self.lifetime as f32 * 2.0 * scale; // 预设一屏故障总的移动范围为 2.0 屏

        for i in 0..count {
            let temp = self.items.get(i).unwrap();
            result.push((temp.0 + v_distance, temp.1));
            // println!("{}", temp.0 + v_distance);
        }
            
        result
    }
}

impl Default for HorizonGlitch {
    fn default() -> Self {
        Self { max_size: 0.1, min_size: 0.05, max_count: 6, min_count: 2, lifetime: 5000, probability: 0.5, strength: 0.05, fade: 0.05, life: 0, items: Vec::new(), is_up: true }
    }
}
pub struct HorizonGlitchRenderer {
    /// 故障扭曲强度 - [0, 1]
    pub(crate) strength: f32,
    /// 故障边界过渡因子
    pub(crate) fade: f32,
    pub(crate) instance: Option<ImageEffectInstanceBufferOffset>,
    pub(crate) uniform: Arc<ImageEffectUniformBuffer>,
    pub(crate) copy: CopyIntensityRenderer,
}
impl HorizonGlitchRenderer {
    pub fn new(param: &HorizonGlitch, resource: &SingleImageEffectResource) -> Self {
        Self {
            strength: param.strength,
            fade: param.fade,
            uniform: resource.uniform_buffer(),
            instance: resource.instance_range(),
            copy: CopyIntensityRenderer::new(&CopyIntensity::default(), resource),
        }
    }
}
impl super::TEffectForBuffer for HorizonGlitchRenderer {
    fn buffer(&self, 
        _: u64,
        geo_matrix: &[f32],
        tex_matrix: (f32, f32, f32, f32),
        alpha: f32, depth: f32,
        _device: &pi_render::rhi::device::RenderDevice,
        queue: &pi_render::rhi::RenderQueue,
        _: (u32, u32),
        _: (u32, u32),
        src_premultiplied: bool,
        dst_premultiply: bool,
    ) -> &pi_render::rhi::buffer::Buffer {
        let mut temp = vec![

        ];
        geo_matrix.iter().for_each(|v| { temp.push(*v) });
        temp.push(tex_matrix.0);
        temp.push(tex_matrix.1);
        temp.push(tex_matrix.2);
        temp.push(tex_matrix.3);

        temp.push(self.strength);
        temp.push(self.fade);
        temp.push(depth);
        temp.push(alpha);

        if src_premultiplied { temp.push(1.); } else { temp.push(0.); }
        if dst_premultiply { temp.push(1.); } else { temp.push(0.); }
        temp.push(0.);
        temp.push(0.);

        queue.write_buffer(self.uniform.buffer(), 0, bytemuck::cast_slice(&temp));
        self.uniform.buffer()
    }
}