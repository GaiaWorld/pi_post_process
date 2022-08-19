#[derive(Clone, Copy, Debug)]
/// 镜头虚光
pub struct Vignette {
    /// 沿半径的起点
    pub begin: f32,
    /// 沿半径的终点
    pub end: f32,
    /// 范围缩放系数
    pub scale: f32,
    /// 颜色 R
    pub r: u8,
    /// 颜色 G
    pub g: u8,
    /// 颜色 B
    pub b: u8,
}

impl Default for Vignette {
    fn default() -> Self {
        Self {
            begin: 1.,
            end: 1.,
            scale: 0.,
            r: 255,
            g: 255,
            b: 255
        }
    }
}

impl Vignette {
    pub fn is_enabled(
        &self
    ) -> bool {
        self.begin < 1.5
    }
}