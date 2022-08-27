
/// 支持 4 个
#[derive(Debug, Clone, Copy)]
pub enum EDepthStencilFormat {
    None,
    Depth32Float,
    Depth24Plus,
    Depth24PlusStencil8,
}