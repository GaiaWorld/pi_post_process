/// 定向模糊
#[derive(Clone, Copy, Debug)]
pub struct BlurDirect {
    /// 模糊半径 - 像素
    pub radius: u8,
    /// * 迭代次数 - 数值越大效果越好, 性能越差 - 通常 6
    /// * 暂弃用, 渲染使用 8 次迭代
    pub iteration: u8,
    /// 方向 x 轴
    pub direct_x: f32,
    /// 方向 y 轴
    pub direct_y: f32,
}

impl BlurDirect {
    pub fn is_enabled(
        &self
    ) -> bool {
        self.radius > 0 && self.iteration > 0 && (self.direct_x > 0. || self.direct_y > 0.)
    }
}