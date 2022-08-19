#[derive(Clone, Copy, Debug)]
pub struct FilterSobel {
    /// 检测范围 - 像素数目
    pub size: u8,
    /// 检测阈值
    pub clip: f32,
    /// 检测结果颜色
    pub color: (u8, u8, u8, u8),
    /// 背景色
    pub bg_color: (u8, u8, u8, u8),
}

impl FilterSobel {
    pub fn is_enabled(&self) -> bool {
        self.size > 0 && self.clip > 0.
    }
}

impl Default for FilterSobel {
    fn default() -> Self {
        Self {
            size: 1,
            clip: 0.5,
            color: (255, 255, 255, 255),
            bg_color: (0, 0, 0, 0),
        }
    }
}