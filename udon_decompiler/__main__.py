import argparse
import logging
import sys
from pathlib import Path
from typing import Optional

from udon_decompiler import (
    BytecodeParser,
    DataFlowAnalyzer,
    ModuleInfoLoader,
    ProgramCodeGenerator,
    ProgramLoader,
    UdonModuleInfo,
    UdonProgramData,
    logger,
)
from udon_decompiler.utils.logger import set_logger_level


def decompile_program_to_source(
    program: UdonProgramData, code_gen: ProgramCodeGenerator
) -> tuple[Optional[str], str]:
    bc_parser = BytecodeParser(program)
    instructions = bc_parser.parse()

    logger.debug(f"ASM: {instructions}")

    analyzer = DataFlowAnalyzer(program, UdonModuleInfo(), instructions)
    function_analyzers = analyzer.analyze()

    code_gen = ProgramCodeGenerator()
    class_name, code = code_gen.generate_program(program, function_analyzers)
    return class_name, code


def process_file(
    json_file: Path,
    output_target: Path,
    is_target_file: bool,
    code_gen: ProgramCodeGenerator,
):
    try:
        program = ProgramLoader.load_from_file(str(json_file))

        class_name, source_code = decompile_program_to_source(program, code_gen)

        if is_target_file:
            final_path = output_target
            final_path.parent.mkdir(parents=True, exist_ok=True)
        else:
            output_target.mkdir(parents=True, exist_ok=True)
            final_path = (
                output_target / f"{class_name if class_name else json_file.stem}.cs"
            )

        with open(final_path, "w", encoding="utf-8") as f:
            f.write(source_code)

        logger.info(f"Decompiled: {json_file.name} -> {final_path}")

    except Exception as e:
        logger.error(f"Failed to decompile {json_file.name}: {str(e)}")
        raise e


def main():
    parser = argparse.ArgumentParser(description="UdonSharp Decompiler CLI")
    parser.add_argument("input", type=Path, help="Input .json file or directory")
    parser.add_argument("-o", "--output", type=Path, help="Output path")
    parser.add_argument(
        "--info",
        type=Path,
        default="./local/UdonModuleInfo.json",
        help="Path to UdonModuleInfo.json",
    )
    parser.add_argument(
        "--log",
        default="INFO",
        choices=["DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL"],
        help="Set log level (default: INFO)",
    )
    args = parser.parse_args()

    set_logger_level(getattr(logging, args.log.upper(), logging.INFO))

    if not args.info.exists():
        logger.error(f"Module info file not found at {args.info}")
        sys.exit(1)

    try:
        ModuleInfoLoader.load_from_file(str(args.info))
    except Exception as e:
        logger.error(f"Error loading ModuleInfo: {e}")
        raise e

    input_path: Path = args.input
    output_path: Path = args.output

    if not input_path.exists():
        logger.error(f"Input path '{input_path}' does not exist.")
        sys.exit(1)

    code_gen = ProgramCodeGenerator()

    if input_path.is_file():
        if input_path.suffix.lower() != ".json":
            logger.error("Input file must be a .json file.")
            sys.exit(1)

        if output_path is None:
            target = input_path.parent
            is_file = False
        else:
            if output_path.suffix.lower() == ".cs":
                target = output_path
                is_file = True
            else:
                target = output_path
                is_file = False

        process_file(input_path, target, is_file, code_gen)

    else:
        if output_path is None:
            target = input_path.parent / f"{input_path.name}-decompiled"
        else:
            target = output_path

        json_files = list(input_path.glob("*.json"))
        if not json_files:
            logger.warning("No .json files found in the directory.")
            return

        for json_file in json_files:
            if json_file.name == "UdonModuleInfo.json":
                continue
            process_file(json_file, target, is_target_file=False, code_gen=code_gen)

    logger.info("Done.")


if __name__ == "__main__":
    main()
