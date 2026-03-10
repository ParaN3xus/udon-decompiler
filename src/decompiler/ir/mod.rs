mod builder;
mod control_flow_graph;
mod control_flow_node;
mod dominance;
mod nodes;

pub use builder::{IrBuilder, build_ir_functions};
pub use control_flow_graph::ControlFlowGraph;
pub use control_flow_node::ControlFlowNode;
pub use dominance::{compute_dominance, mark_nodes_with_reachable_exits};
pub use nodes::{
    IrAssignmentStatement, IrBlock, IrBlockContainer, IrClass, IrConstructorCallExpression,
    IrContainerKind, IrExpression, IrExpressionStatement, IrExternalCallExpression, IrFunction,
    IrHighLevelDoWhile, IrHighLevelSwitch, IrHighLevelSwitchSection, IrHighLevelWhile, IrIf,
    IrInternalCallExpression, IrJump, IrLeave, IrLiteralExpression, IrOperator,
    IrOperatorCallExpression, IrPropertyAccessExpression, IrReturn, IrStatement, IrSwitch,
    IrVariableDeclarationStatement, IrVariableExpression,
};
