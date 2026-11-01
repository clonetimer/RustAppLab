# BadgeForge · SVG 作品卡生成器 · 分层提示

## 第 1 层

三个函数分别负责原始文本、图形内容、完整文档，不要把所有逻辑塞进 main。

## 第 2 层

先转义 & 再转义其它字符；渲染时借用 self 的字段；文档调用 element.render()。

## 第 3 层

document 不要接收特定 Badge 类型；保留 T: svg::Element。原始字符串包含颜色引号时使用两个 # 的分隔符。
