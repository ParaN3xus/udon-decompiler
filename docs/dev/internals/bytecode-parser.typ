#import "/docs/book.typ": book-page
#import "@preview/shiroa:0.3.1": shiroa-sys-target
#import "../../_utils/common.typ": cross-link-heading

#show: book-page.with(title: "反汇编")

通过 `BytecodeParser` 实现. 这部分代码比较直白.

#cross-link-heading("/dev/udon/udon-vm.typ", [== 执行过程])[Udon VM - 执行过程] 一节中提到栈中的值都是堆地址, 且除了 `JUMP` 和 `JUMP_IF_FALSE` 之外的所有指令的 `OPERAND` 值都是堆地址, 所以在 `BytecodeParser._parse_instruction_at` 中, 我们可以放心地到符号表中查找指令的 `operand_name`.
