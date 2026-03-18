use std::collections::{HashMap, HashSet, VecDeque};

use crate::str_constants::{
    SYMBOL_CONST_SYSTEM_UINT32_0, SYMBOL_RETURN_JUMP_U32, TYPE_SYSTEM_VOID,
};
use crate::udon_asm::OpCode;
use tracing::{debug, info, warn};

use super::basic_block::{BasicBlockCollection, BasicBlockType};
use super::context::{DecompileContext, DecompileSymbol};
use super::indir_jump_analysis::{is_return_jump_operand, resolve_switch_info_for_jump_indirect};
use super::module_info::UdonModuleInfo;
use super::{DecompileError, Result};

type HeapLiteralState = HashMap<u32, HeapValue>;
const HIDDEN_ENTRY_PLACEHOLDER_PREFIX: &str = "__hidden_entry_";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionCfg {
    pub function_name: String,
    pub is_function_public: bool,
    pub entry_address: u32,
    pub entry_block: usize,
    pub block_ids: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StackValue {
    HeapAddress(u32),
    HaltJump,
}

impl StackValue {
    pub fn heap_address(&self) -> u32 {
        match self {
            Self::HeapAddress(address) => *address,
            _ => panic!("trying to get the heap_address of a phantom stack value"),
        }
    }

    pub fn resolve_u32_literal(&self, ctx: &DecompileContext) -> Option<u32> {
        match self {
            Self::HeapAddress(address) => ctx.heap_u32_literals.get(address).copied(),
            Self::HaltJump => Some(u32::MAX),
        }
    }

    pub fn as_u32_literal(&self, heap_state: &HeapLiteralState) -> Option<u32> {
        match self {
            Self::HeapAddress(address) => {
                heap_state.get(address).and_then(HeapValue::as_u32_literal)
            }
            Self::HaltJump => Some(u32::MAX),
        }
    }

    pub fn as_heap_value(&self, heap_state: &HeapLiteralState) -> HeapValue {
        match self {
            Self::HeapAddress(address) => heap_state
                .get(address)
                .cloned()
                .unwrap_or(HeapValue::Unknown),
            Self::HaltJump => HeapValue::HaltJump,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeapValue {
    U32(u32),
    HaltJump,
    Unknown,
}

impl HeapValue {
    pub fn as_u32_literal(&self) -> Option<u32> {
        match self {
            Self::U32(value) => Some(*value),
            Self::HaltJump => Some(u32::MAX),
            Self::Unknown => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StackFrame {
    stack: Vec<StackValue>,
}

impl StackFrame {
    pub fn push(&mut self, value: StackValue) {
        self.stack.push(value);
    }

    pub fn pop(&mut self) -> Option<StackValue> {
        self.stack.pop()
    }

    pub fn peek(&self, depth: usize) -> Option<&StackValue> {
        if depth < self.stack.len() {
            self.stack.get(self.stack.len() - 1 - depth)
        } else {
            None
        }
    }

    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    pub fn values(&self) -> &[StackValue] {
        &self.stack
    }
}

#[derive(Debug, Clone, Default)]
pub struct StackSimulationResult {
    pub block_exit_states: HashMap<usize, StackFrame>,
    // address -> stack state before that instruction executes.
    pub instruction_states: HashMap<u32, StackFrame>,
}

impl StackSimulationResult {
    pub fn get_block_exit_state(&self, block_id: usize) -> Option<&StackFrame> {
        self.block_exit_states.get(&block_id)
    }

    pub fn get_instruction_state(&self, address: u32) -> Option<&StackFrame> {
        self.instruction_states.get(&address)
    }
}

#[derive(Debug, Clone, Default)]
pub struct CfgBuildOutput {
    pub functions: Vec<FunctionCfg>,
    pub stack_simulation: StackSimulationResult,
}

#[derive(Debug, Default)]
struct SimulationArtifacts {
    stack_results: StackSimulationResult,
    discovered_entry_addresses: Vec<u32>,
    successors: HashMap<usize, Vec<usize>>,
    predecessors: HashMap<usize, Vec<usize>>,
}

#[derive(Debug, Clone)]
struct PendingSimulation {
    block_id: usize,
    initiator: Option<u32>,
    stack_state: StackFrame,
    heap_state: HeapLiteralState,
}

pub fn build_cfgs_and_discover_entries(ctx: &mut DecompileContext) -> Result<CfgBuildOutput> {
    debug!("running stack simulation, discovering functions and building control flow graphs...");

    let artifacts = simulate_and_discover(ctx)?;
    apply_semantic_edges(
        &mut ctx.basic_blocks,
        &artifacts.successors,
        &artifacts.predecessors,
    );
    append_discovered_entries(ctx, artifacts.discovered_entry_addresses);
    let mut functions = build_function_cfgs(ctx);
    assign_hidden_entry_names(ctx, &mut functions);

    info!(
        "{} functions discovered with their cfgs built",
        functions.len()
    );

    Ok(CfgBuildOutput {
        functions,
        stack_simulation: artifacts.stack_results,
    })
}

fn simulate_and_discover(ctx: &mut DecompileContext) -> Result<SimulationArtifacts> {
    let module_info = UdonModuleInfo::load_default_cached().map_err(|e| {
        DecompileError::new(format!(
            "failed to load UdonModuleInfo.json required by CFG/stack simulation: {e}"
        ))
    })?;
    let mut entry_set = ctx
        .entry_points
        .iter()
        .map(|x| x.entry_call_jump_target(ctx))
        .collect::<HashSet<_>>();
    let initial_heap_literals = build_heap_literal_u32_map(ctx);

    let mut out = SimulationArtifacts::default();
    let mut pending = VecDeque::<PendingSimulation>::new();
    let mut processed_blocks = HashSet::<(u32, usize)>::new();
    let mut jump_callsite_blocks = HashMap::<u32, Vec<usize>>::new();
    let mut block_entry_states = HashMap::<usize, (StackFrame, HeapLiteralState)>::new();

    for entry in &ctx.entry_points {
        let block_id = ctx.basic_block_id_by_start(entry.address).ok_or_else(|| {
            DecompileError::new(format!(
                "entry point '{}' points to missing basic block at 0x{:08X}",
                entry.name, entry.address
            ))
        })?;
        pending.push_back(PendingSimulation {
            block_id,
            initiator: Some(entry.address),
            stack_state: initial_stack_for_entry_point(ctx, entry),
            heap_state: initial_heap_literals.clone(),
        });
    }

    while let Some(PendingSimulation {
        block_id,
        initiator,
        mut stack_state,
        mut heap_state,
    }) = pending.pop_front()
    {
        if let Some(initiator) = initiator
            && !processed_blocks.insert((initiator, block_id))
        {
            continue;
        }
        if initiator.is_none() {
            let Some((saved_stack_state, saved_heap_state)) = block_entry_states.get(&block_id)
            else {
                continue;
            };
            stack_state = saved_stack_state.clone();
            heap_state = saved_heap_state.clone();
        } else {
            block_entry_states.insert(block_id, (stack_state.clone(), heap_state.clone()));
        }

        let block_instructions = ctx.basic_blocks.blocks[block_id].instructions.clone();
        let mut terminated = false;
        let mut block_successors = Vec::<usize>::new();

        for (_block_inst_id, addr, inst) in block_instructions.iter() {
            let global_inst_id = ctx.instructions.id_at_address(addr).ok_or_else(|| {
                DecompileError::new(format!(
                    "missing global instruction id for address 0x{addr:08X} in block {block_id}"
                ))
            })?;
            if initiator.is_some() {
                out.stack_results
                    .instruction_states
                    .insert(addr, stack_state.clone());
            }

            match inst.opcode {
                OpCode::Push => {
                    let address = inst.numeric_operand();
                    stack_state.push(StackValue::HeapAddress(address));
                }
                OpCode::Pop => {
                    let _ = stack_state.pop().ok_or_else(|| {
                        DecompileError::new(format!("stack underflow on POP at 0x{addr:08X}"))
                    })?;
                }
                OpCode::Copy => {
                    let target = stack_state.pop().ok_or_else(|| {
                        DecompileError::new(format!(
                            "stack underflow on COPY target at 0x{addr:08X}"
                        ))
                    })?;
                    let source = stack_state.pop().ok_or_else(|| {
                        DecompileError::new(format!(
                            "stack underflow on COPY source at 0x{addr:08X}"
                        ))
                    })?;
                    let target_address = target.heap_address();
                    let source_value = source.as_heap_value(&heap_state);
                    heap_state.insert(target_address, source_value);
                }
                OpCode::JumpIfFalse => {
                    terminated = true;

                    let _ = stack_state.pop().ok_or_else(|| {
                        DecompileError::new(format!(
                            "stack underflow on JUMP_IF_FALSE at 0x{addr:08X}"
                        ))
                    })?;
                    let target_addr = inst.numeric_operand();
                    enqueue_edge(
                        ctx,
                        &mut block_successors,
                        &mut pending,
                        initiator,
                        block_id,
                        target_addr,
                        &stack_state,
                        &heap_state,
                    )?;

                    let next_addr =
                        ctx.instructions
                            .next_address_of(global_inst_id)
                            .ok_or_else(|| {
                                DecompileError::new(format!(
                                    "missing fallthrough instruction after JUMP_IF_FALSE at 0x{addr:08X}"
                                ))
                            })?;
                    enqueue_edge(
                        ctx,
                        &mut block_successors,
                        &mut pending,
                        initiator,
                        block_id,
                        next_addr,
                        &stack_state,
                        &heap_state,
                    )?;
                    ctx.basic_blocks.blocks[block_id].block_type = BasicBlockType::Conditional;
                    break;
                }
                OpCode::Jump => {
                    terminated = true;

                    let target_addr = inst.numeric_operand();
                    register_jump_callsite(&mut jump_callsite_blocks, target_addr, block_id);
                    if ctx.is_out_of_program_counter_range(target_addr) {
                        ctx.basic_blocks.blocks[block_id].block_type = BasicBlockType::Return;
                        break;
                    }

                    let next_addr = ctx.instructions.next_address_of(global_inst_id);
                    let top = stack_state.peek(0);
                    let seems_like_call = entry_set.contains(&target_addr)
                        || looks_like_function_header(ctx, target_addr);
                    let is_returning_call =
                        next_addr.is_some_and(|x| matches_stack_literal(top, x, &heap_state));

                    if is_returning_call {
                        register_entry_target(
                            ctx,
                            &mut entry_set,
                            &mut out.discovered_entry_addresses,
                            &mut pending,
                            &jump_callsite_blocks,
                            target_addr,
                            &initial_heap_literals,
                        )?;
                        let _ = stack_state.pop().ok_or_else(|| {
                            DecompileError::new(format!(
                                "stack underflow on call-like JUMP at 0x{addr:08X}"
                            ))
                        })?;
                        let fallthrough = next_addr.ok_or_else(|| {
                            DecompileError::new(format!(
                                "missing fallthrough after call-like JUMP at 0x{addr:08X}"
                            ))
                        })?;
                        enqueue_edge(
                            ctx,
                            &mut block_successors,
                            &mut pending,
                            initiator,
                            block_id,
                            fallthrough,
                            &stack_state,
                            &heap_state,
                        )?;
                        ctx.basic_blocks.blocks[block_id].block_type = BasicBlockType::Normal;
                        break;
                    }

                    if seems_like_call {
                        register_entry_target(
                            ctx,
                            &mut entry_set,
                            &mut out.discovered_entry_addresses,
                            &mut pending,
                            &jump_callsite_blocks,
                            target_addr,
                            &initial_heap_literals,
                        )?;
                        ctx.basic_blocks.blocks[block_id].block_type = BasicBlockType::Return;
                        break;
                    }

                    // normal jump
                    enqueue_edge(
                        ctx,
                        &mut block_successors,
                        &mut pending,
                        initiator,
                        block_id,
                        target_addr,
                        &stack_state,
                        &heap_state,
                    )?;
                    ctx.basic_blocks.blocks[block_id].block_type = BasicBlockType::Jump;

                    break;
                }
                OpCode::JumpIndirect => {
                    terminated = true;

                    let operand = inst.numeric_operand();
                    if is_return_jump_operand(ctx, operand, &heap_state) {
                        ctx.basic_blocks.blocks[block_id].block_type = BasicBlockType::Return;
                    } else if let Some(info) = resolve_switch_info_for_jump_indirect(
                        ctx,
                        global_inst_id,
                        operand,
                        &ctx.symbol_name_by_address,
                        &ctx.symbol_type_by_address,
                        Some(module_info),
                    ) {
                        for target in &info.targets {
                            enqueue_edge(
                                ctx,
                                &mut block_successors,
                                &mut pending,
                                initiator,
                                block_id,
                                *target,
                                &stack_state,
                                &heap_state,
                            )?;
                        }
                        ctx.basic_blocks.blocks[block_id].switch_info = Some(info);
                        ctx.basic_blocks.blocks[block_id].block_type = BasicBlockType::Jump;
                    } else {
                        warn!("unrecognized JUMP_INDIRECT encountered at {addr:#06x}! ignoring...");
                        ctx.basic_blocks.blocks[block_id].block_type = BasicBlockType::Jump;
                    }

                    break;
                }
                OpCode::Extern => {
                    simulate_extern_call(
                        ctx,
                        inst,
                        &mut stack_state,
                        &mut heap_state,
                        module_info,
                    )?;
                }
                OpCode::Nop | OpCode::Annotation => {}
            }
        }

        if !terminated {
            let last_inst_id = block_instructions.last_id().ok_or_else(|| {
                DecompileError::new(format!(
                    "block {block_id} has no instructions during CFG simulation"
                ))
            })?;
            let last_addr = block_instructions.address_of(last_inst_id).ok_or_else(|| {
                DecompileError::new(format!(
                    "missing address for last instruction in block {block_id}"
                ))
            })?;
            let global_last_inst_id = ctx.instructions.id_at_address(last_addr).ok_or_else(|| {
                DecompileError::new(format!(
                    "missing global instruction id for last block instruction at 0x{last_addr:08X}"
                ))
            })?;
            if let Some(next_addr) = ctx.instructions.next_address_of(global_last_inst_id) {
                enqueue_edge(
                    ctx,
                    &mut block_successors,
                    &mut pending,
                    initiator,
                    block_id,
                    next_addr,
                    &stack_state,
                    &heap_state,
                )?;
            }
            ctx.basic_blocks.blocks[block_id].block_type = BasicBlockType::Normal;
        }

        replace_block_successors(&mut out, block_id, block_successors);
        if initiator.is_some() {
            out.stack_results
                .block_exit_states
                .insert(block_id, stack_state);
        }
    }

    out.discovered_entry_addresses.sort_unstable();
    out.discovered_entry_addresses.dedup();
    Ok(out)
}

fn enqueue_edge(
    ctx: &DecompileContext,
    block_successors: &mut Vec<usize>,
    pending: &mut VecDeque<PendingSimulation>,
    initiator: Option<u32>,
    _source_block_id: usize,
    target_address: u32,
    stack_state: &StackFrame,
    heap_state: &HeapLiteralState,
) -> Result<()> {
    let target_block_id = ctx.basic_block_id_by_start(target_address).ok_or_else(|| {
        DecompileError::new(format!(
            "cannot enqueue edge to non-block target 0x{target_address:08X}"
        ))
    })?;
    append_unique_successor(block_successors, target_block_id);
    if let Some(initiator) = initiator {
        pending.push_back(PendingSimulation {
            block_id: target_block_id,
            initiator: Some(initiator),
            stack_state: stack_state.clone(),
            heap_state: heap_state.clone(),
        });
    }
    Ok(())
}

fn append_unique_successor(successor_block_ids: &mut Vec<usize>, target_block_id: usize) {
    if !successor_block_ids.contains(&target_block_id) {
        successor_block_ids.push(target_block_id);
    }
}

fn replace_block_successors(
    artifacts: &mut SimulationArtifacts,
    source_block_id: usize,
    new_successors: Vec<usize>,
) {
    // A block can be re-simulated after new entry discovery.
    // In that case we need to replace this block's outgoing edges with the
    // latest result instead of only appending new edges, otherwise stale
    // successor/predecessor links from the previous classification remain.

    // insert new successors & get old successors
    let old_successors = artifacts
        .successors
        .insert(source_block_id, new_successors.clone())
        .unwrap_or_default();

    // remove predecessor of successors
    for old_target in old_successors {
        if let Some(predecessors) = artifacts.predecessors.get_mut(&old_target) {
            predecessors.retain(|pred| *pred != source_block_id);
        }
    }

    // add predecessor of new successors
    for new_target in new_successors {
        let predecessors = artifacts.predecessors.entry(new_target).or_default();
        if !predecessors.contains(&source_block_id) {
            predecessors.push(source_block_id);
        }
    }
}

fn register_jump_callsite(
    jump_callsite_blocks: &mut HashMap<u32, Vec<usize>>,
    target_addr: u32,
    block_id: usize,
) {
    let blocks = jump_callsite_blocks.entry(target_addr).or_default();
    if !blocks.contains(&block_id) {
        blocks.push(block_id);
    }
}

fn apply_semantic_edges(
    blocks: &mut BasicBlockCollection,
    successors: &HashMap<usize, Vec<usize>>,
    predecessors: &HashMap<usize, Vec<usize>>,
) {
    for (idx, block) in blocks.blocks.iter_mut().enumerate() {
        block.successors = successors.get(&idx).cloned().unwrap_or_default();
        block.predecessors = predecessors.get(&idx).cloned().unwrap_or_default();
    }
}

fn build_function_cfgs(ctx: &DecompileContext) -> Vec<FunctionCfg> {
    let mut functions = Vec::<FunctionCfg>::new();
    let mut selected_by_call_target = HashMap::<u32, usize>::new();
    for (index, entry) in ctx.entry_points.iter().enumerate() {
        let call_target = entry.entry_call_jump_target(ctx);
        match selected_by_call_target.get_mut(&call_target) {
            Some(existing_index) => {
                if !ctx.entry_points[*existing_index].exported && entry.exported {
                    *existing_index = index;
                }
            }
            None => {
                selected_by_call_target.insert(call_target, index);
            }
        }
    }

    let mut selected_indices = selected_by_call_target.into_values().collect::<Vec<_>>();
    selected_indices.sort_by_key(|idx| ctx.entry_points[*idx].address);

    for entry_index in selected_indices {
        let entry = &ctx.entry_points[entry_index];
        let Some(entry_block) = ctx.basic_block_id_by_start(entry.address) else {
            continue;
        };
        let mut reachable = collect_reachable_blocks(&ctx.basic_blocks, entry_block);
        reachable.sort_by_key(|x| ctx.basic_blocks.blocks[*x].start_address());
        functions.push(FunctionCfg {
            function_name: entry.name.clone(),
            is_function_public: entry.exported,
            entry_address: entry.address,
            entry_block,
            block_ids: reachable,
        });
    }
    functions
}

fn append_discovered_entries(ctx: &mut DecompileContext, discovered_addresses: Vec<u32>) {
    if discovered_addresses.is_empty() {
        return;
    }

    let mut existing_call_targets = ctx
        .entry_points
        .iter()
        .map(|x| x.entry_call_jump_target(ctx))
        .collect::<HashSet<_>>();

    for address in discovered_addresses {
        if existing_call_targets.contains(&address) {
            continue;
        }
        ctx.entry_points.push(DecompileSymbol {
            name: format!("{HIDDEN_ENTRY_PLACEHOLDER_PREFIX}0x{address:08X}"),
            address,
            exported: false,
            type_name: TYPE_SYSTEM_VOID.to_string(),
        });
        existing_call_targets.insert(address);
    }
    ctx.entry_points.sort_by_key(|x| x.address);
}

fn assign_hidden_entry_names(ctx: &mut DecompileContext, functions: &mut [FunctionCfg]) {
    if ctx.entry_points.is_empty() {
        return;
    }

    let mut used_names = ctx
        .entry_points
        .iter()
        .filter(|entry| entry.exported)
        .map(|entry| entry.name.clone())
        .collect::<HashSet<_>>();
    let mut generated_id = 0_u32;

    let symbol_by_address = ctx
        .symbols
        .iter()
        .map(|x| (x.address, x.name.as_str()))
        .collect::<HashMap<_, _>>();

    for function in functions.iter_mut() {
        if function.is_function_public {
            continue;
        }

        let entry_address = function.entry_address;
        let Some(entry_index) = ctx.entry_point_index_by_address(entry_address) else {
            continue;
        };

        let mut instruction_addresses = Vec::<u32>::new();
        for block_id in &function.block_ids {
            for (_inst_id, address, _inst) in ctx.basic_blocks.blocks[*block_id].instructions.iter()
            {
                instruction_addresses.push(address);
            }
        }

        // todo: there can be other types of assigning ret
        // like
        // __0___0_fibonacci__ret <- __intnl_SystemInt32_2 + ret
        // PUSH, __intnl_SystemInt32_2
        // PUSH, __0___0_fibonacci__ret
        // PUSH, __0___0_fibonacci__ret
        // EXTERN, "SystemInt32.__op_Addition__SystemInt32_SystemInt32__SystemInt32"

        // find a --copy-> __*_{__* | get}_name__ret
        let mut candidate_names = Vec::<String>::new();
        for idx in 0..instruction_addresses.len().saturating_sub(2) {
            let addr2 = instruction_addresses[idx + 1];
            let addr3 = instruction_addresses[idx + 2];
            let Some(copy_inst_id) = ctx.instructions.id_at_address(addr3) else {
                continue;
            };
            let Some(copy_inst) = ctx.instructions.get(copy_inst_id) else {
                continue;
            };
            if copy_inst.opcode != OpCode::Copy {
                continue;
            }
            let Some(push_inst_id) = ctx.instructions.id_at_address(addr2) else {
                continue;
            };
            let Some(push_inst) = ctx.instructions.get(push_inst_id) else {
                continue;
            };
            if push_inst.opcode != OpCode::Push {
                warn!("detected orphan COPY without a near PUSH, program may be broken");
                continue;
            }
            let symbol_addr = push_inst.numeric_operand();
            let Some(symbol_name) = symbol_by_address.get(&symbol_addr) else {
                continue;
            };
            if let Some(name) = parse_function_return_symbol_name(symbol_name) {
                candidate_names.push(name);
            }
        }

        if candidate_names.is_empty() {
            while used_names.contains(&format!("function_{generated_id}")) {
                generated_id += 1;
            }
            let generated = format!("function_{generated_id}");
            generated_id += 1;
            used_names.insert(generated.clone());
            ctx.entry_points[entry_index].name = generated;
            function.function_name = ctx.entry_points[entry_index].name.clone();
            continue;
        }

        let first = candidate_names[0].clone();
        if candidate_names.iter().any(|x| x != &first) {
            let distinct = candidate_names
                .iter()
                .cloned()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            warn!(
                "conflicting function names for hidden entry 0x{entry_address:08X}: {:?}, using {}",
                distinct, first
            );
        }
        used_names.insert(first.clone());
        ctx.entry_points[entry_index].name = first;
        function.function_name = ctx.entry_points[entry_index].name.clone();
    }
}

fn collect_reachable_blocks(blocks: &BasicBlockCollection, entry_block: usize) -> Vec<usize> {
    let mut visited = HashSet::<usize>::new();
    let mut stack = vec![entry_block];
    while let Some(block_id) = stack.pop() {
        if !visited.insert(block_id) {
            continue;
        }
        for succ in &blocks.blocks[block_id].successors {
            stack.push(*succ);
        }
    }
    visited.into_iter().collect()
}

fn parse_function_return_symbol_name(symbol_name: &str) -> Option<String> {
    if symbol_name.len() < 14 || !symbol_name.starts_with("__") || !symbol_name.ends_with("__ret") {
        return None;
    }

    let body = &symbol_name[2..symbol_name.len() - 5];
    if !body.contains("___") {
        let (id1, prop) = body.split_once("_get_")?;
        if !id1.is_empty() && id1.chars().all(|x| x.is_ascii_digit()) && !prop.is_empty() {
            return Some(format!("get_{prop}"));
        }
        return None;
    }

    let (id1, rest) = body.split_once("___")?;
    let (id2, method_name) = rest.split_once('_')?;
    if !id1.is_empty()
        && id1.chars().all(|x| x.is_ascii_digit())
        && !id2.is_empty()
        && id2.chars().all(|x| x.is_ascii_digit())
        && !method_name.is_empty()
    {
        return Some(method_name.to_string());
    }
    None
}

fn initial_stack_for_entry_point(ctx: &DecompileContext, entry: &DecompileSymbol) -> StackFrame {
    if entry.address == entry.entry_call_jump_target(ctx) {
        return initial_stack_for_hidden_entry();
    }
    StackFrame::default()
}

fn initial_stack_for_hidden_entry() -> StackFrame {
    let mut frame = StackFrame::default();
    frame.push(StackValue::HaltJump);
    frame
}

fn register_entry_target(
    ctx: &DecompileContext,
    entry_set: &mut HashSet<u32>,
    discovered_entry_addresses: &mut Vec<u32>,
    pending: &mut VecDeque<PendingSimulation>,
    jump_callsite_blocks: &HashMap<u32, Vec<usize>>,
    target_addr: u32,
    initial_heap_literals: &HeapLiteralState,
) -> Result<()> {
    if !entry_set.insert(target_addr) {
        return Ok(());
    }

    discovered_entry_addresses.push(target_addr);
    let target_block = ctx.basic_block_id_by_start(target_addr).ok_or_else(|| {
        DecompileError::new(format!(
            "discovered entry target 0x{target_addr:08X} has no basic block"
        ))
    })?;
    pending.push_back(PendingSimulation {
        block_id: target_block,
        initiator: Some(target_addr),
        stack_state: initial_stack_for_hidden_entry(),
        heap_state: initial_heap_literals.clone(),
    });
    if let Some(caller_blocks) = jump_callsite_blocks.get(&target_addr) {
        for block_id in caller_blocks {
            pending.push_back(PendingSimulation {
                block_id: *block_id,
                initiator: None,
                stack_state: StackFrame::default(),
                heap_state: HashMap::default(),
            });
        }
    }
    Ok(())
}

fn looks_like_function_header(ctx: &DecompileContext, target: u32) -> bool {
    let Some(header_addr) = target.checked_sub(OpCode::Push.size()) else {
        return false;
    };
    is_header_push_address(ctx, header_addr)
}

fn is_header_push_address(ctx: &DecompileContext, address: u32) -> bool {
    let Some(inst_id) = ctx.instructions.id_at_address(address) else {
        return false;
    };
    let Some(inst) = ctx.instructions.get(inst_id) else {
        return false;
    };
    if inst.opcode != OpCode::Push {
        return false;
    }
    let operand = inst.numeric_operand();

    if ctx.symbol_name_by_address.get(&operand).map(|x| x.as_str())
        != Some(SYMBOL_CONST_SYSTEM_UINT32_0)
    {
        return false;
    }
    if ctx.heap_u32_literals.get(&operand).copied() != Some(u32::MAX) {
        return false;
    }

    let Some(prev_id) = ctx.instructions.prev_of(inst_id) else {
        return true;
    };
    let Some(prev_inst) = ctx.instructions.get(prev_id) else {
        return false;
    };
    if prev_inst.opcode != OpCode::JumpIndirect {
        return false;
    }
    let prev_operand = prev_inst.numeric_operand();
    ctx.symbol_name_by_address
        .get(&prev_operand)
        .is_some_and(|x| x == SYMBOL_RETURN_JUMP_U32)
}

fn build_heap_literal_u32_map(ctx: &DecompileContext) -> HashMap<u32, HeapValue> {
    let mut out = HashMap::<u32, HeapValue>::with_capacity(ctx.heap_u32_literals.len());
    for (address, value) in &ctx.heap_u32_literals {
        out.insert(*address, HeapValue::U32(*value));
    }
    out
}

fn matches_stack_literal(
    stack_value: Option<&StackValue>,
    expected: u32,
    heap_state: &HeapLiteralState,
) -> bool {
    let Some(stack_value) = stack_value else {
        return false;
    };
    stack_value.as_u32_literal(heap_state) == Some(expected)
}

fn simulate_extern_call(
    ctx: &DecompileContext,
    inst: &crate::udon_asm::AsmInstruction,
    stack_state: &mut StackFrame,
    heap_state: &mut HeapLiteralState,
    module_info: &UdonModuleInfo,
) -> Result<()> {
    let operand = inst.numeric_operand();
    let signature = ctx.heap_string_literals.get(&operand).ok_or_else(|| {
        DecompileError::new(format!(
            "EXTERN operand 0x{operand:08X} does not resolve to a heap string literal"
        ))
    })?;
    let info = module_info.get_function_info(signature).ok_or_else(|| {
        DecompileError::new(format!(
            "missing module info for extern signature: {signature}"
        ))
    })?;

    let mut first_popped = None::<StackValue>;
    for idx in 0..info.parameter_count() {
        let popped = stack_state.pop().ok_or_else(|| {
            DecompileError::new(format!(
                "stack underflow while simulating EXTERN call: signature={signature}"
            ))
        })?;
        if idx == 0 {
            first_popped = Some(popped.clone());
        }
    }

    // Same behavior as Python BlockStackSimulator:
    // non-void extern writes into the last formal parameter (typically an out slot).
    if !info.returns_void {
        let target = first_popped.ok_or_else(|| {
            DecompileError::new(format!(
                "EXTERN non-void signature missing destination slot on stack: {signature}"
            ))
        })?;
        let target_address = target.heap_address();
        heap_state.insert(target_address, HeapValue::Unknown);
    }
    Ok(())
}
