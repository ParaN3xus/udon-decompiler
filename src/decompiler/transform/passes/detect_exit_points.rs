use std::collections::{HashMap, HashSet};

use crate::decompiler::Result;
use crate::decompiler::ir::{
    IrBlock, IrBlockContainer, IrFunction, IrHighLevelDoWhile, IrHighLevelSwitch, IrHighLevelWhile,
    IrIf, IrJump, IrLeave, IrStatement,
};
use crate::decompiler::transform::pass_base::{ITransform, TransformContext};

#[derive(Debug, Clone)]
enum CurrentExit {
    NotYetDetermined,
    NoExit,
    Exit(Box<IrStatement>),
}

#[derive(Debug, Clone)]
struct ExitOccurrence {
    statement: IrStatement,
    slot: *mut IrStatement,
}

pub struct DetectExitPoints {
    pub can_introduce_exit_for_return: bool,
}

impl DetectExitPoints {
    pub fn new(can_introduce_exit_for_return: bool) -> Self {
        Self {
            can_introduce_exit_for_return,
        }
    }

    pub fn compatible_exit_instruction(a: &IrStatement, b: &IrStatement) -> bool {
        match (a, b) {
            (
                IrStatement::Jump(IrJump { target_address: ta }),
                IrStatement::Jump(IrJump { target_address: tb }),
            ) => ta == tb,
            (
                IrStatement::Leave(IrLeave {
                    target_container_id: ta,
                }),
                IrStatement::Leave(IrLeave {
                    target_container_id: tb,
                }),
            ) => ta == tb,
            _ => false,
        }
    }
}

impl ITransform for DetectExitPoints {

    fn run(&self, function: &mut IrFunction, _context: &mut TransformContext<'_, '_>) -> Result<()> {
        let mut state = DetectState::new(function.body.id, self.can_introduce_exit_for_return);
        state.visit_container(&mut function.body, None, None);
        Ok(())
    }
}

struct DetectState {
    function_body_id: u32,
    can_introduce_exit_for_return: bool,

    current_container_id: Option<u32>,
    current_exit: CurrentExit,
    potential_exits: Option<Vec<ExitOccurrence>>,

    descendant_block_cache: HashMap<u32, HashSet<u32>>,
    self_leave_cache: HashMap<u32, bool>,
}

impl DetectState {
    fn new(function_body_id: u32, can_introduce_exit_for_return: bool) -> Self {
        Self {
            function_body_id,
            can_introduce_exit_for_return,
            current_container_id: None,
            current_exit: CurrentExit::NoExit,
            potential_exits: None,
            descendant_block_cache: HashMap::new(),
            self_leave_cache: HashMap::new(),
        }
    }

    fn visit_container(
        &mut self,
        container: &mut IrBlockContainer,
        parent_block: Option<&mut IrBlock>,
        parent_index: Option<usize>,
    ) {
        let old_exit = self.current_exit.clone();
        let old_container_id = self.current_container_id;
        let old_potential_exits = self.potential_exits.take();

        let this_exit = self.get_exit_after_statement(parent_block.as_deref(), parent_index);
        self.current_exit = this_exit;
        self.current_container_id = Some(container.id);
        let has_self_leave = statement_has_leave_target_container(container, container.id);
        self.self_leave_cache.insert(container.id, has_self_leave);
        self.potential_exits = if matches!(self.current_exit, CurrentExit::NotYetDetermined) {
            Some(Vec::new())
        } else {
            None
        };

        let descendant_blocks = self.descendant_blocks(container);

        for block in &mut container.blocks {
            self.visit_block(block, container.id, &descendant_blocks);
        }

        let should_introduce = matches!(self.current_exit, CurrentExit::NotYetDetermined)
            && self.potential_exits.as_ref().is_some_and(|x| !x.is_empty())
            && parent_block.is_some()
            && parent_index.is_some();

        if should_introduce {
            let chosen_exit = self.choose_exit(self.potential_exits.as_ref().expect("present"));

            if let Some(exits) = self.potential_exits.as_ref() {
                for occurrence in exits {
                    if Self::compatible_ref(&chosen_exit, &occurrence.statement) {
                        // SAFETY: pointers are collected from mutable references during traversal
                        // and are only written here before any structural mutation of the owning
                        // statement vectors in this container.
                        unsafe {
                            *occurrence.slot = IrStatement::Leave(IrLeave {
                                target_container_id: container.id,
                            });
                        }
                    }
                }
            }

            if let (Some(parent_block), Some(parent_index)) = (parent_block, parent_index) {
                let insert_at = (parent_index + 1).min(parent_block.statements.len());
                parent_block
                    .statements
                    .insert(insert_at, self.clone_exit_statement(&chosen_exit));
            }
        }

        self.current_exit = old_exit;
        self.current_container_id = old_container_id;
        self.potential_exits = old_potential_exits;
    }

    fn visit_block(
        &mut self,
        block: &mut IrBlock,
        current_container_id: u32,
        descendant_blocks: &HashSet<u32>,
    ) {
        let mut index = 0usize;
        while index < block.statements.len() {
            let statement_ptr: *mut IrStatement = &mut block.statements[index];

            // SAFETY: pointer created from exclusive borrow of block.statements[index]
            // and used only within this loop iteration.
            let statement = unsafe { &mut *statement_ptr };
            self.visit_statement(
                statement,
                current_container_id,
                descendant_blocks,
                None,
                None,
            );
            index += 1;
        }
    }

    fn visit_statement(
        &mut self,
        statement: &mut IrStatement,
        current_container_id: u32,
        descendant_blocks: &HashSet<u32>,
        parent_block: Option<&mut IrBlock>,
        parent_index: Option<usize>,
    ) {
        match statement {
            IrStatement::BlockContainer(container) => {
                self.visit_container(container, parent_block, parent_index);
            }
            IrStatement::If(IrIf {
                true_statement,
                false_statement,
                ..
            }) => {
                self.visit_statement(
                    true_statement,
                    current_container_id,
                    descendant_blocks,
                    None,
                    None,
                );
                if let Some(false_statement) = false_statement.as_mut() {
                    self.visit_statement(
                        false_statement,
                        current_container_id,
                        descendant_blocks,
                        None,
                        None,
                    );
                }
            }
            IrStatement::Jump(IrJump { target_address }) => {
                if !descendant_blocks.contains(target_address) {
                    self.handle_exit(statement, current_container_id);
                }
            }
            IrStatement::Leave(_) => {
                self.handle_exit(statement, current_container_id);
            }
            _ => {}
        }
    }

    fn handle_exit(&mut self, statement: &mut IrStatement, current_container_id: u32) {
        match &self.current_exit {
            CurrentExit::NotYetDetermined => {
                if self.can_introduce_as_exit(statement)
                    && let Some(potential_exits) = self.potential_exits.as_mut()
                {
                    potential_exits.push(ExitOccurrence {
                        statement: statement.clone(),
                        slot: statement as *mut IrStatement,
                    });
                }
            }
            CurrentExit::Exit(exit) => {
                if Self::compatible_ref(statement, exit.as_ref()) {
                    *statement = IrStatement::Leave(IrLeave {
                        target_container_id: current_container_id,
                    });
                }
            }
            CurrentExit::NoExit => {}
        }
    }

    fn can_introduce_as_exit(&self, statement: &IrStatement) -> bool {
        let Some(container_id) = self.current_container_id else {
            return false;
        };

        if self.container_has_leave_to_self(container_id) {
            return false;
        }

        match statement {
            IrStatement::Leave(IrLeave {
                target_container_id,
            }) if *target_container_id == self.function_body_id => {
                self.can_introduce_exit_for_return
            }
            IrStatement::Jump(_) | IrStatement::Leave(_) => true,
            _ => false,
        }
    }

    fn container_has_leave_to_self(&self, container_id: u32) -> bool {
        self.self_leave_cache
            .get(&container_id)
            .copied()
            .unwrap_or(false)
    }

    fn get_exit_after_statement(
        &self,
        parent_block: Option<&IrBlock>,
        parent_index: Option<usize>,
    ) -> CurrentExit {
        let (Some(parent_block), Some(parent_index)) = (parent_block, parent_index) else {
            return CurrentExit::NoExit;
        };

        let next_index = parent_index + 1;
        if next_index < parent_block.statements.len() {
            return CurrentExit::Exit(Box::new(parent_block.statements[next_index].clone()));
        }

        CurrentExit::NotYetDetermined
    }

    fn clone_exit_statement(&self, statement: &IrStatement) -> IrStatement {
        match statement {
            IrStatement::Jump(IrJump { target_address }) => IrStatement::Jump(IrJump {
                target_address: *target_address,
            }),
            IrStatement::Leave(IrLeave {
                target_container_id,
            }) => IrStatement::Leave(IrLeave {
                target_container_id: *target_container_id,
            }),
            _ => statement.clone(),
        }
    }

    fn choose_exit(&self, exits: &[ExitOccurrence]) -> IrStatement {
        let first = exits[0].statement.clone();
        if is_function_return_leave(&first, self.function_body_id) {
            for occurrence in exits.iter().skip(1) {
                if !is_function_return_leave(&occurrence.statement, self.function_body_id) {
                    return occurrence.statement.clone();
                }
            }
        }
        first
    }

    fn compatible_ref(a: &IrStatement, b: &IrStatement) -> bool {
        DetectExitPoints::compatible_exit_instruction(a, b)
    }

    fn descendant_blocks(&mut self, container: &IrBlockContainer) -> HashSet<u32> {
        if let Some(cached) = self.descendant_block_cache.get(&container.id) {
            return cached.clone();
        }

        let mut out = HashSet::<u32>::new();
        collect_descendant_block_addresses(container, &mut out);
        self.descendant_block_cache
            .insert(container.id, out.clone());
        out
    }
}

fn is_function_return_leave(statement: &IrStatement, function_body_id: u32) -> bool {
    matches!(
        statement,
        IrStatement::Leave(IrLeave { target_container_id }) if *target_container_id == function_body_id
    )
}

fn collect_descendant_block_addresses(container: &IrBlockContainer, out: &mut HashSet<u32>) {
    for block in &container.blocks {
        out.insert(block.start_address);
        for statement in &block.statements {
            collect_descendants_from_statement(statement, out);
        }
    }
}

fn collect_descendants_from_statement(statement: &IrStatement, out: &mut HashSet<u32>) {
    match statement {
        IrStatement::BlockContainer(container) => {
            collect_descendant_block_addresses(container, out)
        }
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            collect_descendants_from_statement(true_statement, out);
            if let Some(false_statement) = false_statement.as_ref() {
                collect_descendants_from_statement(false_statement, out);
            }
        }
        IrStatement::Block(block) => {
            out.insert(block.start_address);
            for nested in &block.statements {
                collect_descendants_from_statement(nested, out);
            }
        }
        IrStatement::HighLevelSwitch(IrHighLevelSwitch { sections, .. }) => {
            for section in sections {
                out.insert(section.body.start_address);
                for nested in &section.body.statements {
                    collect_descendants_from_statement(nested, out);
                }
            }
        }
        IrStatement::HighLevelWhile(IrHighLevelWhile { body, .. }) => {
            collect_descendant_block_addresses(body, out)
        }
        IrStatement::HighLevelDoWhile(IrHighLevelDoWhile { body, .. }) => {
            collect_descendant_block_addresses(body, out)
        }
        _ => {}
    }
}

fn statement_has_leave_target_container(
    container: &IrBlockContainer,
    target_container_id: u32,
) -> bool {
    for block in &container.blocks {
        for statement in &block.statements {
            if statement_has_leave_target(statement, target_container_id) {
                return true;
            }
        }
    }
    false
}

fn statement_has_leave_target(statement: &IrStatement, target_container_id: u32) -> bool {
    match statement {
        IrStatement::Leave(IrLeave {
            target_container_id: id,
        }) => *id == target_container_id,
        IrStatement::If(IrIf {
            true_statement,
            false_statement,
            ..
        }) => {
            statement_has_leave_target(true_statement, target_container_id)
                || false_statement
                    .as_ref()
                    .is_some_and(|x| statement_has_leave_target(x, target_container_id))
        }
        IrStatement::Block(block) => block
            .statements
            .iter()
            .any(|x| statement_has_leave_target(x, target_container_id)),
        IrStatement::BlockContainer(container) => {
            statement_has_leave_target_container(container, target_container_id)
        }
        IrStatement::HighLevelSwitch(IrHighLevelSwitch { sections, .. }) => {
            sections.iter().any(|section| {
                section
                    .body
                    .statements
                    .iter()
                    .any(|x| statement_has_leave_target(x, target_container_id))
            })
        }
        IrStatement::HighLevelWhile(IrHighLevelWhile { body, .. }) => {
            statement_has_leave_target_container(body, target_container_id)
        }
        IrStatement::HighLevelDoWhile(IrHighLevelDoWhile { body, .. }) => {
            statement_has_leave_target_container(body, target_container_id)
        }
        _ => false,
    }
}
