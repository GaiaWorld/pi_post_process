/// 区域蒙版
#[derive(Clone, Copy, Debug)]
pub struct AreaMask {
    /// 区域横向起点
    pub start_x: f32,
    /// 区域横向终点
    pub end_x: f32,
    /// 区域纵向起点
    pub start_y: f32,
    /// 区域纵向终点
    pub end_y: f32,
    /// 过渡区间
    pub fade: f32,
}