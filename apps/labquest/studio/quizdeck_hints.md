# QuizDeck · 终端知识闯关 · 分层提示

## 第 1 层

答案比较、题库解析、成绩汇总是三个函数，不要把终端输入写进它们。

## 第 2 层

先 trim 再 eq_ignore_ascii_case；题库 split TAB 后检查字段数；卡片字段要拥有 String。

## 第 3 层

计分可以用普通 for 循环逐对比较；错题保存 card.question.clone()，不用 clone 整个题库。
