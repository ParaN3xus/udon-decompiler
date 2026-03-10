use super::control_flow_node::ControlFlowNode;

pub fn compute_dominance(entry_index: usize, nodes: &mut [ControlFlowNode]) {
    let post_order = collect_post_order(entry_index, nodes);
    if post_order.is_empty() {
        return;
    }

    for (index, node_index) in post_order.iter().enumerate() {
        nodes[*node_index].post_order_number = index as i32;
    }

    nodes[entry_index].immediate_dominator = Some(entry_index);
    let mut changed = true;
    while changed {
        changed = false;

        for node_index in post_order.iter().rev().skip(1) {
            let mut new_idom = None::<usize>;
            let predecessors = nodes[*node_index].predecessors.clone();
            for pred in predecessors {
                if nodes[pred].immediate_dominator.is_none() {
                    continue;
                }
                new_idom = Some(match new_idom {
                    Some(existing) => find_common_dominator(pred, existing, nodes),
                    None => pred,
                });
            }

            let Some(new_idom) = new_idom else {
                continue;
            };
            if nodes[*node_index].immediate_dominator != Some(new_idom) {
                nodes[*node_index].immediate_dominator = Some(new_idom);
                changed = true;
            }
        }
    }

    for node in nodes.iter_mut() {
        if node.immediate_dominator.is_some() {
            node.dominator_tree_children = Some(Vec::new());
        }
    }

    nodes[entry_index].immediate_dominator = None;
    let immediate = nodes
        .iter()
        .enumerate()
        .map(|(idx, node)| (idx, node.immediate_dominator))
        .collect::<Vec<_>>();

    for (node_index, idom) in immediate {
        if let Some(parent) = idom
            && let Some(children) = nodes[parent].dominator_tree_children.as_mut()
        {
            children.push(node_index);
        }
        nodes[node_index].visited = false;
    }
}

pub fn mark_nodes_with_reachable_exits(cfg_nodes: &[ControlFlowNode]) -> Vec<bool> {
    let mut result = vec![false; cfg_nodes.len()];

    for join_node in cfg_nodes {
        if !join_node.is_reachable() {
            continue;
        }

        let has_multiple_inputs = join_node.predecessors.len() >= 2;
        let has_root_extra_input =
            !join_node.predecessors.is_empty() && join_node.immediate_dominator.is_none();
        if !has_multiple_inputs && !has_root_extra_input {
            continue;
        }

        for pred in &join_node.predecessors {
            let mut runner = Some(*pred);
            while let Some(current) = runner {
                if Some(current) == join_node.immediate_dominator || current == join_node.user_index
                {
                    break;
                }
                result[current] = true;
                runner = cfg_nodes[current].immediate_dominator;
            }
        }
    }

    result
}

fn collect_post_order(entry_index: usize, nodes: &mut [ControlFlowNode]) -> Vec<usize> {
    for node in nodes.iter_mut() {
        node.visited = false;
    }

    let mut out = Vec::<usize>::new();
    dfs_post_order(entry_index, nodes, &mut out);
    out
}

fn dfs_post_order(index: usize, nodes: &mut [ControlFlowNode], out: &mut Vec<usize>) {
    if nodes[index].visited {
        return;
    }
    nodes[index].visited = true;
    let successors = nodes[index].successors.clone();
    for succ in successors {
        dfs_post_order(succ, nodes, out);
    }
    out.push(index);
}

fn find_common_dominator(mut a: usize, mut b: usize, nodes: &[ControlFlowNode]) -> usize {
    while a != b {
        while nodes[a].post_order_number < nodes[b].post_order_number {
            a = nodes[a]
                .immediate_dominator
                .expect("dominator tree invariant broken: missing idom for a");
        }
        while nodes[b].post_order_number < nodes[a].post_order_number {
            b = nodes[b]
                .immediate_dominator
                .expect("dominator tree invariant broken: missing idom for b");
        }
    }
    a
}
