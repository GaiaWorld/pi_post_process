use crate::prelude::SingleImageEffectResource;

use super::{FilterBrightness, BlurDualRenderer, BlurDual, CopyIntensity, FilterBrightnessRenderer, CopyIntensityRenderer, BlurDualRendererList};

/// 使用 DualBlur 的辉光 - 只能全屏效果
#[derive(Clone, Copy, Debug)]
pub struct BloomDual {
    /// 辉光扩散范围 - 像素值
    pub radius: u8,
    /// 辉光强度
    pub intensity: f32,
    /// 迭代次数 - 越高效果越柔和 - 通常 1 或 2 已足够
    pub iteration: u8,
    pub threshold: f32,
    pub threshold_knee: f32,
}

impl BloomDual {
    pub fn is_enabled(
        &self
    ) -> bool {
        self.radius > 0 && self.iteration > 0 && self.intensity > 0.
    }
}
impl BloomDual {
    fn default(resources: &SingleImageEffectResource) -> Self {
        Self { radius: 1, iteration: 1, intensity: 1.0, threshold: 0., threshold_knee: 0. }
    }
}

pub struct BloomDualRenderer {
    pub(crate) iteration: usize,
    pub(crate) brightness_filter: FilterBrightnessRenderer,
    pub(crate) blur_duals: BlurDualRendererList,
    pub(crate) copy: CopyIntensityRenderer,
    pub(crate) copy_intensity: CopyIntensityRenderer,
}
impl BloomDualRenderer {
    pub const MAX_LEVEL: usize = 4;
    pub fn new(bloom_dual: &BloomDual, resources: &SingleImageEffectResource) -> Self {
        let base = BlurDual { radius: bloom_dual.radius, iteration: bloom_dual.iteration, intensity: bloom_dual.intensity, simplified_up: true };
        let brightness_filter = FilterBrightnessRenderer { param: FilterBrightness { threshold: bloom_dual.threshold, threshold_knee: bloom_dual.threshold_knee }, uniform: resources.uniform_buffer()  };
        let copy = CopyIntensityRenderer { param: CopyIntensity::default(), uniform: resources.uniform_buffer() };
        let mut copy_intensity = CopyIntensityRenderer { param: CopyIntensity::default(), uniform: resources.uniform_buffer() };
        copy_intensity.param.intensity = bloom_dual.intensity;

        Self {
            iteration: Self::MAX_LEVEL.min(bloom_dual.iteration as usize),
            brightness_filter,
            blur_duals: BlurDualRendererList::new(&base, resources),
            copy,
            copy_intensity,
        }
    }
    
    pub fn update(&mut self, bloom_dual: &BloomDual) {
        let base = BlurDual { radius: bloom_dual.radius, iteration: bloom_dual.iteration, intensity: bloom_dual.intensity, simplified_up: true };
        self.blur_duals.update(&base);
        self.brightness_filter.param.threshold = bloom_dual.threshold;
        self.brightness_filter.param.threshold_knee = bloom_dual.threshold_knee;
        self.copy_intensity.param.intensity = bloom_dual.intensity;
    }
}