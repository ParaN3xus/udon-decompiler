from src import ProgramLoader, CFGBuilder, BytecodeParser


program = ProgramLoader.load_from_file("./local/ex.json")
bc_parser = BytecodeParser(program)

instructions = bc_parser.parse()

cfg_builder = CFGBuilder(program, instructions)

cfgs = cfg_builder.build()

for (func, cfg) in cfgs.items():
    cfg.to_dot().write(path=f"local/{func}.png", format="png")
