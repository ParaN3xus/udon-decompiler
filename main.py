from pathlib import Path
from udon_decompiler import ProgramLoader,  BytecodeParser, ModuleInfoLoader, UdonModuleInfo, DataFlowAnalyzer, ProgramCodeGenerator

ModuleInfoLoader.load_from_file("./local/UdonModuleInfo.json")

program = ProgramLoader.load_from_file("./local/ex.json")
bc_parser = BytecodeParser(program)

instructions = bc_parser.parse()

analyzer = DataFlowAnalyzer(program, UdonModuleInfo(), instructions)
function_analyzers = analyzer.analyze()

code_gen = ProgramCodeGenerator()
generated_code = code_gen.generate_program(
    function_analyzers,
    class_name="Test"
)

output_path = Path("local/ex.cs")

output_path.parent.mkdir(parents=True, exist_ok=True)

with open(output_path, 'w', encoding='utf-8') as f:
    f.write(generated_code)
