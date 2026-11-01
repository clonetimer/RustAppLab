# NetFetch 三级提示（只看当前步骤）

1. **Step 1**：先 `strip_prefix("http://")`，再找第一个 `/` 分开主机和路径。没有路径时用 `/`，端口解析用 `parse::<u16>()`。错误始终走 `Err`。
2. **Step 2**：`windows(4).position(|x| x == b"\r\n\r\n")` 可以找到头部结束位置；先检查状态，再找 Content-Length，最后核对 `body.len()`。
3. **Step 3**：先 `request(url)?` 得到已经校验的正文，然后用 `OpenOptions::new().write(true).create_new(true)` 创建新文件；写入失败后清理半成品。不要用 `File::create()` 覆盖旧文件。

卡住时先查看第一条编译错误。修改一项再运行 `studio check <目录> <当前步>`，不要直接照抄完整参考实现。
