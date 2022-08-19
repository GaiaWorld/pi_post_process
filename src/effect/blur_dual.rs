/// Dual 模糊
#[derive(Clone, Copy, Debug)]
pub struct BlurDual {
    /// 模糊半径 - 像素
    pub radius: u8,
    /// 迭代次数
    pub iteration: u8,
    /// Up 时的缩放强度
    pub intensity: f32,
    /// 是否简化 升采样流程
    pub simplified_up: bool,
}

impl Default for BlurDual {
    fn default() -> Self {
        Self { radius: 0, iteration: 0, intensity: 1.0, simplified_up: false }
    }
}

impl BlurDual {
    pub fn is_enabled(
        &self
    ) -> bool {
        self.radius > 0 && self.iteration > 0 && self.intensity > 0.0
    }
}