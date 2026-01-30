#import "/docs/book.typ": book-page
#import "@preview/shiroa:0.3.1": shiroa-sys-target
#import "../../_utils/common.typ": cross-link-heading, numba-scfg

#show: book-page.with(title: "代码生成")

主要包括 AST 构建和伪代码生成.

= 构建 AST

构建 AST 的主要工作是控制流识别, 添加变量定义, 把表达式转为语句.

== 控制流识别

本编译器采取的策略是先使用 #numba-scfg 进行初次控制流识别, 构建 Structured Control Flow Graph 并进行结构重建, 然后使用多次匹配固定模式的 lift 过程, 将基于 SCFG 生成的 AST 转为更可读的形式.

=== SCFG 构建

我们使用 `SCFGAdapter` 将函数 CFG 转为 `SCFG`, 然后执行 `scfg.restructure()`.

=== AST emit 和 SCFG lift

`restructure` 后的 `SCFG` 首先使用 `SCFGRawEmitter` 进行原意翻译, 产生原始的 AST. 接下来, 该 AST 将通过一系列 lift 过程被优化为更可读的形式. 按执行顺序, 负责执行这些 lift 过程的 `Lifter` 包括
- `LoopLifter`
- `BackedgeLifter`
- `ExitVarLifter`
- `ControlVarLifter`
- `CleanupLifter`
- `WhileTrueLifter`
- `CleanupLifter` (第二次)

下面用一些例子阐述各个 `Lifter` 的主要工作

==== `LoopLifter` 和 `BackedgeLifter`

目的是把 `loop_cont/backedge` 变量驱动的低级循环, 恢复为 `while(cond)` 或 `do { ... } while(cond)` 的结构.


`LoopLifter` 负责把 `loop_cont` 的尾部检查. `BackedgeLifter` 则消除 `backedge` 变量赋值, 只保留必要的 `break`.


Lift 前:
```cs
__scfg_loop_cont_1__ = true;
while (__scfg_loop_cont_1__)
{
    if (i < 5)
    {
        sum += i;
        i++;
        __scfg_backedge_var_0__ = 0;
    }
    else
    {
        __scfg_backedge_var_0__ = 1;
    }
    __scfg_loop_cont_1__ = __scfg_backedge_var_0__ == 0;
}
```

Lift 后:
```cs
while (i < 5)
{
    sum += i;
    i++;
}
```

==== `ExitVarLifter`

目的是消除 loop 之后的 `exit_var` 判断, 把分支逻辑内联回 loop 内部.

Lift 前:
```cs
while (true)
{
    if (i < 4)
    {
        if (i == 2)
        {
            hit = true;
            __scfg_exit_var_0__ = 0;
            break;
        }
        i++;
        __scfg_exit_var_0__ = -1;
    }
    else
    {
        __scfg_exit_var_0__ = 1;
        break;
    }
}
if (__scfg_exit_var_0__ == 0)
{
    Debug.Log("hit");
}
```

Lift 后:
```cs
while (true)
{
    if (i < 4)
    {
        if (i == 2)
        {
            hit = true;
            Debug.Log("hit");
            break;
        }
        i++;
    }
    else
    {
        break;
    }
}
```

可以看出循环外的 `if`/`switch` 被内联到 loop 内 `break` 位置, 从而消除了 `exit_var`.

==== `ControlVarLifter`

目的是合并控制变量驱动的 `switch` + 默认分支判断的模式, 恢复到自然的 `switch`.

Lift 前:
```cs
switch (mode)
{
    case 0: __scfg_control_var_0__ = 2; break;
    case 1: name = "A"; __scfg_control_var_0__ = 3; break;
    case 2: name = "B"; __scfg_control_var_0__ = 4; break;
}
if (__scfg_control_var_0__ == 0 | __scfg_control_var_0__ == 1 | __scfg_control_var_0__ == 2)
{
    name = "Unknown";
}
```

Lift 后:
```cs
switch (mode)
{
    case 1: name = "A"; break;
    case 2: name = "B"; break;
    default: name = "Unknown"; break;
}
```

可以看出 Lift 后, 多余的控制变量被删除, 只负责默认分支的 case 被合并为 `default`.

==== `CleanupLifter`

目的是清理 lift 后的残留松散结构, 常见工作如
- 删除空语句/空 `if`
- 反转 `if` 以消除空 `then`
- 移除未使用的 `__scfg_*` 变量赋值与声明

示意:
```cs
// before
if (cond) { } else { body; }
__scfg_tmp_0__ = 1; // never been read

// after
if (!cond) { body; }
```

==== `WhileTrueLifter`

目的处理残余的 `while(true)` 模式, 尽量恢复 `while(cond)` 或 `do/while`.

Lift 前:
```cs
while (true)
{
    tmp = i < limit;
    if (tmp)
    {
        i++;
    }
    else
    {
        break;
    }
}
```

Lift 后:
```cs
do
{
    i++;
    tmp = i < limit;
} while (tmp);
```

== 添加变量定义和把表达式转为语句

这部分代码比较直白, 在此不再赘述.

= 添加全局变量定义

全局变量由 `ProgramCodeGenerator` 汇总, 其主要工作包括:
- 收集各函数中被引用的变量名
- 仅导出被用到的 `global` 变量
- 过滤返回跳转、内部临时符号等不应暴露的变量

= 生成伪代码

这部分代码比较直白, 在此不再赘述.

外代码生成后额外使用了 `clang-format` 进行格式化.
