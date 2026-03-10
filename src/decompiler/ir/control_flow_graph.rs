use std::collections::{HashMap, HashSet};

use super::control_flow_node::ControlFlowNode;
use super::dominance::{compute_dominance, mark_nodes_with_reachable_exits};
use super::nodes::{
    IrBlock, IrBlockContainer, IrHighLevelDoWhile, IrHighLevelSwitch, IrHighLevelWhile, IrIf,
    IrJump, IrLeave, IrReturn, IrStatement, IrSwitch,
};

#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    pub container_id: u32,
    pub function_body_container_id: u32,
    pub nodes: Vec<ControlFlowNode>,

    block_index_by_address: HashMap<u32, usize>,
    node_has_direct_exit_out_of_container: Vec<bool>,
    node_has_reachable_exit: Vec<bool>,
}

impl ControlFlowGraph {
    pub fn new(container: &IrBlockContainer, function_body_container_id: u32) -> Self {
        let mut out = Self {
            container_id: container.id,
            function_body_container_id,
            nodes: Vec::new(),
            block_index_by_address: HashMap::new(),
            node_has_direct_exit_out_of_container: Vec::new(),
            node_has_reachable_exit: Vec::new(),
        };

        out.initialize_nodes(container);
        out.create_edges(container);

        if !out.nodes.is_empty() {
            compute_dominance(0, &mut out.nodes);
            out.node_has_reachable_exit = mark_nodes_with_reachable_exits(&out.nodes);
            let leaving = out.find_nodes_with_exits_out_of_container();
            out.node_has_reachable_exit = out
                .node_has_reachable_exit
                .iter()
                .zip(leaving)
                .map(|(a, b)| *a || b)
                .collect();
        }

        out
    }

    pub fn get_node(&self, block: &IrBlock) -> Option<&ControlFlowNode> {
        let index = self.block_index_by_address.get(&block.start_address)?;
        self.nodes.get(*index)
    }

    pub fn has_reachable_exit(&self, node_index: usize) -> bool {
        self.node_has_reachable_exit
            .get(node_index)
            .copied()
            .unwrap_or(false)
    }

    pub fn has_direct_exit_out_of_container(&self, node_index: usize) -> bool {
        self.node_has_direct_exit_out_of_container
            .get(node_index)
            .copied()
            .unwrap_or(false)
    }

    fn initialize_nodes(&mut self, container: &IrBlockContainer) {
        for (index, block) in container.blocks.iter().enumerate() {
            self.nodes
                .push(ControlFlowNode::new(index, Some(block.start_address)));
            self.block_index_by_address
                .insert(block.start_address, index);
        }

        self.node_has_direct_exit_out_of_container = vec![false; self.nodes.len()];
        self.node_has_reachable_exit = vec![false; self.nodes.len()];
    }

    fn create_edges(&mut self, container: &IrBlockContainer) {
        let nested_descendant_blocks = collect_nested_descendant_blocks(container);

        for (index, block) in container.blocks.iter().enumerate() {
            let descendant_container_ids = collect_descendant_container_ids(block);
            for statement in &block.statements {
                self.process_statement(
                    statement,
                    index,
                    &nested_descendant_blocks,
                    &descendant_container_ids,
                );
            }
            self.add_fallthrough_edge_if_needed(block, index);
        }
    }

    fn add_fallthrough_edge_if_needed(&mut self, block: &IrBlock, block_index: usize) {
        if block_has_unreachable_endpoint(block) {
            return;
        }

        let next_index = block_index + 1;
        if next_index < self.nodes.len() {
            self.connect_nodes(block_index, next_index);
            return;
        }

        self.node_has_direct_exit_out_of_container[block_index] = true;
    }

    fn process_statement(
        &mut self,
        statement: &IrStatement,
        source_node_index: usize,
        nested_descendant_blocks: &HashSet<u32>,
        descendant_container_ids: &HashSet<u32>,
    ) {
        match statement {
            IrStatement::Jump(IrJump { target_address }) => {
                self.process_branch_target(
                    *target_address,
                    source_node_index,
                    nested_descendant_blocks,
                );
            }
            IrStatement::If(IrIf {
                true_statement,
                false_statement,
                ..
            }) => {
                self.process_statement(
                    true_statement,
                    source_node_index,
                    nested_descendant_blocks,
                    descendant_container_ids,
                );
                if let Some(false_stmt) = false_statement.as_ref() {
                    self.process_statement(
                        false_stmt,
                        source_node_index,
                        nested_descendant_blocks,
                        descendant_container_ids,
                    );
                }
            }
            IrStatement::Switch(IrSwitch {
                cases,
                default_target,
                ..
            }) => {
                for target in cases.values() {
                    self.process_branch_target(
                        *target,
                        source_node_index,
                        nested_descendant_blocks,
                    );
                }
                if let Some(target) = default_target {
                    self.process_branch_target(
                        *target,
                        source_node_index,
                        nested_descendant_blocks,
                    );
                }
            }
            IrStatement::Leave(IrLeave {
                target_container_id,
            }) => {
                if self.is_leave_out_of_container(*target_container_id, descendant_container_ids) {
                    self.node_has_direct_exit_out_of_container[source_node_index] = true;
                }
            }
            IrStatement::BlockContainer(container) => {
                for nested in &container.blocks {
                    for nested_statement in &nested.statements {
                        self.process_statement(
                            nested_statement,
                            source_node_index,
                            nested_descendant_blocks,
                            descendant_container_ids,
                        );
                    }
                }
            }
            IrStatement::Block(block) => {
                for nested_statement in &block.statements {
                    self.process_statement(
                        nested_statement,
                        source_node_index,
                        nested_descendant_blocks,
                        descendant_container_ids,
                    );
                }
            }
            _ => {}
        }
    }

    fn process_branch_target(
        &mut self,
        target_address: u32,
        source_node_index: usize,
        nested_descendant_blocks: &HashSet<u32>,
    ) {
        if let Some(target_index) = self.block_index_by_address.get(&target_address).copied() {
            self.connect_nodes(source_node_index, target_index);
            return;
        }

        if nested_descendant_blocks.contains(&target_address) {
            return;
        }

        self.node_has_direct_exit_out_of_container[source_node_index] = true;
    }

    fn is_leave_out_of_container(
        &self,
        leave_target_container_id: u32,
        descendant_container_ids: &HashSet<u32>,
    ) -> bool {
        if leave_target_container_id == self.function_body_container_id {
            return false;
        }
        if descendant_container_ids.contains(&leave_target_container_id) {
            return false;
        }
        true
    }

    fn connect_nodes(&mut self, source_index: usize, target_index: usize) {
        if !self.nodes[source_index].successors.contains(&target_index) {
            self.nodes[source_index].successors.push(target_index);
        }
        if !self.nodes[target_index]
            .predecessors
            .contains(&source_index)
        {
            self.nodes[target_index].predecessors.push(source_index);
        }
    }

    fn find_nodes_with_exits_out_of_container(&self) -> Vec<bool> {
        let mut leaving = vec![false; self.nodes.len()];
        for node in &self.nodes {
            if leaving[node.user_index]
                || !self.node_has_direct_exit_out_of_container[node.user_index]
            {
                continue;
            }

            let mut current = Some(node.user_index);
            while let Some(index) = current {
                if leaving[index] {
                    break;
                }
                leaving[index] = true;
                current = self.nodes[index].immediate_dominator;
            }
        }
        leaving
    }
}

fn block_has_unreachable_endpoint(block: &IrBlock) -> bool {
    let Some(last) = block.statements.last() else {
        return false;
    };
    statement_has_unreachable_endpoint(last)
}

fn statement_has_unreachable_endpoint(statement: &IrStatement) -> bool {
    match statement {
        IrStatement::Jump(_) | IrStatement::Leave(_) | IrStatement::Return(IrReturn) => true,
        IrStatement::Switch(IrSwitch { .. }) => true,
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            let Some(false_stmt) = false_statement else {
                return false;
            };
            statement_has_unreachable_endpoint(true_statement)
                && statement_has_unreachable_endpoint(false_stmt)
        }
        IrStatement::Block(block) => block
            .statements
            .last()
            .is_some_and(statement_has_unreachable_endpoint),
        IrStatement::HighLevelSwitch(IrHighLevelSwitch { .. }) => true,
        IrStatement::HighLevelWhile(IrHighLevelWhile { .. }) => false,
        IrStatement::HighLevelDoWhile(IrHighLevelDoWhile { .. }) => false,
        _ => false,
    }
}

fn collect_nested_descendant_blocks(container: &IrBlockContainer) -> HashSet<u32> {
    let mut descendants = HashSet::<u32>::new();
    let mut visited_containers = HashSet::<u32>::new();

    fn visit_statement(
        statement: &IrStatement,
        descendants: &mut HashSet<u32>,
        visited_containers: &mut HashSet<u32>,
    ) {
        match statement {
            IrStatement::BlockContainer(nested) => {
                visit_container(nested, descendants, visited_containers)
            }
            IrStatement::If(IrIf {
                true_statement,
                false_statement,
                ..
            }) => {
                visit_statement(true_statement, descendants, visited_containers);
                if let Some(false_stmt) = false_statement.as_ref() {
                    visit_statement(false_stmt, descendants, visited_containers);
                }
            }
            IrStatement::Block(block) => {
                for child in &block.statements {
                    visit_statement(child, descendants, visited_containers);
                }
            }
            _ => {}
        }
    }

    fn visit_container(
        nested: &IrBlockContainer,
        descendants: &mut HashSet<u32>,
        visited_containers: &mut HashSet<u32>,
    ) {
        if !visited_containers.insert(nested.id) {
            return;
        }
        for nested_block in &nested.blocks {
            descendants.insert(nested_block.start_address);
            for statement in &nested_block.statements {
                visit_statement(statement, descendants, visited_containers);
            }
        }
    }

    for block in &container.blocks {
        for statement in &block.statements {
            visit_statement(statement, &mut descendants, &mut visited_containers);
        }
    }

    descendants
}

fn collect_descendant_container_ids(block: &IrBlock) -> HashSet<u32> {
    let mut result = HashSet::<u32>::new();
    let mut visited_containers = HashSet::<u32>::new();

    fn visit_statement(
        statement: &IrStatement,
        result: &mut HashSet<u32>,
        visited_containers: &mut HashSet<u32>,
    ) {
        match statement {
            IrStatement::BlockContainer(nested) => {
                if !visited_containers.insert(nested.id) {
                    return;
                }
                result.insert(nested.id);
                for nested_block in &nested.blocks {
                    for nested_statement in &nested_block.statements {
                        visit_statement(nested_statement, result, visited_containers);
                    }
                }
            }
            IrStatement::If(IrIf {
                true_statement,
                false_statement,
                ..
            }) => {
                visit_statement(true_statement, result, visited_containers);
                if let Some(false_stmt) = false_statement.as_ref() {
                    visit_statement(false_stmt, result, visited_containers);
                }
            }
            IrStatement::Block(inner_block) => {
                for inner in &inner_block.statements {
                    visit_statement(inner, result, visited_containers);
                }
            }
            _ => {}
        }
    }

    for statement in &block.statements {
        visit_statement(statement, &mut result, &mut visited_containers);
    }

    result
}
