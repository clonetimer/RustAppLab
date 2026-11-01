# TextScope · 文本观察镜 · 分层提示

## 第 1 层

先预测中文文本的字节数与字符数，再写 stats。搜索返回原文引用。

## 第 2 层

用 tokens 产生词元；HashMap 累计，排序后 truncate。search 中 enumerate 的行号需要加 1。

## 第 3 层

排序比较器可先比较 b.1 与 a.1，再比较 a.0 与 b.0；返回 text.lines() 给出的 &str，不创建临时 lowercase 字符串。
