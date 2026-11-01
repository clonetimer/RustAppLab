# ServiceDesk 三级提示（只看当前步骤）

1. **Step 1**：先拒绝控制字符，再 `trim()`，通过 `chars().count()` 验证长度。函数返回拥有数据的 `String`。
2. **Step 2**：`match action`。添加分支调用第一步验证，再 `tickets.push(...)`；关闭分支用 `iter_mut().find(...)` 查找，重复关闭返回错误。
3. **Step 3**：用 `json_string(&ticket.title)` 处理字符串；用 `enumerate()` 决定何时添加逗号，最后追加列表总数。网页用 `textContent`，不要把标题塞进 `innerHTML`。

可先查看 `src/http.rs` 了解请求流向，但不要修改它来绕开业务测试。
