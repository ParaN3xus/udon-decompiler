from __future__ import annotations

from typing import TYPE_CHECKING, List, Optional, Tuple

from numba_scfg.core.datastructures.basic_block import (
    BasicBlock as SCFGBasicBlock,
)
from numba_scfg.core.datastructures.basic_block import (
    RegionBlock as SCFGRegionBlock,
)
from numba_scfg.core.datastructures.basic_block import (
    SyntheticAssignment,
    SyntheticBranch,
    SyntheticExitBranch,
    SyntheticExitingLatch,
    SyntheticFill,
    SyntheticHead,
    SyntheticReturn,
    SyntheticTail,
)
from numba_scfg.core.datastructures.scfg import SCFG

from udon_decompiler.analysis.basic_block import BasicBlock, BasicBlockType
from udon_decompiler.analysis.expression_builder import (
    Operator,
)
from udon_decompiler.codegen.ast_nodes import (
    AssignmentNode,
    BlockNode,
    ExpressionNode,
    IfElseNode,
    LiteralNode,
    OperatorNode,
    ReturnNode,
    StatementNode,
    SwitchCaseNode,
    SwitchNode,
    VariableDeclNode,
    VariableNode,
    WhileNode,
)

if TYPE_CHECKING:
    from udon_decompiler.codegen.ast_builder import ASTBuilder


class _SCFGEmitter:
    def __init__(
        self,
        builder: "ASTBuilder",
        scfg: SCFG,
        name_to_block: dict[str, BasicBlock],
        switch_branches: dict[str, BasicBlock],
    ):
        self.builder = builder
        self.scfg = scfg
        self.name_to_block = name_to_block
        self.block_to_name = {v: k for k, v in name_to_block.items()}
        self.switch_branches = switch_branches
        self.synthetic_vars: dict[str, str] = {}
        self.region_stack: list[SCFGRegionBlock] = [self.scfg.region]
        self.loop_cont_counter = 0
        self.emitted: set[str] = set()
        self.all_blocks: dict[str, SCFGBasicBlock] = self._collect_all_blocks()
        self.branch_assignments = self._collect_branch_assignments()

    def synthetic_declarations(self) -> List[VariableDeclNode]:
        decls: List[VariableDeclNode] = []
        for name, var_type in sorted(self.synthetic_vars.items()):
            decls.append(VariableDeclNode(var_name=name, var_type=var_type))
        return decls

    def emit(self) -> List[StatementNode]:
        statements: List[StatementNode] = []
        for _, block in self.scfg.concealed_region_view.items():
            if isinstance(block, SCFGRegionBlock) and block.kind == "branch":
                continue
            key = self._block_key(block)
            if key is not None and key in self.emitted:
                continue
            statements.extend(self.codegen(block))
        return statements

    def _declare_var(self, name: str, var_type: str) -> None:
        if name not in self.synthetic_vars:
            self.synthetic_vars[name] = var_type

    def lookup(self, item: str) -> SCFGBasicBlock:
        """
        Find the block with given name

        :param item: name
        """
        subregion = self.region_stack[-1].subregion
        if subregion and item in subregion:
            return subregion[item]
        parent = self.region_stack[-1].parent_region
        if parent is not None:
            try:
                return self._rlookup(parent, item)
            except KeyError:
                pass
        if item in self.all_blocks:
            return self.all_blocks[item]
        raise KeyError(f"Item {item} not found in subregion or parent")

    def _rlookup(self, region: SCFGRegionBlock, item: str) -> SCFGBasicBlock:
        if region.subregion and item in region.subregion:
            return region.subregion[item]
        if region.parent_region is None:
            raise KeyError(f"Item {item} not found in subregion or parent")
        return self._rlookup(region.parent_region, item)

    def _collect_all_blocks(self) -> dict[str, SCFGBasicBlock]:
        all_blocks: dict[str, SCFGBasicBlock] = {}

        def walk_region(region: SCFGRegionBlock) -> None:
            if region.subregion is None:
                return
            subregion: SCFG = region.subregion
            for name, block in subregion.concealed_region_view.items():
                all_blocks.setdefault(name, block)
                if isinstance(block, SCFGRegionBlock):
                    walk_region(block)

        walk_region(self.scfg.region)
        return all_blocks

    def _block_key(self, block: SCFGBasicBlock) -> Optional[str]:
        if isinstance(block, SCFGRegionBlock):
            return block.name
        if isinstance(block, SCFGBasicBlock):
            return block.name
        if isinstance(block, (SyntheticAssignment, SyntheticBranch, SyntheticHead)):
            return block.name
        return None

    def codegen(self, block: SCFGBasicBlock) -> List[StatementNode]:
        key = self._block_key(block)
        if key is not None:
            if key in self.emitted:
                return []
            self.emitted.add(key)
        if isinstance(block, SCFGRegionBlock):
            return self._codegen_region(block)
        if isinstance(block, SyntheticAssignment):
            return self._codegen_synth_assignment(block)
        if isinstance(block, SyntheticTail):
            return []
        if isinstance(block, SyntheticFill):
            return []
        if isinstance(block, SyntheticReturn):
            return [ReturnNode()]
        if isinstance(block, SyntheticExitingLatch):
            return self._codegen_synth_exiting_latch(block)
        if isinstance(block, (SyntheticExitBranch, SyntheticHead, SyntheticBranch)):
            if block.name in self.switch_branches:
                origin = self.switch_branches[block.name]
                return self._codegen_switch_block(origin, block.jump_targets)
            if self._is_branch_assignment_control(block.variable):
                return []
            return self._codegen_synth_branch(block)
        if isinstance(block, SCFGBasicBlock):
            origin = self.name_to_block.get(block.name)
            if origin is None:
                return []
            if (
                origin.block_type == BasicBlockType.CONDITIONAL
                and len(block.jump_targets) == 2
            ):
                return self._codegen_conditional_block(origin, block.jump_targets)
            if origin.switch_info is not None and len(block.jump_targets) > 1:
                return self._codegen_switch_block(origin, block.jump_targets)
            node_block = BlockNode()
            self.builder._translate_basic_block(origin, node_block)
            return list(node_block.statements)
        return []

    def _codegen_region(self, region: SCFGRegionBlock) -> List[StatementNode]:
        self.region_stack.append(region)

        def codegen_view() -> List[StatementNode]:
            out: List[StatementNode] = []
            assert region.subregion is not None
            assert isinstance(region.subregion, SCFG)
            for _, block in region.subregion.concealed_region_view.items():
                if isinstance(block, SCFGRegionBlock) and block.kind == "branch":
                    continue
                out.extend(self.codegen(block))
            return out

        if region.kind in ("head", "tail", "branch"):
            result = codegen_view()
        elif region.kind == "loop":
            self.loop_cont_counter += 1
            loop_continue = f"__scfg_loop_cont_{self.loop_cont_counter}__"
            self._declare_var(loop_continue, "System.Boolean")
            init = AssignmentNode(
                target=loop_continue,
                value=LiteralNode(literal_type="System.Boolean", value=True),
            )
            body_block = BlockNode()
            for stmt in codegen_view():
                body_block.add_statement(stmt)
            result = [
                init,
                WhileNode(
                    condition=VariableNode(var_name=loop_continue),
                    body=body_block,
                ),
            ]
        else:
            result = []

        self.region_stack.pop()
        return result

    def _codegen_synth_assignment(
        self, block: SyntheticAssignment
    ) -> List[StatementNode]:
        statements: List[StatementNode] = []
        if (
            len(block.variable_assignment) == 1
            and (item := next(iter(block.variable_assignment.items()), None))
            is not None
        ):
            # just ignore, only emit it's target
            name, value = item
            target = self._resolve_branch_assignment_target(name, value)
            if target is not None:
                for stmt in self.codegen(self.lookup(target)):
                    statements.append(stmt)
                return statements
        for name, value in block.variable_assignment.items():
            if self._is_branch_assignment_control(name):
                continue
            self._declare_var(name, "System.Int32")
            statements.append(
                AssignmentNode(
                    target=name,
                    value=LiteralNode(literal_type="System.Int32", value=value),
                )
            )
        return statements

    def _collect_branch_assignments(self) -> dict[str, SyntheticBranch]:
        mapping: dict[str, SyntheticBranch] = {}

        def walk(region: SCFGRegionBlock) -> None:
            if region.subregion is None:
                return
            subregion: SCFG = region.subregion
            for _, block in subregion.concealed_region_view.items():
                if isinstance(block, (SyntheticBranch, SyntheticHead)):
                    if self._is_branch_assignment_control(block.variable):
                        mapping[block.variable] = block
                if isinstance(block, SCFGRegionBlock):
                    walk(block)

        walk(self.scfg.region)
        return mapping

    def _is_branch_assignment_control(self, name: str) -> bool:
        # these variables are only used to signal which block to jump to
        # so they can be ignored
        return name.startswith("__scfg_control_var_")

    def _resolve_branch_assignment_target(self, name: str, value: int) -> Optional[str]:
        branch = self.branch_assignments.get(name)
        if branch is None:
            return None
        return branch.branch_value_table.get(value)

    def _codegen_synth_exiting_latch(
        self, block: SyntheticExitingLatch
    ) -> List[StatementNode]:
        loop_continue = f"__scfg_loop_cont_{self.loop_cont_counter}__"
        self._declare_var(loop_continue, "System.Boolean")
        self._declare_var(block.variable, "System.Int32")
        cond = OperatorNode(
            operator=Operator.Equality,
            receiver=None,
            operands=[
                VariableNode(var_name=block.variable),
                LiteralNode(literal_type="System.Int32", value=0),
            ],
        )
        self.loop_cont_counter -= 1
        return [
            AssignmentNode(
                target=loop_continue,
                value=cond,
            )
        ]

    def _codegen_synth_branch(self, block: SyntheticBranch) -> List[StatementNode]:
        self._declare_var(block.variable, "System.Int32")
        reverse: dict[str, List[int]] = {}
        for value, target in block.branch_value_table.items():
            reverse.setdefault(target, []).append(value)

        cases: List[SwitchCaseNode] = []
        for target, values in sorted(reverse.items(), key=lambda item: item[0]):
            case_body = BlockNode()
            for stmt in self.codegen(self.lookup(target)):
                case_body.add_statement(stmt)
            case_values: List[ExpressionNode] = [
                LiteralNode(literal_type="System.Int32", value=v) for v in values
            ]
            cases.append(SwitchCaseNode(values=case_values, body=case_body))

        switch_stmt = SwitchNode(
            expression=VariableNode(var_name=block.variable),
            cases=cases,
            default_case=None,
        )
        return [switch_stmt]

    def _codegen_conditional_block(
        self, origin: BasicBlock, jump_targets: Tuple[str, str]
    ) -> List[StatementNode]:
        statements: List[StatementNode] = []
        for stmt in self.builder._translate_block_for_condition(origin):
            statements.append(stmt)
        condition = self.builder._extract_condition_from_block(origin)
        false_target, true_target = jump_targets[0], jump_targets[1]

        then_block = BlockNode()
        for stmt in self.codegen(self.lookup(true_target)):
            then_block.add_statement(stmt)
        else_block = BlockNode()
        for stmt in self.codegen(self.lookup(false_target)):
            else_block.add_statement(stmt)

        statements.append(
            IfElseNode(
                condition=condition,
                then_block=then_block,
                else_block=else_block,
                address=origin.start_address,
            )
        )
        return statements

    def _codegen_switch_block(
        self, origin: BasicBlock, jump_targets: Tuple[str, ...]
    ) -> List[StatementNode]:
        switch_info = origin.switch_info
        if switch_info is None:
            return []
        var = self.builder.analyzer.variable_identifier.get_variable(
            switch_info.index_operand
        )
        if not var:
            return []
        switch_expr = self.builder._variable_to_ast(var)

        target_counts: dict[int, int] = {}
        for t in switch_info.targets:
            target_counts[t] = target_counts.get(t, 0) + 1
        default_target = max(target_counts, key=lambda k: target_counts[k])

        cases: List[SwitchCaseNode] = []
        default_case = None
        unique_targets = list(dict.fromkeys(switch_info.targets))
        for target_addr in unique_targets:
            target_block = self.builder.cfg.get_block_at(target_addr)
            if target_block is None:
                continue
            target_name = self.block_to_name.get(target_block)
            case_body = BlockNode()
            if target_name is None:
                self.builder._translate_basic_block(target_block, case_body)
            else:
                try:
                    for stmt in self.codegen(self.lookup(target_name)):
                        case_body.add_statement(stmt)
                except KeyError:
                    self.builder._translate_basic_block(target_block, case_body)
            case_values: List[ExpressionNode] = [
                LiteralNode(literal_type="System.Int32", value=i)
                for i, addr in enumerate(switch_info.targets)
                if addr == target_addr and addr != default_target
            ]
            if target_addr == default_target:
                default_case = SwitchCaseNode(
                    values=[], body=case_body, is_default=True
                )
            else:
                cases.append(SwitchCaseNode(values=case_values, body=case_body))

        switch_stmt = SwitchNode(
            expression=switch_expr,
            cases=cases,
            default_case=default_case,
            address=origin.start_address,
        )
        return [switch_stmt]
