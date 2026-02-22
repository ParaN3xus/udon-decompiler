from udon_decompiler.analysis.transform.passes.class_construction import (
    IRClassConstructionTransform,
)
from udon_decompiler.analysis.transform.passes.condition_detection import (
    ConditionDetection,
)
from udon_decompiler.analysis.transform.passes.collect_variables import (
    CollectVariables,
)
from udon_decompiler.analysis.transform.passes.collect_label_usage import (
    CollectLabelUsage,
)
from udon_decompiler.analysis.transform.passes.const_to_literal import (
    ConstToLiteral,
)
from udon_decompiler.analysis.transform.passes.control_flow_simplification import (
    ControlFlowSimplification,
)
from udon_decompiler.analysis.transform.passes.detect_exit_points import (
    DetectExitPoints,
)
from udon_decompiler.analysis.transform.passes.high_level_loop_transform import (
    HighLevelLoopTransform,
)
from udon_decompiler.analysis.transform.passes.high_level_switch_transform import (
    HighLevelSwitchTransform,
)
from udon_decompiler.analysis.transform.passes.loop_detection import (
    LoopDetection,
)
from udon_decompiler.analysis.transform.passes.promote_globals import (
    PromoteGlobals,
)
from udon_decompiler.analysis.transform.passes import (
    structured_control_flow_cleanup_transform,
)
from udon_decompiler.analysis.transform.passes.temp_variable_inline import (
    TempVariableInline,
)

__all__ = [
    "IRClassConstructionTransform",
    "ConditionDetection",
    "CollectVariables",
    "CollectLabelUsage",
    "ConstToLiteral",
    "ControlFlowSimplification",
    "DetectExitPoints",
    "HighLevelLoopTransform",
    "HighLevelSwitchTransform",
    "StructuredControlFlowCleanupTransform",
    "LoopDetection",
    "PromoteGlobals",
    "TempVariableInline",
]

StructuredControlFlowCleanupTransform = (
    structured_control_flow_cleanup_transform.StructuredControlFlowCleanupTransform
)
