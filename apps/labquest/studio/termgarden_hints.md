# TermGarden 分层提示

先预测状态，再只看卡住的那一步。这里不给完整业务函数。

## 第 1 步
方向：输入先去空白，`add ` 有数据，其余命令是固定名字。
结构：先 `if let Some(title) = line.strip_prefix("add ")`，剩余使用 match。
检查：valid_title 检查内容；事件拥有标题，需要 `to_owned()`，不是返回对输入的借用。

## 第 2 步
方向：先 Add，再 Next/Prev，之后 Toggle/Filter，最后 Quit。别边可变遍历 Vec 边向 Vec push。
结构：可见 ID 列表来自 visible_ids；位置加减在列表长度内循环，先处理长度为零。
检查：任何错误之前不修改状态。被筛掉的当前项要重新定位为第一个可见 ID；没有可见项时 None。

## 第 3 步
方向：用 String 累积输出，所有数据都来自传入的 &Garden。
结构：先统计 done，决定 seed/sprout/bloom，再遍历可见任务。
检查：标题、选中符、完成符都随输入变化；显示函数不能推进任务状态。Renderer 是另一层运行时界面选择，不必重写业务模型。
