use std::collections::HashSet;

use crate::decompiler::Result;
use crate::decompiler::ir::{
    ControlFlowGraph, ControlFlowNode, IrBlock, IrBlockContainer, IrContainerKind, IrFunction,
    IrIf, IrJump, IrLeave, IrStatement, IrSwitch, compute_dominance,
};
use crate::decompiler::transform::ir_utils::{find_container_mut, iter_block_containers};
use crate::decompiler::transform::pass_base::{ITransform, TransformContext};

pub struct LoopDetection;

impl ITransform for LoopDetection {

    fn run(&self, function: &mut IrFunction, context: &mut TransformContext<'_, '_>) -> Result<()> {
        let function_body_id = function.body.id;
        let mut state = LoopDetectionState::from_context(function, context);
        let container_ids = iter_block_containers(function);
        for container_id in container_ids {
            let Some(container) = find_container_mut(&mut function.body, container_id) else {
                continue;
            };
            detect_in_container(container, function_body_id, &mut state);
        }
        state.commit(context);
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct LoopDetectionState {
    next_container_id: u32,
    synthetic_block_address: i64,
}

impl LoopDetectionState {
    fn from_context(function: &IrFunction, context: &TransformContext<'_, '_>) -> Self {
        let current = context
            .program_context
            .metadata
            .get("_synthetic_block_addr")
            .copied()
            .unwrap_or(-1);
        Self {
            next_container_id: max_container_id(&function.body).saturating_add(1),
            synthetic_block_address: current,
        }
    }

    fn alloc_container_id(&mut self) -> u32 {
        let id = self.next_container_id;
        self.next_container_id = self.next_container_id.saturating_add(1);
        id
    }

    fn alloc_block_address(&mut self) -> u32 {
        let current = self.synthetic_block_address;
        self.synthetic_block_address -= 1;
        (current as i32) as u32
    }

    fn commit(self, context: &mut TransformContext<'_, '_>) {
        context.program_context.metadata.insert(
            "_synthetic_block_addr".to_string(),
            self.synthetic_block_address,
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExitSearch {
    UseHeuristic,
    NoExit,
    Exit(usize),
}

fn detect_in_container(
    container: &mut IrBlockContainer,
    function_body_container_id: u32,
    state: &mut LoopDetectionState,
) {
    if !matches!(
        container.kind,
        IrContainerKind::Block | IrContainerKind::Switch
    ) {
        return;
    }

    let cfg = ControlFlowGraph::new(container, function_body_container_id);
    if cfg.nodes.is_empty() {
        return;
    }

    let mut block_addrs = dominator_post_order_block_addrs(&cfg);
    if block_addrs.is_empty() {
        block_addrs = container
            .blocks
            .iter()
            .map(|x| x.start_address)
            .collect::<Vec<_>>();
    }

    for block_addr in block_addrs {
        let _ = detect_switch_body(container, &cfg, block_addr, state);
        let _ = detect_loop_from_head(container, &cfg, block_addr, state);
    }
}

fn dominator_post_order_block_addrs(cfg: &ControlFlowGraph) -> Vec<u32> {
    if cfg.nodes.is_empty() {
        return Vec::new();
    }

    fn visit(node: usize, cfg: &ControlFlowGraph, out: &mut Vec<u32>) {
        if let Some(children) = cfg.nodes[node].dominator_tree_children.as_ref() {
            for child in children {
                visit(*child, cfg, out);
            }
        }
        if let Some(addr) = cfg.nodes[node].block_start_address {
            out.push(addr);
        }
    }

    let mut out = Vec::<u32>::new();
    visit(0, cfg, &mut out);
    out
}

fn detect_switch_body(
    container: &mut IrBlockContainer,
    cfg: &ControlFlowGraph,
    block_address: u32,
    state: &mut LoopDetectionState,
) -> bool {
    let Some(head_idx) = node_index_by_address(cfg, block_address) else {
        return false;
    };
    let Some(block_index) = block_index_by_address(container, block_address) else {
        return false;
    };
    let Some(block) = container.blocks.get(block_index) else {
        return false;
    };
    if has_container_entry(block) {
        return false;
    }
    let Some(IrStatement::Switch(switch_inst)) = block.statements.last().cloned() else {
        return false;
    };

    let mut visited = vec![false; cfg.nodes.len()];
    let mut nodes_in_switch = vec![head_idx];
    visited[head_idx] = true;

    let mut exit_point = extend_loop(
        cfg,
        container,
        head_idx,
        &mut nodes_in_switch,
        &mut visited,
        true,
    );

    if let Some(exit_idx) = exit_point
        && cfg.nodes[exit_idx].predecessors.len() == 1
        && !cfg.has_reachable_exit(exit_idx)
    {
        for node_idx in dominator_pre_order(cfg, exit_idx) {
            if visited[node_idx] {
                continue;
            }
            visited[node_idx] = true;
            nodes_in_switch.push(node_idx);
        }
        exit_point = None;
    }

    nodes_in_switch.sort_by_key(|idx| -cfg.nodes[*idx].post_order_number);
    for node_idx in &nodes_in_switch {
        visited[*node_idx] = false;
    }

    let switch_container_id = state.alloc_container_id();
    let mut switch_container = IrBlockContainer {
        id: switch_container_id,
        blocks: vec![IrBlock {
            statements: vec![IrStatement::Switch(switch_inst)],
            start_address: state.alloc_block_address(),
            should_emit_label: false,
        }],
        kind: IrContainerKind::Switch,
        should_emit_exit_label: false,
    };

    let exit_target = exit_point.and_then(|idx| cfg.nodes[idx].block_start_address);
    move_blocks_into_container(container, cfg, &nodes_in_switch, &mut switch_container);
    rewrite_switch_exit_edges(&mut switch_container, exit_target);

    if let Some(head_block_idx) = block_index_by_address(container, block_address)
        && let Some(head_block) = container.blocks.get_mut(head_block_idx)
        && let Some(last) = head_block.statements.last_mut()
    {
        *last = IrStatement::BlockContainer(switch_container);
        if let Some(exit_target) = exit_target {
            head_block.statements.push(IrStatement::Jump(IrJump {
                target_address: exit_target,
            }));
        }
        return true;
    }
    false
}

fn detect_loop_from_head(
    container: &mut IrBlockContainer,
    cfg: &ControlFlowGraph,
    block_address: u32,
    state: &mut LoopDetectionState,
) -> bool {
    let Some(head_idx) = node_index_by_address(cfg, block_address) else {
        return false;
    };
    if let Some(block_idx) = block_index_by_address(container, block_address)
        && let Some(block) = container.blocks.get(block_idx)
        && has_container_entry(block)
    {
        return false;
    }

    let mut visited = vec![false; cfg.nodes.len()];
    let mut loop_nodes = Vec::<usize>::new();

    for pred_idx in cfg.nodes[head_idx].predecessors.clone() {
        if !dominates(cfg, head_idx, pred_idx) {
            continue;
        }
        if loop_nodes.is_empty() {
            loop_nodes.push(head_idx);
            visited[head_idx] = true;
        }
        traverse_pre_order(
            pred_idx,
            |idx| cfg.nodes[idx].predecessors.clone(),
            &mut visited,
            &mut |idx| loop_nodes.push(idx),
        );
    }

    if loop_nodes.is_empty() {
        return false;
    }

    include_nested_containers(cfg, container, &mut loop_nodes, &mut visited);
    let exit_point = extend_loop(
        cfg,
        container,
        head_idx,
        &mut loop_nodes,
        &mut visited,
        false,
    );

    loop_nodes.sort_by_key(|idx| -cfg.nodes[*idx].post_order_number);
    for node_idx in &loop_nodes {
        visited[*node_idx] = false;
    }

    construct_loop(container, cfg, &loop_nodes, exit_point, state)
}

fn has_container_entry(block: &IrBlock) -> bool {
    matches!(
        block.statements.first(),
        Some(IrStatement::BlockContainer(_))
    )
}

fn include_nested_containers(
    cfg: &ControlFlowGraph,
    container: &IrBlockContainer,
    loop_nodes: &mut Vec<usize>,
    visited: &mut [bool],
) {
    let mut index = 0usize;
    while index < loop_nodes.len() {
        include_nested_block(cfg, container, loop_nodes[index], loop_nodes, visited);
        index += 1;
    }
}

fn include_nested_block(
    cfg: &ControlFlowGraph,
    container: &IrBlockContainer,
    node_idx: usize,
    loop_nodes: &mut Vec<usize>,
    visited: &mut [bool],
) {
    let Some(block_addr) = cfg.nodes[node_idx].block_start_address else {
        return;
    };
    let Some(block_idx) = block_index_by_address(container, block_addr) else {
        return;
    };
    let Some(block) = container.blocks.get(block_idx) else {
        return;
    };
    let Some(IrStatement::BlockContainer(nested_container)) = block.statements.first() else {
        return;
    };

    if let Some(entry_block) = nested_container.entry_block()
        && let Some(entry_idx) = node_index_by_address(cfg, entry_block.start_address)
    {
        include_nested_block(cfg, container, entry_idx, loop_nodes, visited);
    }

    for nested_block in nested_container.blocks.iter().skip(1) {
        let Some(nested_idx) = node_index_by_address(cfg, nested_block.start_address) else {
            continue;
        };
        if visited[nested_idx] {
            continue;
        }
        visited[nested_idx] = true;
        loop_nodes.push(nested_idx);
    }
}

fn extend_loop(
    cfg: &ControlFlowGraph,
    container: &IrBlockContainer,
    head_idx: usize,
    loop_nodes: &mut Vec<usize>,
    visited: &mut [bool],
    treat_back_edges_as_exits: bool,
) -> Option<usize> {
    match find_exit_point(
        cfg,
        container,
        head_idx,
        loop_nodes.as_slice(),
        visited,
        treat_back_edges_as_exits,
    ) {
        ExitSearch::Exit(exit_idx) => {
            add_dominated_until_exit(cfg, head_idx, loop_nodes, visited, Some(exit_idx));
            Some(exit_idx)
        }
        ExitSearch::NoExit => {
            add_dominated_until_exit(cfg, head_idx, loop_nodes, visited, None);
            None
        }
        ExitSearch::UseHeuristic => {
            extend_loop_heuristic(cfg, head_idx, head_idx, loop_nodes, visited);
            None
        }
    }
}

fn find_exit_point(
    cfg: &ControlFlowGraph,
    container: &IrBlockContainer,
    head_idx: usize,
    natural_loop: &[usize],
    visited: &[bool],
    treat_back_edges_as_exits: bool,
) -> ExitSearch {
    let mut has_reachable_exit = cfg.has_reachable_exit(head_idx);
    if !has_reachable_exit && treat_back_edges_as_exits {
        has_reachable_exit = cfg.nodes[head_idx]
            .predecessors
            .iter()
            .any(|idx| dominates(cfg, head_idx, *idx));
    }

    if !has_reachable_exit {
        if let Some(best) = pick_exit_point(cfg, container, head_idx) {
            return ExitSearch::Exit(best);
        }
        return ExitSearch::NoExit;
    }

    let (rev_cfg, exit_idx, exit_node_arity) =
        prepare_reverse_cfg(cfg, head_idx, treat_back_edges_as_exits);
    let mut common_ancestor = head_idx;
    if !is_reachable(&rev_cfg[common_ancestor]) {
        return ExitSearch::UseHeuristic;
    }

    for node_idx in natural_loop {
        if is_reachable(&rev_cfg[*node_idx]) {
            common_ancestor = find_common_dominator(*node_idx, common_ancestor, &rev_cfg);
        }
    }

    while common_ancestor != exit_idx {
        if !visited[common_ancestor] && validate_exit_point(cfg, head_idx, common_ancestor) {
            return ExitSearch::Exit(common_ancestor);
        }
        let Some(idom) = rev_cfg[common_ancestor].immediate_dominator else {
            break;
        };
        common_ancestor = idom;
    }

    if exit_node_arity > 1 {
        return ExitSearch::UseHeuristic;
    }
    ExitSearch::NoExit
}

fn prepare_reverse_cfg(
    cfg: &ControlFlowGraph,
    head_idx: usize,
    treat_back_edges_as_exits: bool,
) -> (Vec<ControlFlowNode>, usize, usize) {
    let mut rev_cfg = cfg
        .nodes
        .iter()
        .enumerate()
        .map(|(idx, node)| ControlFlowNode::new(idx, node.block_start_address))
        .collect::<Vec<_>>();

    let mut node_treated_as_exit = None::<usize>;
    let mut multiple_exit_nodes = false;

    let exit_idx = rev_cfg.len();
    rev_cfg.push(ControlFlowNode::new(exit_idx, None));

    for (idx, node) in cfg.nodes.iter().enumerate() {
        if !dominates(cfg, head_idx, idx) {
            continue;
        }
        for succ in &node.successors {
            if dominates(cfg, head_idx, *succ) && (!treat_back_edges_as_exits || head_idx != *succ)
            {
                add_edge(&mut rev_cfg, *succ, idx);
            } else {
                if node_treated_as_exit.is_none() {
                    node_treated_as_exit = Some(*succ);
                }
                if node_treated_as_exit != Some(*succ) {
                    multiple_exit_nodes = true;
                }
                add_edge(&mut rev_cfg, exit_idx, idx);
            }
        }
        if cfg.has_direct_exit_out_of_container(idx) {
            add_edge(&mut rev_cfg, exit_idx, idx);
        }
    }

    let exit_node_arity = if multiple_exit_nodes {
        2
    } else if node_treated_as_exit.is_some() {
        1
    } else {
        0
    };

    compute_dominance(exit_idx, &mut rev_cfg);
    (rev_cfg, exit_idx, exit_node_arity)
}

fn validate_exit_point(cfg: &ControlFlowGraph, head_idx: usize, exit_idx: usize) -> bool {
    fn walk(cfg: &ControlFlowGraph, head_idx: usize, exit_idx: usize, node_idx: usize) -> bool {
        if !cfg.has_reachable_exit(node_idx) {
            return true;
        }
        for succ in &cfg.nodes[node_idx].successors {
            if *succ != head_idx
                && dominates(cfg, head_idx, *succ)
                && !dominates(cfg, exit_idx, *succ)
            {
                return false;
            }
        }
        for child in cfg.nodes[node_idx]
            .dominator_tree_children
            .clone()
            .unwrap_or_default()
        {
            if !walk(cfg, head_idx, exit_idx, child) {
                return false;
            }
        }
        true
    }

    walk(cfg, head_idx, exit_idx, exit_idx)
}

fn pick_exit_point(
    cfg: &ControlFlowGraph,
    container: &IrBlockContainer,
    head_idx: usize,
) -> Option<usize> {
    fn walk(
        cfg: &ControlFlowGraph,
        container: &IrBlockContainer,
        node_idx: usize,
        best: &mut Option<usize>,
        best_start: &mut u32,
    ) {
        let Some(start_addr) = cfg.nodes[node_idx].block_start_address else {
            return;
        };
        if start_addr > *best_start
            && !cfg.has_reachable_exit(node_idx)
            && container
                .blocks
                .iter()
                .any(|x| x.start_address == start_addr)
        {
            *best = Some(node_idx);
            *best_start = start_addr;
            return;
        }
        for child in cfg.nodes[node_idx]
            .dominator_tree_children
            .clone()
            .unwrap_or_default()
        {
            walk(cfg, container, child, best, best_start);
        }
    }

    let mut best = None::<usize>;
    let mut best_start = 0u32;
    for child in cfg.nodes[head_idx]
        .dominator_tree_children
        .clone()
        .unwrap_or_default()
    {
        walk(cfg, container, child, &mut best, &mut best_start);
    }
    best
}

fn add_dominated_until_exit(
    cfg: &ControlFlowGraph,
    head_idx: usize,
    loop_nodes: &mut Vec<usize>,
    visited: &mut [bool],
    exit_idx: Option<usize>,
) {
    let mut stack = vec![head_idx];
    while let Some(node_idx) = stack.pop() {
        if Some(node_idx) != exit_idx && !visited[node_idx] {
            visited[node_idx] = true;
            loop_nodes.push(node_idx);
        }
        if Some(node_idx) == exit_idx {
            continue;
        }
        for child in cfg.nodes[node_idx]
            .dominator_tree_children
            .clone()
            .unwrap_or_default()
            .into_iter()
            .rev()
        {
            stack.push(child);
        }
    }
}

fn extend_loop_heuristic(
    cfg: &ControlFlowGraph,
    head_idx: usize,
    candidate_idx: usize,
    loop_nodes: &mut Vec<usize>,
    visited: &mut [bool],
) {
    let _ = head_idx;

    if !visited[candidate_idx] {
        let mut additional_nodes = Vec::<usize>::new();
        traverse_pre_order(
            candidate_idx,
            |idx| cfg.nodes[idx].predecessors.clone(),
            visited,
            &mut |idx| additional_nodes.push(idx),
        );

        let mut new_exit_points = HashSet::<usize>::new();
        for node_idx in &additional_nodes {
            for succ in &cfg.nodes[*node_idx].successors {
                if !visited[*succ] {
                    new_exit_points.insert(*succ);
                }
            }
        }

        for node_idx in &additional_nodes {
            visited[*node_idx] = false;
        }

        let removed_exit_points = additional_nodes
            .iter()
            .filter(|idx| is_exit_point(cfg, **idx, visited))
            .count();
        let added_exit_points = new_exit_points
            .iter()
            .filter(|idx| !is_exit_point(cfg, **idx, visited))
            .count();

        if removed_exit_points > added_exit_points {
            traverse_pre_order(
                candidate_idx,
                |idx| cfg.nodes[idx].predecessors.clone(),
                visited,
                &mut |idx| loop_nodes.push(idx),
            );
        }
    }

    for child in cfg.nodes[candidate_idx]
        .dominator_tree_children
        .clone()
        .unwrap_or_default()
    {
        extend_loop_heuristic(cfg, head_idx, child, loop_nodes, visited);
    }
}

fn is_exit_point(cfg: &ControlFlowGraph, node_idx: usize, visited: &[bool]) -> bool {
    if visited[node_idx] {
        return false;
    }
    cfg.nodes[node_idx]
        .predecessors
        .iter()
        .any(|pred| visited[*pred])
}

fn construct_loop(
    container: &mut IrBlockContainer,
    cfg: &ControlFlowGraph,
    loop_nodes: &[usize],
    exit_point: Option<usize>,
    state: &mut LoopDetectionState,
) -> bool {
    let Some(old_entry_idx) = loop_nodes.first().copied() else {
        return false;
    };
    let Some(old_entry_addr) = cfg.nodes[old_entry_idx].block_start_address else {
        return false;
    };
    let Some(old_entry_block_idx) = block_index_by_address(container, old_entry_addr) else {
        return false;
    };
    let old_entry_snapshot = container.blocks[old_entry_block_idx].clone();
    let exit_target = exit_point.and_then(|idx| cfg.nodes[idx].block_start_address);

    let loop_id = state.alloc_container_id();
    let new_entry_addr = state.alloc_block_address();
    let mut loop_container = IrBlockContainer {
        id: loop_id,
        blocks: vec![IrBlock {
            statements: old_entry_snapshot.statements.clone(),
            start_address: new_entry_addr,
            should_emit_label: false,
        }],
        kind: IrContainerKind::Loop,
        should_emit_exit_label: false,
    };

    move_blocks_into_container(container, cfg, loop_nodes, &mut loop_container);
    rewrite_loop_control_flow(
        &mut loop_container,
        old_entry_addr,
        new_entry_addr,
        exit_target,
        loop_id,
    );

    if let Some(entry_idx) = block_index_by_address(container, old_entry_addr)
        && let Some(entry_block) = container.blocks.get_mut(entry_idx)
    {
        entry_block.statements = vec![IrStatement::BlockContainer(loop_container)];
        if let Some(exit_target) = exit_target {
            entry_block.statements.push(IrStatement::Jump(IrJump {
                target_address: exit_target,
            }));
        }
        return true;
    }
    false
}

fn move_blocks_into_container(
    container: &mut IrBlockContainer,
    cfg: &ControlFlowGraph,
    nodes: &[usize],
    target_container: &mut IrBlockContainer,
) {
    let Some(head_idx) = nodes.first().copied() else {
        return;
    };
    let head_addr = cfg.nodes[head_idx].block_start_address;
    for node_idx in nodes.iter().skip(1) {
        let Some(address) = cfg.nodes[*node_idx].block_start_address else {
            continue;
        };
        if Some(address) == head_addr {
            continue;
        }
        if let Some(index) = block_index_by_address(container, address) {
            let block = container.blocks.remove(index);
            target_container.blocks.push(block);
        }
    }
}

fn rewrite_switch_exit_edges(container: &mut IrBlockContainer, exit_target: Option<u32>) {
    let Some(exit_target) = exit_target else {
        return;
    };
    let switch_id = container.id;
    for block in &mut container.blocks {
        for statement in &mut block.statements {
            rewrite_switch_exit_statement(statement, switch_id, exit_target);
        }
    }
}

fn rewrite_switch_exit_statement(statement: &mut IrStatement, switch_id: u32, exit_target: u32) {
    match statement {
        IrStatement::Jump(IrJump { target_address }) => {
            if *target_address == exit_target {
                *statement = IrStatement::Leave(IrLeave {
                    target_container_id: switch_id,
                });
            }
        }
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            rewrite_switch_exit_statement(true_statement, switch_id, exit_target);
            if let Some(false_statement) = false_statement.as_mut() {
                rewrite_switch_exit_statement(false_statement, switch_id, exit_target);
            }
        }
        IrStatement::BlockContainer(container) => {
            for block in &mut container.blocks {
                for nested in &mut block.statements {
                    rewrite_switch_exit_statement(nested, switch_id, exit_target);
                }
            }
        }
        IrStatement::Block(block) => {
            for nested in &mut block.statements {
                rewrite_switch_exit_statement(nested, switch_id, exit_target);
            }
        }
        _ => {}
    }
}

fn rewrite_loop_control_flow(
    container: &mut IrBlockContainer,
    old_entry_address: u32,
    new_entry_address: u32,
    exit_target: Option<u32>,
    loop_id: u32,
) {
    for block in &mut container.blocks {
        for statement in &mut block.statements {
            rewrite_loop_statement(
                statement,
                old_entry_address,
                new_entry_address,
                exit_target,
                loop_id,
            );
        }
    }
}

fn rewrite_loop_statement(
    statement: &mut IrStatement,
    old_entry_address: u32,
    new_entry_address: u32,
    exit_target: Option<u32>,
    loop_id: u32,
) {
    match statement {
        IrStatement::Jump(IrJump { target_address }) => {
            if *target_address == old_entry_address {
                *target_address = new_entry_address;
            }
            if exit_target.is_some_and(|x| x == *target_address) {
                *statement = IrStatement::Leave(IrLeave {
                    target_container_id: loop_id,
                });
            }
        }
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            rewrite_loop_statement(
                true_statement,
                old_entry_address,
                new_entry_address,
                exit_target,
                loop_id,
            );
            if let Some(false_statement) = false_statement.as_mut() {
                rewrite_loop_statement(
                    false_statement,
                    old_entry_address,
                    new_entry_address,
                    exit_target,
                    loop_id,
                );
            }
        }
        IrStatement::Switch(IrSwitch {
            cases,
            default_target,
            ..
        }) => {
            for target in cases.values_mut() {
                if *target == old_entry_address {
                    *target = new_entry_address;
                }
            }
            if let Some(default_target) = default_target.as_mut()
                && *default_target == old_entry_address
            {
                *default_target = new_entry_address;
            }
        }
        IrStatement::Block(block) => {
            for nested in &mut block.statements {
                rewrite_loop_statement(
                    nested,
                    old_entry_address,
                    new_entry_address,
                    exit_target,
                    loop_id,
                );
            }
        }
        IrStatement::BlockContainer(container) => rewrite_loop_control_flow(
            container,
            old_entry_address,
            new_entry_address,
            exit_target,
            loop_id,
        ),
        IrStatement::HighLevelSwitch(switch_stmt) => {
            for section in &mut switch_stmt.sections {
                for nested in &mut section.body.statements {
                    rewrite_loop_statement(
                        nested,
                        old_entry_address,
                        new_entry_address,
                        exit_target,
                        loop_id,
                    );
                }
            }
        }
        IrStatement::HighLevelWhile(while_stmt) => rewrite_loop_control_flow(
            &mut while_stmt.body,
            old_entry_address,
            new_entry_address,
            exit_target,
            loop_id,
        ),
        IrStatement::HighLevelDoWhile(do_while_stmt) => rewrite_loop_control_flow(
            &mut do_while_stmt.body,
            old_entry_address,
            new_entry_address,
            exit_target,
            loop_id,
        ),
        _ => {}
    }
}

fn traverse_pre_order<FNeighbors, FVisit>(
    start: usize,
    mut neighbors: FNeighbors,
    visited: &mut [bool],
    visit: &mut FVisit,
) where
    FNeighbors: FnMut(usize) -> Vec<usize>,
    FVisit: FnMut(usize),
{
    let mut stack = vec![start];
    while let Some(node_idx) = stack.pop() {
        if visited[node_idx] {
            continue;
        }
        visited[node_idx] = true;
        visit(node_idx);
        for next in neighbors(node_idx).into_iter().rev() {
            stack.push(next);
        }
    }
}

fn dominator_pre_order(cfg: &ControlFlowGraph, root: usize) -> Vec<usize> {
    let mut out = Vec::<usize>::new();
    let mut stack = vec![root];
    while let Some(node_idx) = stack.pop() {
        out.push(node_idx);
        for child in cfg.nodes[node_idx]
            .dominator_tree_children
            .clone()
            .unwrap_or_default()
            .into_iter()
            .rev()
        {
            stack.push(child);
        }
    }
    out
}

fn dominates(cfg: &ControlFlowGraph, dominator: usize, dominated: usize) -> bool {
    if dominator == dominated {
        return true;
    }
    let mut current = Some(dominated);
    while let Some(node_idx) = current {
        if node_idx == dominator {
            return true;
        }
        current = cfg.nodes[node_idx].immediate_dominator;
    }
    false
}

fn add_edge(nodes: &mut [ControlFlowNode], source: usize, target: usize) {
    if !nodes[source].successors.contains(&target) {
        nodes[source].successors.push(target);
    }
    if !nodes[target].predecessors.contains(&source) {
        nodes[target].predecessors.push(source);
    }
}

fn is_reachable(node: &ControlFlowNode) -> bool {
    node.dominator_tree_children.is_some()
}

fn find_common_dominator(mut a: usize, mut b: usize, nodes: &[ControlFlowNode]) -> usize {
    while a != b {
        while nodes[a].post_order_number < nodes[b].post_order_number {
            let Some(next) = nodes[a].immediate_dominator else {
                return a;
            };
            a = next;
        }
        while nodes[b].post_order_number < nodes[a].post_order_number {
            let Some(next) = nodes[b].immediate_dominator else {
                return b;
            };
            b = next;
        }
    }
    a
}

fn node_index_by_address(cfg: &ControlFlowGraph, address: u32) -> Option<usize> {
    cfg.nodes
        .iter()
        .position(|x| x.block_start_address == Some(address))
}

fn block_index_by_address(container: &IrBlockContainer, address: u32) -> Option<usize> {
    container
        .blocks
        .iter()
        .position(|x| x.start_address == address)
}

fn max_container_id(container: &IrBlockContainer) -> u32 {
    let mut max_id = container.id;
    for block in &container.blocks {
        for statement in &block.statements {
            max_id = max_id.max(max_container_id_in_statement(statement));
        }
    }
    max_id
}

fn max_container_id_in_statement(statement: &IrStatement) -> u32 {
    match statement {
        IrStatement::BlockContainer(container) => max_container_id(container),
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            let mut max_id = max_container_id_in_statement(true_statement);
            if let Some(false_statement) = false_statement.as_ref() {
                max_id = max_id.max(max_container_id_in_statement(false_statement));
            }
            max_id
        }
        IrStatement::Block(block) => block
            .statements
            .iter()
            .map(max_container_id_in_statement)
            .max()
            .unwrap_or(0),
        IrStatement::HighLevelSwitch(switch_stmt) => switch_stmt
            .sections
            .iter()
            .flat_map(|section| section.body.statements.iter())
            .map(max_container_id_in_statement)
            .max()
            .unwrap_or(0),
        IrStatement::HighLevelWhile(while_stmt) => max_container_id(&while_stmt.body),
        IrStatement::HighLevelDoWhile(do_while_stmt) => max_container_id(&do_while_stmt.body),
        _ => 0,
    }
}
