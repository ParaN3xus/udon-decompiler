use std::collections::{BTreeMap, HashSet};

use tracing::{debug, info};

use crate::decompiler::basic_block::{BasicBlock, BasicBlockType};
use crate::decompiler::cfg::StackFrame;
use crate::decompiler::context::DecompileContext;
use crate::decompiler::module_info::{ExternFunctionInfo, FunctionDefinitionType, UdonModuleInfo};
use crate::str_constants::SYMBOL_RETURN_JUMP_U32;
use crate::udon_asm::{OpCode, OperandToken};

use super::nodes::{
    IrAssignmentStatement, IrBlock, IrBlockContainer, IrConstructorCallExpression, IrContainerKind,
    IrExpression, IrExpressionStatement, IrExternalCallExpression, IrFunction, IrIf,
    IrInternalCallExpression, IrJump, IrLeave, IrOperator, IrOperatorCallExpression,
    IrPropertyAccessExpression, IrRawExpression, IrReturn, IrStatement, IrSwitch,
    IrVariableExpression,
};
use crate::decompiler::FunctionCfg;

enum InternalCallKind {
    Returning {
        function_name: String,
        entry_address: u32,
        call_jump_target: u32,
    },
    Tail {
        function_name: String,
        entry_address: u32,
        call_jump_target: u32,
    },
}

pub fn build_extern_ir_expression(
    function_info: ExternFunctionInfo,
    signature: String,
    arguments: Vec<IrExpression>,
) -> IrExpression {
    match function_info.def_type {
        FunctionDefinitionType::Field => IrExpression::PropertyAccess(IrPropertyAccessExpression {
            function_info,
            signature,
            arguments,
        }),
        FunctionDefinitionType::Ctor => {
            IrExpression::ConstructorCall(IrConstructorCallExpression {
                function_info,
                signature,
                arguments,
            })
        }
        FunctionDefinitionType::Operator => {
            let operator = IrOperator::from_extern_signature(signature.as_str())
                .unwrap_or_else(|| panic!("unsupported operator extern signature: {signature}"));
            let arguments = if operator == IrOperator::ExplicitConversion {
                let mut arguments = arguments;
                arguments.insert(
                    0,
                    IrExpression::Raw(IrRawExpression {
                        value: function_info.type_name.clone(),
                    }),
                );
                arguments
            } else {
                arguments
            };
            IrExpression::OperatorCall(IrOperatorCallExpression {
                arguments,
                operator,
            })
        }
        FunctionDefinitionType::Method => IrExpression::ExternalCall(IrExternalCallExpression {
            function_info,
            signature,
            arguments,
        }),
    }
}

pub struct IrBuilder<'a> {
    ctx: &'a DecompileContext,
    function_cfg: &'a FunctionCfg,
    module_info: &'static UdonModuleInfo,
    body_container_id: u32,
}

impl<'a> IrBuilder<'a> {
    const ROOT_CONTAINER_ID: u32 = 0;

    pub fn new(ctx: &'a DecompileContext, function_cfg: &'a FunctionCfg) -> Self {
        Self {
            ctx,
            function_cfg,
            module_info: UdonModuleInfo::load_default_cached()
                .expect("IrBuilder requires loaded UdonModuleInfo"),
            body_container_id: Self::ROOT_CONTAINER_ID,
        }
    }

    pub fn build(self) -> IrFunction {
        let mut blocks = Vec::<IrBlock>::new();

        // create all `IrBlock`s
        for block_id in &self.function_cfg.block_ids {
            let raw = &self.ctx.basic_blocks.blocks[*block_id];
            let start_address = raw.start_address();
            blocks.push(IrBlock {
                statements: Vec::new(),
                start_address,
                should_emit_label: false,
            });
        }

        // build statements
        for (index, block_id) in self.function_cfg.block_ids.iter().enumerate() {
            let raw = &self.ctx.basic_blocks.blocks[*block_id];
            blocks[index].statements = self.build_statements(raw);
        }

        IrFunction {
            function_name: self.function_cfg.function_name.clone(),
            is_function_public: self.function_cfg.is_function_public,
            entry_address: self.function_cfg.entry_address,
            // will be built in later passes
            variable_declarations: Vec::new(),
            body: IrBlockContainer {
                id: self.body_container_id,
                blocks,
                kind: IrContainerKind::Block,
                should_emit_exit_label: false,
            },
        }
    }

    fn build_statements(&self, block: &BasicBlock) -> Vec<IrStatement> {
        let mut out = Vec::<IrStatement>::new();
        for (_block_instruction_id, address, instruction) in block.instructions.iter() {
            //
            if self.is_switch_scaffold_instruction(block, address) {
                continue;
            }
            let mut statements =
                self.build_statements_from_instruction(block, address, instruction);
            out.append(&mut statements);
        }
        self.append_implicit_fallthrough(block, &mut out);
        out
    }

    fn build_statements_from_instruction(
        &self,
        block: &BasicBlock,
        address: u32,
        instruction: &crate::udon_asm::AsmInstruction,
    ) -> Vec<IrStatement> {
        match instruction.opcode {
            OpCode::Copy => self.build_copy_statements(address),
            OpCode::Extern => self.build_extern_statements(address, instruction),
            OpCode::Jump => self.build_jump_statements(address, instruction),
            OpCode::JumpIfFalse => self.build_jump_if_false_statements(block, address, instruction),
            OpCode::JumpIndirect => self.build_jump_indirect_statements(block),
            OpCode::Nop | OpCode::Annotation | OpCode::Pop | OpCode::Push => Vec::new(),
        }
    }

    fn build_copy_statements(&self, address: u32) -> Vec<IrStatement> {
        let state = self.require_instruction_state(address);
        let target_value = state
            .peek(0)
            .unwrap_or_else(|| panic!("COPY at 0x{address:08X} has incomplete stack operands"));
        let source_value = state
            .peek(1)
            .unwrap_or_else(|| panic!("COPY at 0x{address:08X} has incomplete stack operands"));

        let target_address = target_value.value;
        if self
            .ctx
            .variables
            .get_by_address(target_address)
            .is_some_and(|variable| variable.name == SYMBOL_RETURN_JUMP_U32)
        {
            return Vec::new();
        }

        vec![IrStatement::Assignment(IrAssignmentStatement {
            target: IrExpression::from_heap_addr(&self.ctx.variables, target_address),
            value: IrExpression::from_heap_addr(&self.ctx.variables, source_value.value),
        })]
    }

    fn build_extern_statements(
        &self,
        address: u32,
        instruction: &crate::udon_asm::AsmInstruction,
    ) -> Vec<IrStatement> {
        let signature_addr = match instruction.operand.as_ref() {
            Some(OperandToken::Number(signature_addr)) => *signature_addr,
            _ => panic!("EXTERN at 0x{address:08X} missing numeric operand"),
        };

        let signature = self
            .ctx
            .heap_string_literals
            .get(&signature_addr)
            .cloned()
            .unwrap_or_else(|| {
                panic!("EXTERN at 0x{address:08X} missing heap string signature for 0x{signature_addr:08X}")
            });
        let function_info = self
            .module_info
            .get_function_info(&signature)
            .unwrap_or_else(|| panic!("Unknown extern signature at 0x{address:08X}: {signature}"));

        let args = self.build_call_arguments(address, function_info.parameter_count());

        if is_property_setter(&function_info) {
            let (value, target_args) = args.split_last().unwrap_or_else(|| {
                panic!("Property setter at 0x{address:08X} requires at least one argument")
            });
            return vec![IrStatement::Assignment(IrAssignmentStatement {
                target: self.build_extern_expression(
                    function_info,
                    signature,
                    target_args.to_vec(),
                ),
                value: value.clone(),
            })];
        }

        if function_info.returns_void {
            let call_expr = self.build_extern_expression(function_info, signature, args);
            return vec![IrStatement::Expression(IrExpressionStatement {
                expression: call_expr,
            })];
        }

        let return_slot = args.last().cloned();
        let call_expr =
            self.build_extern_expression(function_info, signature, args[..args.len() - 1].to_vec());

        if let Some(IrExpression::Variable(IrVariableExpression { address })) = return_slot {
            return vec![IrStatement::Assignment(IrAssignmentStatement {
                target: IrExpression::Variable(IrVariableExpression { address }),
                value: call_expr,
            })];
        }

        vec![IrStatement::Expression(IrExpressionStatement {
            expression: call_expr,
        })]
    }

    fn build_extern_expression(
        &self,
        function_info: ExternFunctionInfo,
        signature: String,
        arguments: Vec<IrExpression>,
    ) -> IrExpression {
        build_extern_ir_expression(function_info, signature, arguments)
    }

    fn build_call_arguments(&self, address: u32, parameter_count: usize) -> Vec<IrExpression> {
        let state = self.require_instruction_state(address);

        let mut args = Vec::<IrExpression>::new();
        if state.depth() < parameter_count {
            panic!(
                "EXTERN at 0x{address:08X} requires {parameter_count} stack args, got {}",
                state.depth()
            );
        }

        for index in 0..parameter_count {
            let depth = parameter_count - 1 - index;
            let value = state
                .peek(depth)
                .unwrap_or_else(|| panic!("EXTERN at 0x{address:08X} missing stack arg {index}"));
            args.push(IrExpression::from_heap_addr(
                &self.ctx.variables,
                value.value,
            ));
        }
        args
    }

    fn build_jump_statements(
        &self,
        address: u32,
        instruction: &crate::udon_asm::AsmInstruction,
    ) -> Vec<IrStatement> {
        if let Some(call_kind) = self.resolve_internal_call_entry(address, instruction) {
            let (function_name, entry_address, call_jump_target, should_return) = match call_kind {
                InternalCallKind::Returning {
                    function_name,
                    entry_address,
                    call_jump_target,
                } => (function_name, entry_address, call_jump_target, false),
                InternalCallKind::Tail {
                    function_name,
                    entry_address,
                    call_jump_target,
                } => (function_name, entry_address, call_jump_target, true),
            };

            let mut statements = vec![IrStatement::Expression(IrExpressionStatement {
                expression: IrExpression::InternalCall(IrInternalCallExpression {
                    function_name: Some(function_name),
                    entry_address,
                    call_jump_target,
                }),
            })];
            if should_return {
                statements.push(IrStatement::Return(IrReturn));
            }
            return statements;
        }

        let target_addr = instruction.numeric_operand();

        vec![self.build_direct_jump_target_statement(target_addr)]
    }

    fn build_jump_if_false_statements(
        &self,
        block: &BasicBlock,
        address: u32,
        instruction: &crate::udon_asm::AsmInstruction,
    ) -> Vec<IrStatement> {
        let false_addr = instruction.numeric_operand();

        let condition = self.build_condition_expression(address);
        let false_branch = self.build_direct_jump_target_statement(false_addr);
        let true_branch = if block.block_type == BasicBlockType::Return {
            Some(self.build_function_exit_statement())
        } else {
            Some(self.build_next_instruction_jump_statement(address))
        };

        vec![IrStatement::If(IrIf {
            condition: IrExpression::OperatorCall(IrOperatorCallExpression {
                arguments: vec![condition],
                operator: IrOperator::UnaryNegation,
            }),
            true_statement: Box::new(false_branch),
            false_statement: true_branch.map(Box::new),
        })]
    }

    fn build_next_instruction_jump_statement(&self, address: u32) -> IrStatement {
        let instruction_id = self
            .ctx
            .instructions
            .id_at_address(address)
            .unwrap_or_else(|| panic!("missing instruction id for address 0x{address:08X}"));
        let next_address = self
            .ctx
            .instructions
            .next_of(instruction_id)
            .and_then(|next| self.ctx.instructions.address_of(next))
            .unwrap_or_else(|| {
                panic!("missing next instruction after JUMP_IF_FALSE at 0x{address:08X}")
            });
        self.build_direct_jump_target_statement(next_address)
    }

    fn build_jump_indirect_statements(&self, block: &BasicBlock) -> Vec<IrStatement> {
        if block.block_type == BasicBlockType::Return {
            return vec![self.build_function_exit_statement()];
        }

        let Some(switch_info) = block.switch_info.as_ref() else {
            return vec![self.build_function_exit_statement()];
        };

        let mut cases = BTreeMap::<u32, u32>::new();
        for (case_value, target_address) in switch_info.targets.iter().enumerate() {
            cases.insert(case_value as u32, *target_address);
        }

        let excluded = switch_info
            .targets
            .iter()
            .copied()
            .collect::<HashSet<u32>>();
        let default_target = self.resolve_fallthrough_target(block, &excluded);

        vec![IrStatement::Switch(IrSwitch {
            index_expression: IrExpression::from_heap_addr(
                &self.ctx.variables,
                switch_info.index_operand,
            ),
            cases,
            default_target,
        })]
    }

    fn append_implicit_fallthrough(&self, block: &BasicBlock, statements: &mut Vec<IrStatement>) {
        if statements
            .last()
            .is_some_and(|statement| statement.is_terminator())
        {
            return;
        }

        if block.block_type == BasicBlockType::Return {
            statements.push(self.build_function_exit_statement());
            return;
        }

        if let Some(target) = self.resolve_fallthrough_target(block, &HashSet::new()) {
            statements.push(IrStatement::Jump(IrJump {
                target_address: target,
            }));
        }
    }

    fn resolve_fallthrough_target(
        &self,
        block: &BasicBlock,
        excluded_addresses: &HashSet<u32>,
    ) -> Option<u32> {
        let mut candidates = block
            .successors
            .iter()
            .filter_map(|x| self.ctx.basic_blocks.blocks.get(*x))
            .map(|x| x.start_address())
            .filter(|x| !excluded_addresses.contains(x))
            .collect::<Vec<_>>();

        if candidates.len() == 1 {
            return candidates.pop();
        }
        if candidates.len() > 1 {
            if let Some(next_address) =
                self.resolve_fallthrough_by_next_address(block, excluded_addresses)
            {
                return Some(next_address);
            }
            candidates.sort_unstable();
            return candidates.first().copied();
        }

        self.resolve_fallthrough_by_next_address(block, excluded_addresses)
    }

    fn resolve_fallthrough_by_next_address(
        &self,
        block: &BasicBlock,
        excluded_addresses: &HashSet<u32>,
    ) -> Option<u32> {
        let last_inst_id = block.instructions.last_id()?;
        let last_address = block.instructions.address_of(last_inst_id)?;
        let instruction_id = self.ctx.instructions.id_at_address(last_address)?;
        let next_address = self
            .ctx
            .instructions
            .next_of(instruction_id)
            .and_then(|next| self.ctx.instructions.address_of(next))?;

        if self.function_contains_block_start(next_address)
            && !excluded_addresses.contains(&next_address)
        {
            return Some(next_address);
        }
        None
    }

    fn build_direct_jump_target_statement(&self, target_addr: u32) -> IrStatement {
        if self.ctx.is_out_of_program_counter_range(target_addr) {
            return self.build_function_exit_statement();
        }
        IrStatement::Jump(IrJump {
            target_address: target_addr,
        })
    }

    fn build_function_exit_statement(&self) -> IrStatement {
        IrStatement::Leave(IrLeave {
            target_container_id: self.body_container_id,
        })
    }

    fn build_condition_expression(&self, address: u32) -> IrExpression {
        let state = self.require_instruction_state(address);
        let value = state
            .peek(0)
            .unwrap_or_else(|| panic!("JUMP_IF_FALSE at 0x{address:08X} missing condition value"));
        IrExpression::from_heap_addr(&self.ctx.variables, value.value)
    }

    fn resolve_internal_call_entry(
        &self,
        address: u32,
        instruction: &crate::udon_asm::AsmInstruction,
    ) -> Option<InternalCallKind> {
        if instruction.opcode != OpCode::Jump {
            return None;
        }

        let target = instruction.numeric_operand();
        let entry = self.ctx.entry_points.iter().find(|entry| {
            let call_target = entry.entry_call_jump_target(self.ctx);
            entry.address == target || call_target == target
        })?;

        let state = self.require_instruction_state(address);

        // there might be a call jump at the end of the whole program,
        // that's why we don't simply add 8 to `address`
        let next_address = self
            .ctx
            .instructions
            .id_at_address(address)
            .and_then(|instruction_id| self.ctx.instructions.next_of(instruction_id))
            .and_then(|next| self.ctx.instructions.address_of(next));
        if let Some(top) = state.peek(0)
            && let Some(next_address) = next_address
            && self.ctx.heap_u32_literals.get(&top.value).copied() == Some(next_address)
        {
            return Some(InternalCallKind::Returning {
                function_name: entry.name.clone(),
                entry_address: entry.address,
                call_jump_target: target,
            });
        }

        Some(InternalCallKind::Tail {
            function_name: entry.name.clone(),
            entry_address: entry.address,
            call_jump_target: target,
        })
    }

    fn require_instruction_state(&self, address: u32) -> &StackFrame {
        self.ctx
            .stack_simulation
            .get_instruction_state(address)
            .unwrap_or_else(|| panic!("Missing stack state at instruction 0x{address:08X}"))
    }

    fn is_switch_scaffold_instruction(&self, block: &BasicBlock, instruction_address: u32) -> bool {
        // 3 PUSH + 1 EXTERN
        block.switch_info.as_ref().is_some_and(|x| {
            x.scaffold_instruction_addresses
                .contains(&instruction_address)
        })
    }

    fn function_contains_block_start(&self, address: u32) -> bool {
        let Some(block_id) = self.ctx.basic_block_id_by_start(address) else {
            return false;
        };
        self.function_cfg.block_ids.contains(&block_id)
    }
}

pub(crate) fn is_property_setter(function_info: &ExternFunctionInfo) -> bool {
    function_info.def_type == FunctionDefinitionType::Field
        && function_info.function_name.starts_with("__set_")
}

pub fn build_ir_functions(ctx: &DecompileContext) -> Vec<IrFunction> {
    debug!("building IrFunctions...");

    let mut functions = Vec::<IrFunction>::new();
    for function_cfg in &ctx.cfg_functions {
        functions.push(IrBuilder::new(ctx, function_cfg).build());
    }

    info!("{} IrFunctions built", functions.len());
    functions
}
