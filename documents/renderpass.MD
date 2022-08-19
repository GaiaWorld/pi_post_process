# Render Pass

* Create Render Pass
* Set Pipeline
* Set Viewport
* Set Scissor Rect

* RenderPass > Set Blend Constant

* RenderPass > Bind Group
* RenderPass > Set Vertex Buffer
* RenderPass > Set Index Buffer
* RenderPass > Draw Indexed
* RenderPass > Draw

## 3D

* 材质自定义拓展
  * 动态编译
    * Component
    * System
* 渲染队列控制
  * 剔除条件
    * 相机范围剔除
    * Layer 剔除
  * 渲染筛选条件
    * 物体类型
      * 不透明物体
      * 天空盒
      * 半透明物体
    * 阴影类型
      * 生成阴影
    * 排序控制
      * RenderQuene 排序
      * 距离排序

* example
  * 开启相机范围剔除 开启 Layer 剔除
  * 渲染阴影
    * 第一步 
        * 设置渲染队列控制 - 不透明物体
        * 设置排序控制 - 距离排序
  * 正常渲染
    * 第一步 
        * 设置渲染队列控制 - 不透明物体
        * 设置排序控制 - 距离排序
    * 第二步 
        * 设置渲染队列控制 - 天空盒
        * 设置排序控制 - RenderQuene 排序
    * 第三步 
        * 设置渲染队列控制 - 半透明物体
        * 设置排序控制 - RenderQuene 排序
