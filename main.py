import json
from typing import Dict, List, Tuple

class UdonDisassembler:
    OPCODES = {
        0x00: ('NOP', 0),
        0x01: ('PUSH', 1),
        0x02: ('POP', 0),
        0x04: ('JUMP_IF_FALSE', 1),
        0x05: ('JUMP', 1),
        0x06: ('EXTERN', 1),
        0x07: ('ANNOTATION', 1),
        0x08: ('JUMP_INDIRECT', 1),
        0x09: ('COPY', 0),
    }
    
    def __init__(self, json_path: str):
        with open(json_path, 'r', encoding='utf-8') as f:
            self.data = json.load(f)
        
        # 解析字节码
        hex_str = self.data['byteCodeHex']
        self.bytecode = bytes.fromhex(hex_str)
        
        # 构建地址到符号的映射
        self.addr_to_symbol = {}
        for sym_name, sym_info in self.data['symbols'].items():
            addr = sym_info['address']
            self.addr_to_symbol[addr] = sym_name
        
        self.heap = self.data['heapInitialValues']
    
    def disassemble(self) -> List[str]:
        result = []
        pc = 0
        
        while pc < len(self.bytecode):
            # 读取opcode (大端序)
            opcode = int.from_bytes(self.bytecode[pc:pc+4], 'big')
            
            if opcode not in self.OPCODES:
                result.append(f"{pc:08x}: UNKNOWN({opcode:08x})")
                pc += 4
                continue
            
            mnemonic, operand_count = self.OPCODES[opcode]
            
            if operand_count == 0:
                result.append(f"{pc:08x}: {mnemonic}")
                pc += 4
            else:
                operand = int.from_bytes(self.bytecode[pc+4:pc+8], 'big')
                operand_str = self._format_operand(opcode, operand)
                result.append(f"{pc:08x}: {mnemonic} {operand_str}")
                pc += 8
        
        return result
    
    def _format_operand(self, opcode: int, addr: int) -> str:
        # 获取符号名
        symbol = self.addr_to_symbol.get(addr, f"0x{addr:08x}")
        
        # 如果是EXTERN，显示函数签名
        if opcode == 0x06:
            heap_entry = self.heap.get(str(addr))
            if heap_entry and heap_entry['valueType'] == 'string':
                func_sig = json.loads(heap_entry['valueJson'])
                return f"{symbol} ; {func_sig}"
        
        # 如果是PUSH，显示常量值
        if opcode == 0x01:
            heap_entry = self.heap.get(str(addr))
            if heap_entry:
                value_preview = heap_entry['valueJson'][:50]
                return f"{symbol} ; {heap_entry['typeName']} = {value_preview}"
        
        return symbol

class UdonDecompiler:
    def __init__(self, disassembler: UdonDisassembler):
        self.disasm = disassembler
        self.instructions = self._parse_instructions()
    
    def _parse_instructions(self) -> List[Tuple[int, str, int]]:
        result = []
        pc = 0
        bytecode = self.disasm.bytecode
        
        while pc < len(bytecode):
            opcode = int.from_bytes(bytecode[pc:pc+4], 'big')
            if opcode not in self.disasm.OPCODES:
                pc += 4
                continue
            
            _, operand_count = self.disasm.OPCODES[opcode]
            operand = 0
            if operand_count > 0:
                operand = int.from_bytes(bytecode[pc+4:pc+8], 'big')
                pc += 8
            else:
                pc += 4
            
            result.append((pc - (8 if operand_count else 4), opcode, operand))
        
        return result
    
    def decompile(self) -> str:
        stack = []
        statements = []
        
        for pc, opcode, operand in self.instructions:
            if opcode == 0x01:  # PUSH
                symbol = self.disasm.addr_to_symbol.get(operand, f"addr_{operand:08x}")
                stack.append(symbol)
            
            elif opcode == 0x02:  # POP
                if stack:
                    stack.pop()
            
            elif opcode == 0x06:  # EXTERN
                func_addr = operand
                heap_entry = self.disasm.heap.get(str(func_addr))
                if heap_entry and heap_entry['valueType'] == 'string':
                    func_sig = json.loads(heap_entry['valueJson'])
                    result = self._generate_extern_call(func_sig, stack)
                    statements.append(result)
            
            elif opcode == 0x09:  # COPY
                if len(stack) >= 2:
                    src = stack.pop()
                    dst = stack.pop()
                    statements.append(f"{dst} = {src};")
            
            elif opcode == 0x05:  # JUMP
                statements.append(f"goto label_{operand:08x};")
            
            elif opcode == 0x04:  # JUMP_IF_FALSE
                if stack:
                    condition = stack.pop()
                    statements.append(f"if (!{condition}) goto label_{operand:08x};")
            
            elif opcode == 0x08:  # JUMP_INDIRECT
                statements.append(f"return;")
        
        return "\n".join(statements)
    
    def _parse_extern_signature(self, sig: str):
        """解析: ClassName.__MethodName__ParamTypes__ReturnType"""
        parts = sig.split('__')
        if len(parts) < 2:
            return None, None, [], None
        
        class_name = parts[0]
        method_name = parts[1]
        
        # 解析参数和返回值
        if len(parts) >= 3:
            param_str = parts[2]
            params = [p for p in param_str.split('_') if p] if param_str else []
        else:
            params = []
        
        return_type = parts[3] if len(parts) > 3 else None
        
        return class_name, method_name, params, return_type
    
    def _generate_extern_call(self, func_sig: str, stack: List[str]) -> str:
        class_name, method_name, param_types, return_type = self._parse_extern_signature(func_sig)
        
        if not method_name:
            stack.clear()
            return f"// Unknown: {func_sig}"
        
        # 计算需要从栈弹出的参数数量
        has_return = return_type and return_type != "SystemVoid"
        is_instance_method = len(param_types) > 0 or method_name.startswith('get_')
        
        # 从栈弹出参数（按PUSH顺序）
        param_count = len(param_types) + (1 if is_instance_method and not method_name.startswith('get_') else 0)
        if method_name.startswith('get_'):
            param_count = 1  # 只有instance
        
        total_args = param_count + (1 if has_return else 0)
        
        args = []
        for _ in range(total_args):
            if stack:
                args.append(stack.pop(0))  # 从栈底弹出（保持顺序）
            else:
                args.append("???")
        
        # 第一个参数是输出变量（如果有返回值）
        output_var = args[0] if has_return else None
        remaining_args = args[1:] if has_return else args
        
        # 简化类名
        class_name = class_name.replace('VRCSDKBase', '').replace('UnityEngine', '')
        
        # 生成调用代码
        if method_name.startswith('get_'):
            # Getter属性
            prop_name = method_name[4:]
            if remaining_args:
                instance = remaining_args[0]
                call = f"{instance}.{prop_name}"
            else:
                call = f"{class_name}.{prop_name}"
        else:
            # 方法调用
            instance = remaining_args[0] if remaining_args else None
            method_args = remaining_args[1:] if len(remaining_args) > 1 else []
            
            args_str = ", ".join(method_args)
            if instance:
                call = f"{instance}.{method_name}({args_str})"
            else:
                call = f"{class_name}.{method_name}({args_str})"
        
        if output_var:
            return f"{output_var} = {call};"
        else:
            return f"{call};"

# 使用
disasm = UdonDisassembler('sample.json')
decompiler = UdonDecompiler(disasm)
print(decompiler.decompile())
