use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Parser;
use serde_json::{Value, json};
use udon_decompiler::decompiler::Result as DcResult;
use udon_decompiler::decompiler::{
    BasicBlockCollection, DecompileContext, ProgramTransformContext, UdonModuleInfo,
    VariableRecord, VariableTable, build_cfgs_and_discover_entries, build_default_pipeline,
    build_ir_functions,
};

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long)]
    case: PathBuf,
    #[arg(long, default_value = "UdonModuleInfo.json")]
    module_info: PathBuf,
    #[arg(long)]
    output: Option<PathBuf>,
}

const IL_PASS_NAMES: &[&str] = &[
    "ControlFlowSimplification",
    "ConstToLiteral",
    "TempVariableInline",
    "DetectExitPoints(false)",
    "LoopDetection",
    "DetectExitPoints(true)",
    "ConditionDetection",
    "HighLevelLoopTransform",
    "HighLevelSwitchTransform",
    "HighLevelLoopStatementTransform",
    "StructuredControlFlowCleanupTransform",
    "CollectLabelUsage",
    "CollectVariables",
];

fn main() -> Result<()> {
    let cli = Cli::parse();
    UdonModuleInfo::set_default_module_info_path(cli.module_info.clone())
        .map_err(|e| anyhow::anyhow!("failed to configure module info path: {}", e))?;

    let hex_text = load_code_fence(&cli.case, "hex")?;
    let mut ctx = DecompileContext::from_compressed_hex_text(
        &hex_text,
        cli.case
            .file_name()
            .and_then(|x| x.to_str())
            .map(|x| x.to_string()),
    )
    .map_err(|e| anyhow::anyhow!("failed to load decompile context: {}", e))?;

    run_pre_ir_steps(&mut ctx).map_err(|e| anyhow::anyhow!("pipeline setup failed: {}", e))?;

    let mut functions = build_ir_functions(&ctx);
    let mut stages = Vec::<Value>::new();
    stages.push(json!({
        "stage": "InitialIr",
        "functions": dump_functions(&functions, &ctx),
    }));

    let class_name = ctx.infer_class_name_for_output();
    let mut transform_context = ProgramTransformContext::new(class_name, &ctx);
    let pipeline = build_default_pipeline();
    if pipeline.il_transforms.len() != IL_PASS_NAMES.len() {
        bail!(
            "unexpected il transform count: expected {}, got {}",
            IL_PASS_NAMES.len(),
            pipeline.il_transforms.len()
        );
    }

    for (transform, pass_name) in pipeline.il_transforms.iter().zip(IL_PASS_NAMES.iter()) {
        for function in &mut functions {
            let function_name = function.function_name.clone();
            let mut il_context = transform_context.create_il_context(function_name);
            transform
                .run(function, &mut il_context)
                .map_err(|e| anyhow::anyhow!("transform {pass_name} failed: {}", e))?;
        }
        stages.push(json!({
            "stage": pass_name,
            "functions": dump_functions(&functions, &ctx),
        }));
    }

    let out = serde_json::to_string_pretty(&json!({
        "case": cli.case.display().to_string(),
        "stages": stages,
    }))?;

    if let Some(output) = cli.output {
        fs::write(&output, out).with_context(|| format!("failed to write {}", output.display()))?;
    } else {
        println!("{out}");
    }

    Ok(())
}

fn run_pre_ir_steps(ctx: &mut DecompileContext) -> DcResult<()> {
    ctx.variables = VariableTable::identify_from_heap(&ctx.heap_entries, &ctx.symbols);
    ctx.rebuild_symbol_address_maps_from_variables();

    ctx.basic_blocks = BasicBlockCollection::identify_from_context(ctx);
    ctx.rebuild_basic_block_address_map();

    let cfg_output = build_cfgs_and_discover_entries(ctx)?;
    ctx.cfg_functions = cfg_output.functions;
    ctx.stack_simulation = cfg_output.stack_simulation;
    Ok(())
}

fn load_code_fence(path: &Path, lang: &str) -> Result<String> {
    let text =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let parts = text.split("```").collect::<Vec<_>>();
    for idx in (1..parts.len()).step_by(2) {
        let raw = parts[idx];
        let mut lines = raw.lines();
        let block_lang = lines.next().unwrap_or_default().trim();
        if block_lang == lang {
            return Ok(format!("{}\n", lines.collect::<Vec<_>>().join("\n")));
        }
    }
    bail!("missing ```{lang} fence in {}", path.display())
}

fn dump_functions(
    functions: &[udon_decompiler::decompiler::IrFunction],
    ctx: &DecompileContext,
) -> Value {
    let mut functions = functions.iter().collect::<Vec<_>>();
    functions.sort_by_key(|function| (function.entry_address, function.function_name.as_str()));
    Value::Array(
        functions
            .into_iter()
            .map(|function| dump_function(function, ctx))
            .collect(),
    )
}

fn dump_function(
    function: &udon_decompiler::decompiler::IrFunction,
    ctx: &DecompileContext,
) -> Value {
    let container_index = ContainerIndex::for_function(function);
    json!({
        "function_name": function.function_name,
        "is_public": function.is_function_public,
        "entry_address": function.entry_address,
        "variable_declarations": function
            .variable_declarations
            .iter()
            .map(|decl| dump_variable_declaration(decl, ctx))
            .collect::<Vec<_>>(),
        "body": dump_container(&function.body, ctx, &container_index),
    })
}

fn dump_variable_declaration(
    decl: &udon_decompiler::decompiler::IrVariableDeclarationStatement,
    ctx: &DecompileContext,
) -> Value {
    let variable = require_variable(ctx, decl.variable_address);
    json!({
        "address": variable.address,
        "name": variable.name,
        "type": variable.type_name,
        "scope": format!("{:?}", variable.scope),
        "init": decl.init_value.as_ref().map(dump_literal),
    })
}

fn dump_container(
    container: &udon_decompiler::decompiler::IrBlockContainer,
    ctx: &DecompileContext,
    container_index: &ContainerIndex<'_>,
) -> Value {
    json!({
        "kind": format!("{:?}", container.kind),
        "entry_address": container.entry_block().map(|block| block.start_address),
        "should_emit_exit_label": container.should_emit_exit_label,
        "blocks": container
            .blocks
            .iter()
            .map(|block| dump_block(block, ctx, container_index))
            .collect::<Vec<_>>(),
    })
}

fn dump_block(
    block: &udon_decompiler::decompiler::IrBlock,
    ctx: &DecompileContext,
    container_index: &ContainerIndex<'_>,
) -> Value {
    json!({
        "start_address": block.start_address,
        "should_emit_label": block.should_emit_label,
        "statements": block
            .statements
            .iter()
            .map(|stmt| dump_statement(stmt, ctx, container_index))
            .collect::<Vec<_>>(),
    })
}

fn dump_statement(
    statement: &udon_decompiler::decompiler::IrStatement,
    ctx: &DecompileContext,
    container_index: &ContainerIndex<'_>,
) -> Value {
    use udon_decompiler::decompiler::IrStatement;

    match statement {
        IrStatement::Assignment(assign) => {
            json!({
                "kind": "Assignment",
                "target": dump_expression(&assign.target, ctx),
                "value": dump_expression(&assign.value, ctx),
            })
        }
        IrStatement::Expression(expr) => json!({
            "kind": "Expression",
            "expression": dump_expression(&expr.expression, ctx),
        }),
        IrStatement::VariableDeclaration(decl) => json!({
            "kind": "VariableDeclaration",
            "declaration": dump_variable_declaration(decl, ctx),
        }),
        IrStatement::Block(block) => json!({
            "kind": "Block",
            "block": dump_block(block, ctx, container_index),
        }),
        IrStatement::BlockContainer(container) => json!({
            "kind": "BlockContainer",
            "container": dump_container(container, ctx, container_index),
        }),
        IrStatement::If(stmt) => json!({
            "kind": "If",
            "condition": dump_expression(&stmt.condition, ctx),
            "true": dump_statement(stmt.true_statement.as_ref(), ctx, container_index),
            "false": stmt.false_statement.as_ref().map(|stmt| dump_statement(stmt, ctx, container_index)),
        }),
        IrStatement::Jump(jump) => json!({
            "kind": "Jump",
            "target_address": jump.target_address,
        }),
        IrStatement::Leave(leave) => json!({
            "kind": "Leave",
            "target": container_index.describe(leave.target_container_id),
        }),
        IrStatement::Return(_) => json!({ "kind": "Return" }),
        IrStatement::Switch(stmt) => json!({
            "kind": "Switch",
            "index_expression": dump_expression(&stmt.index_expression, ctx),
            "cases": stmt.cases.iter().map(|(label, address)| {
                json!({"label": label, "target_address": address})
            }).collect::<Vec<_>>(),
            "default_target": stmt.default_target,
        }),
        IrStatement::HighLevelSwitch(stmt) => json!({
            "kind": "HighLevelSwitch",
            "index_expression": dump_expression(&stmt.index_expression, ctx),
            "sections": stmt.sections.iter().map(|section| {
                json!({
                    "labels": section.labels,
                    "is_default": section.is_default,
                    "body": dump_block(&section.body, ctx, container_index),
                })
            }).collect::<Vec<_>>(),
        }),
        IrStatement::HighLevelWhile(stmt) => json!({
            "kind": "HighLevelWhile",
            "condition": stmt.condition.as_ref().map(|expr| dump_expression(expr, ctx)),
            "continue_target": stmt.continue_target,
            "break_target": container_index.describe_loop_break(
                stmt.break_target,
                if stmt.condition.is_some() { "While" } else { "Loop" },
                stmt.continue_target,
            ),
            "body": dump_container(&stmt.body, ctx, container_index),
        }),
        IrStatement::HighLevelDoWhile(stmt) => json!({
            "kind": "HighLevelDoWhile",
            "condition": dump_expression(&stmt.condition, ctx),
            "continue_target": stmt.continue_target,
            "break_target": container_index.describe_loop_break(
                stmt.break_target,
                "DoWhile",
                stmt.body.entry_block().map(|block| block.start_address).unwrap_or(stmt.continue_target),
            ),
            "body": dump_container(&stmt.body, ctx, container_index),
        }),
    }
}

fn dump_expression(
    expression: &udon_decompiler::decompiler::IrExpression,
    ctx: &DecompileContext,
) -> Value {
    use udon_decompiler::decompiler::IrExpression;

    match expression {
        IrExpression::Literal(literal) => json!({
            "kind": "Literal",
            "literal": dump_literal(literal),
        }),
        IrExpression::Variable(variable) => {
            let record = require_variable(ctx, variable.address);
            json!({
                "kind": "Variable",
                "address": record.address,
                "name": record.name,
                "type": record.type_name,
                "scope": format!("{:?}", record.scope),
            })
        }
        IrExpression::Raw(raw) => json!({
            "kind": "Raw",
            "value": raw.value,
        }),
        IrExpression::InternalCall(call) => json!({
            "kind": "InternalCall",
            "function_name": call.function_name,
            "entry_address": call.entry_address,
            "call_jump_target": call.call_jump_target,
        }),
        IrExpression::ExternalCall(call) => json!({
            "kind": "ExternalCall",
            "signature": call.signature,
            "type_name": call.function_info.type_name,
            "function_name": call.function_info.function_name,
            "arguments": call.arguments.iter().map(|arg| dump_expression(arg, ctx)).collect::<Vec<_>>(),
        }),
        IrExpression::PropertyAccess(call) => json!({
            "kind": "PropertyAccess",
            "signature": call.signature,
            "type_name": call.function_info.type_name,
            "function_name": call.function_info.function_name,
            "arguments": call.arguments.iter().map(|arg| dump_expression(arg, ctx)).collect::<Vec<_>>(),
        }),
        IrExpression::ConstructorCall(call) => json!({
            "kind": "ConstructorCall",
            "signature": call.signature,
            "type_name": call.function_info.type_name,
            "function_name": call.function_info.function_name,
            "arguments": call.arguments.iter().map(|arg| dump_expression(arg, ctx)).collect::<Vec<_>>(),
        }),
        IrExpression::OperatorCall(call) => json!({
            "kind": "OperatorCall",
            "operator": format!("{:?}", call.operator),
            "arguments": call.arguments.iter().map(|arg| dump_expression(arg, ctx)).collect::<Vec<_>>(),
        }),
    }
}

fn dump_literal(literal: &udon_decompiler::decompiler::IrLiteralExpression) -> Value {
    json!({
        "value": literal.value,
        "type_hint": literal.type_hint,
    })
}

fn require_variable(ctx: &DecompileContext, address: u32) -> &VariableRecord {
    ctx.variables
        .get_by_address(address)
        .unwrap_or_else(|| panic!("missing variable at address 0x{address:08x} during IR dump"))
}

struct ContainerIndex<'a> {
    by_id: HashMap<u32, &'a udon_decompiler::decompiler::IrBlockContainer>,
    detached_by_id: HashMap<u32, Value>,
}

impl<'a> ContainerIndex<'a> {
    fn for_function(function: &'a udon_decompiler::decompiler::IrFunction) -> Self {
        let mut by_id = HashMap::<u32, &'a udon_decompiler::decompiler::IrBlockContainer>::new();
        let mut detached_by_id = HashMap::<u32, Value>::new();
        collect_container_refs(&function.body, &mut by_id);
        collect_detached_break_targets(&function.body, &mut detached_by_id);
        Self {
            by_id,
            detached_by_id,
        }
    }

    fn describe(&self, container_id: u32) -> Value {
        let Some(container) = self.by_id.get(&container_id) else {
            if let Some(detached) = self.detached_by_id.get(&container_id) {
                return detached.clone();
            }
            return json!({
                "kind": null,
                "entry_address": null,
            });
        };
        json!({
            "kind": format!("{:?}", container.kind),
            "entry_address": container.entry_block().map(|block| block.start_address),
        })
    }

    fn describe_loop_break(
        &self,
        container_id: u32,
        fallback_kind: &str,
        fallback_entry_address: u32,
    ) -> Value {
        let described = self.describe(container_id);
        if !described["kind"].is_null() {
            return described;
        }
        json!({
            "kind": fallback_kind,
            "entry_address": fallback_entry_address,
        })
    }
}

fn collect_container_refs<'a>(
    container: &'a udon_decompiler::decompiler::IrBlockContainer,
    out: &mut HashMap<u32, &'a udon_decompiler::decompiler::IrBlockContainer>,
) {
    out.insert(container.id, container);
    for block in &container.blocks {
        for statement in &block.statements {
            collect_statement_container_refs(statement, out);
        }
    }
}

fn collect_statement_container_refs<'a>(
    statement: &'a udon_decompiler::decompiler::IrStatement,
    out: &mut HashMap<u32, &'a udon_decompiler::decompiler::IrBlockContainer>,
) {
    use udon_decompiler::decompiler::IrStatement;

    match statement {
        IrStatement::Block(block) => {
            for nested in &block.statements {
                collect_statement_container_refs(nested, out);
            }
        }
        IrStatement::BlockContainer(container) => collect_container_refs(container, out),
        IrStatement::If(stmt) => {
            collect_statement_container_refs(stmt.true_statement.as_ref(), out);
            if let Some(false_stmt) = stmt.false_statement.as_ref() {
                collect_statement_container_refs(false_stmt, out);
            }
        }
        IrStatement::HighLevelSwitch(stmt) => {
            for section in &stmt.sections {
                for nested in &section.body.statements {
                    collect_statement_container_refs(nested, out);
                }
            }
        }
        IrStatement::HighLevelWhile(stmt) => collect_container_refs(&stmt.body, out),
        IrStatement::HighLevelDoWhile(stmt) => collect_container_refs(&stmt.body, out),
        IrStatement::Assignment(_)
        | IrStatement::Expression(_)
        | IrStatement::VariableDeclaration(_)
        | IrStatement::Jump(_)
        | IrStatement::Leave(_)
        | IrStatement::Return(_)
        | IrStatement::Switch(_) => {}
    }
}

fn collect_detached_break_targets(
    container: &udon_decompiler::decompiler::IrBlockContainer,
    out: &mut HashMap<u32, Value>,
) {
    for block in &container.blocks {
        for statement in &block.statements {
            collect_statement_detached_break_targets(statement, out);
        }
    }
}

fn collect_statement_detached_break_targets(
    statement: &udon_decompiler::decompiler::IrStatement,
    out: &mut HashMap<u32, Value>,
) {
    use udon_decompiler::decompiler::IrStatement;

    match statement {
        IrStatement::Block(block) => {
            for nested in &block.statements {
                collect_statement_detached_break_targets(nested, out);
            }
        }
        IrStatement::BlockContainer(container) => collect_detached_break_targets(container, out),
        IrStatement::If(stmt) => {
            collect_statement_detached_break_targets(stmt.true_statement.as_ref(), out);
            if let Some(false_stmt) = stmt.false_statement.as_ref() {
                collect_statement_detached_break_targets(false_stmt, out);
            }
        }
        IrStatement::HighLevelSwitch(stmt) => {
            for section in &stmt.sections {
                for nested in &section.body.statements {
                    collect_statement_detached_break_targets(nested, out);
                }
            }
        }
        IrStatement::HighLevelWhile(stmt) => {
            out.entry(stmt.break_target).or_insert_with(|| {
                json!({
                    "kind": if stmt.condition.is_some() { "While" } else { "Loop" },
                    "entry_address": stmt.continue_target,
                })
            });
            collect_detached_break_targets(&stmt.body, out);
        }
        IrStatement::HighLevelDoWhile(stmt) => {
            out.entry(stmt.break_target).or_insert_with(|| {
                json!({
                    "kind": "DoWhile",
                    "entry_address": stmt.body.entry_block().map(|block| block.start_address),
                })
            });
            collect_detached_break_targets(&stmt.body, out);
        }
        IrStatement::Assignment(_)
        | IrStatement::Expression(_)
        | IrStatement::VariableDeclaration(_)
        | IrStatement::Jump(_)
        | IrStatement::Leave(_)
        | IrStatement::Return(_)
        | IrStatement::Switch(_) => {}
    }
}
