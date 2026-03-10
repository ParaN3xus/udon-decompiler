#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlFlowNode {
    pub user_index: usize,
    pub block_start_address: Option<u32>,

    pub visited: bool,
    pub post_order_number: i32,

    pub immediate_dominator: Option<usize>,
    pub dominator_tree_children: Option<Vec<usize>>,

    pub predecessors: Vec<usize>,
    pub successors: Vec<usize>,
}

impl ControlFlowNode {
    pub fn new(user_index: usize, block_start_address: Option<u32>) -> Self {
        Self {
            user_index,
            block_start_address,
            visited: false,
            post_order_number: -1,
            immediate_dominator: None,
            dominator_tree_children: None,
            predecessors: Vec::new(),
            successors: Vec::new(),
        }
    }

    pub fn is_reachable(&self) -> bool {
        self.dominator_tree_children.is_some()
    }
}
