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

impl Default for BloomDual {
    fn default() -> Self {
        Self { radius: 1, iteration: 1, intensity: 1.0, threshold: 0., threshold_knee: 0. }
    }
}