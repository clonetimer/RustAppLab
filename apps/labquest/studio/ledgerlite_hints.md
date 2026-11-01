# LedgerLite · 本地生活账本 · 分层提示

## 第 1 层

先把金额理解成分，后面不要再换回浮点进行累计。

## 第 2 层

整数部分乘 100；一位小数乘 10；两位直接作分。日期校验函数已经提供。

## 第 3 层

checked_mul/checked_add 返回 Option，可用 ok_or_else 转成 Result；用 BTreeMap 的 entry 更新分类，不要覆写此前金额。
