# pi_postprocess

## 已支持的效果

* Copy Intensity
  * 拷贝
    * 附带多边形裁剪 & 圆形裁剪
* HSB
  * 色彩变换
* Vignette
  * 径向镜头氛围
* Color Balance
  * 色彩平衡
* Color Scale
  * 色阶
* Color Filter
  * 色彩过滤
* Dual Blur
  * Dual 快速模糊
* Direct Blur
  * 定向模糊
* Radial Blur
  * 径向模糊
* Bokeh Blur
  * 散景模糊
* Bloom Dual
  * 基于 Dual Blur 的辉光
* Radial Wave
  * 径向波纹扭曲
* Horizon Glitch
  * 水平故障纹
* Filter Sobel
  * Sobel 算子的特征提取
    * 提取出颜色边界

## 内部效果先后顺序

* Color Effect
  * HSB, Color Balance, Color Scale, Vignette, Color Filter
* Blur
  * Blur Dual
  * Blur Direct
  * Blur Radial
  * Blur Bokeh
* Bloom
  * Bloom Dual
* 扭曲
  * Radial Wave
  * Horizon Glitch
* Filter
  * Filter Sobel
* Copy
  * Copy Intensity 

## 使用

* 创建 PostProcess 数据
* 创建 PostProcessRenderer 渲染管理
  * 全局唯一
* 调用接口 PostProcess.draw()

## 网格说明

* 渲染范围对应网格顶点坐标范围为
  * [-0.5, 0.5]