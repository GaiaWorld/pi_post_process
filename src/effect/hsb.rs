#[derive(Clone, Copy, Debug)]
/// 颜色变换
pub struct HSB {
    /// [-180, 180]
    pub hue: i16,
    /// [-100, 100]
    pub saturate: i8,
    /// [-100, 100]
    pub brightness: i8,
}

impl Default for HSB {
    fn default() -> Self {
        Self {
            hue: 0,
            saturate: 0,
            brightness: 0,
        }
    }
}


impl HSB {
    pub fn is_enabled(
        &self
    ) -> bool {
        self.hue != 0 || self.saturate != 0 || self.brightness != 0
    }
}