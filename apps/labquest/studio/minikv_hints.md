# MiniKV 分层提示

先记录自己的预期，卡住后再展开当前阶段；不要一次读完整参考源码。

## 第 1 步
一级：空字符串、UTF-8 字节长度、首尾空白、控制字符是四项不同检查。
二级：`str::len()` 是字节数；比较 `trim()` 的结果，但不要把 trim 后的键直接存进去。
三级：合法时返回 `Ok(())`；非法时返回 `KvError::InvalidKey`。不需要创建文件。

## 第 2 步
一级：Get、Put、Delete 中哪两种会改变状态？哪一种应该在访问日志之前返回？
二级：用 `match &operation` 先借用参数检查，再调用 `append(&operation)?`，最后消耗 operation 更新 map。
三级：`BTreeMap::insert` 的返回值能告诉你是否替换；删除不存在的键应该先报错；日志失败不能改变内存。

## 第 3 步
一级：inner 已经从 prefix 开始遍历；下一个键不匹配时，后面还能再出现匹配键吗？
二级：调用 `self.inner.next()`；用 `starts_with` 决定返回引用或设置 finished。
三级：返回 `key.as_str()` / `value.as_str()`，不要在 next 内创建临时 String 再借用它。

## 额外实验：E0502
把 scan 收集成 `Vec<String>` 之后再执行删除。不是所有 clone 都错误；只在确实需要跨越借用边界时拥有数据。
