#[derive(Clone, Copy, Debug)]
pub struct CopyIntensity {
    /// 拷贝时强度因子
    pub intensity: f32,
    /// * 多边形边数
    ///   * 小于2 无效
    ///   * 2 - 圆形
    ///   * 3 - 三角形
    ///   * ...
    ///   * 255 - 255 边形
    pub polygon: u8,
    /// * 多边形外切圆半径
    pub radius: f32,
    /// * 多边形旋转角度
    pub angle: f32,
    /// * 形状之外颜色
    pub bg_color: (u8, u8, u8, u8),
}

impl Default for CopyIntensity {
    fn default() -> Self {
        Self {
            intensity: 1.0,
            polygon: 0,
            radius: 1.0,
            angle: 0.0,
            bg_color: (0, 0, 0, 0)
        }
    }
}