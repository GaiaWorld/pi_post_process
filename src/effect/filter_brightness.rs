#[derive(Clone, Copy, Debug)]
pub struct FilterBrightness {
    /// 检测阈值
    pub threshold: f32,
    /// 检测阈值的变化曲线参数
    pub threshold_knee: f32,
}

impl FilterBrightness {
    pub fn is_enabled(&self) -> bool {
        true
    }
}

impl Default for FilterBrightness {
    fn default() -> Self {
        Self {
            threshold: 0.5,
            threshold_knee: 1.0,
        }
    }
}