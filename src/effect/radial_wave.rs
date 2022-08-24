
#[derive(Clone, Copy, Debug)]
/// 径向波纹扭曲
pub struct RadialWave {
    /// 是否应用纵横比 - 应用则为 圆形， 否则随纵横比形变
    pub aspect_ratio: bool,
    /// 扭曲半径起点 - 渲染范围 [-1, 1]
    pub start: f32,
    /// 扭曲半径终点 - 渲染范围 [-1, 1]
    pub end: f32,
    /// 扭曲中心点坐标 x - 渲染范围 [-1, 1]
    pub center_x: f32,
    /// 扭曲中心点坐标 y - 渲染范围 [-1, 1]
    pub center_y: f32,
    /// 波纹周期数
    pub cycle: u8,
    /// 扭曲强度
    pub weight: f32,
}

impl RadialWave {
    pub fn is_enabled(&self) -> bool {
        self.cycle > 0 && self.weight > 0.0 && (
            (self.start - self.center_x).abs() < 3.
            && (self.start - self.center_y).abs() < 3.
        )
    }  
}