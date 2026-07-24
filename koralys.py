import sys
import time
from typing import List, Dict, Tuple, Any
from reader import Reader
from luau import (
    get_opcode,
    get_arg_a,
    get_arg_b,
    get_arg_c,
    get_arg_Bx,
    get_arg_sBx,
    get_arg_sAx,
    get_op_table,
)

DEBUG = False  #! Will slow down the decompilation process significantly


def debug(*args, **kwargs):
    return print(*args, **kwargs) if DEBUG else None


# < CONSTANT TYPES > #
LBC_CONSTANT_NIL = 0
LBC_CONSTANT_BOOLEAN = 1
LBC_CONSTANT_NUMBER = 2
LBC_CONSTANT_STRING = 3
LBC_CONSTANT_IMPORT = 4
LBC_CONSTANT_TABLE = 5
LBC_CONSTANT_CLOSURE = 6
LBC_CONSTANT_VECTOR = 8


def deserialize_v5(
    reader: Reader,
) -> Tuple[Dict[str, Any], List[Dict[str, Any]], List[str], int, int]:
    types_version = reader.nextByte()
    if types_version not in [1, 2, 3]:
        raise ValueError(f"Invalid types version (types version: {types_version})")

    proto_table: List[Dict[str, Any]] = []
    string_table: List[str] = []

    size_strings = reader.nextVarInt()
    string_table.extend(reader.nextString() for _ in range(size_strings))
    if types_version >= 3:
        index = reader.nextByte()
        while index != 0:
            index = reader.nextByte()
            
    size_protos = reader.nextVarInt()
    proto_table.extend(create_empty_proto() for _ in range(size_protos))

    for i in range(size_protos):
        proto = proto_table[i]
        read_proto_data(reader, proto, string_table)

    mainProtoId = reader.nextVarInt()
    if mainProtoId >= len(proto_table):
        raise IndexError(
            f"Index {mainProtoId} out of range for protoTable with length {len(proto_table)}"
        )
    return proto_table[mainProtoId], proto_table, string_table, 5, types_version


def create_empty_proto() -> Dict[str, Any]:
    return {
        "codeTable": [],
        "kTable": [],
        "pTable": [],
        "smallLineInfo": [],
        "largeLineInfo": [],
    }


def read_proto_data(reader: Reader, proto: Dict[str, Any], string_table: List[str]):
    proto["maxStackSize"] = reader.nextByte()
    proto["numParams"] = reader.nextByte()
    proto["numUpValues"] = reader.nextByte()
    proto["isVarArg"] = reader.nextByte()
    proto["flags"] = reader.nextByte()
    
    typesize = reader.nextVarInt()
    type_info = [reader.nextByte() for _ in range(typesize)]
    proto["typeInfo"] = type_info

    proto["sizeCode"] = reader.nextVarInt()
    proto["codeTable"].extend(reader.nextInt() for _ in range(proto["sizeCode"]))

    proto["sizeConsts"] = reader.nextVarInt()
    proto["kTable"] = [
        read_constant(reader, string_table) for _ in range(proto["sizeConsts"])
    ]

    proto["sizeProtos"] = reader.nextVarInt()
    proto["pTable"] = [reader.nextVarInt() for _ in range(proto["sizeProtos"])]

    proto["lineDefined"] = reader.nextVarInt()
    proto["source"] = read_proto_source(reader, string_table)

    if reader.nextByte() == 1:  # has line info?
        read_line_info(reader, proto)

    if reader.nextByte() == 1:  # has debug info?
        debug_info = {"varInfo": [], "upvalueInfo": []}
        size_vars = reader.nextVarInt()
        for _ in range(size_vars):
            var_name_idx = reader.nextVarInt() - 1
            var_name = string_table[var_name_idx] if 0 <= var_name_idx < len(string_table) else f"<var {var_name_idx}>"
            # FIX: These must be VarInt, not Uint32!
            start_pc = reader.nextVarInt()
            end_pc = reader.nextVarInt()
            reg = reader.nextByte()
            debug_info["varInfo"].append({"name": var_name, "startpc": start_pc, "endpc": end_pc, "reg": reg})
        size_upvalues = reader.nextVarInt()
        for _ in range(size_upvalues):
            uv_name_idx = reader.nextVarInt() - 1
            uv_name = string_table[uv_name_idx] if 0 <= uv_name_idx < len(string_table) else f"<upvalue {uv_name_idx}>"
            debug_info["upvalueInfo"].append({"name": uv_name})
        proto["debugInfo"] = debug_info


def read_constant(reader: Reader, string_table: List[str]) -> Dict[str, Any]:
    k = {"type": reader.nextByte()}
    if k["type"] == LBC_CONSTANT_NIL:
        k["value"] = None
    elif k["type"] == LBC_CONSTANT_BOOLEAN:
        k["value"] = reader.nextByte() == 1
    elif k["type"] == LBC_CONSTANT_NUMBER:
        k["value"] = reader.nextDouble()
    elif k["type"] == LBC_CONSTANT_STRING:
        raw_index = reader.nextVarInt()
        index = raw_index - 1
        k["value"] = (
            string_table[index]
            if 0 <= index < len(string_table)
            else "Invalid string index"
        )
    elif k["type"] == LBC_CONSTANT_IMPORT:
        k["value"] = reader.nextInt()
    elif k["type"] == LBC_CONSTANT_TABLE:
        size = reader.nextVarInt()
        k["value"] = {
            "size": size,
            "ids": [reader.nextVarInt() for _ in range(size)],
        }
    elif k["type"] == LBC_CONSTANT_CLOSURE:
        k["value"] = reader.nextVarInt() + 1
    elif k["type"] == LBC_CONSTANT_VECTOR:
        k["value"] = [reader.nextFloat() for _ in range(4)]
    elif k["type"] != 0:
        raise ValueError(f"Unrecognized constant type: {k['type']}")
    return k


def read_proto_source(reader: Reader, string_table: List[str]) -> str:
    protoSourceId = reader.nextVarInt()
    return (
        string_table[protoSourceId - 1]
        if 0 <= protoSourceId - 1 < len(string_table)
        else "Invalid source index"
    )


def read_line_info(reader: Reader, proto: Dict[str, Any]):
    compKey = reader.nextByte()
    proto["smallLineInfo"] = [reader.nextByte() for _ in range(proto["sizeCode"])]
    intervals = ((proto["sizeCode"] - 1) >> compKey) + 1
    proto["largeLineInfo"] = [reader.nextInt() for _ in range(intervals)]


def deserialize(
    bytecode: bytes,
) -> Tuple[Dict[str, Any], List[Dict[str, Any]], List[str], int, int]:
    reader = Reader(bytecode)
    version = reader.nextByte()
    if version == 5:
        return deserialize_v5(reader)
    elif version == 6:
        return deserialize_v6(reader)
    else:
        raise ValueError(f"Unsupported bytecode version: {version}")


def deserialize_v6(
    reader: Reader,
) -> Tuple[Dict[str, Any], List[Dict[str, Any]], List[str], int, int]:
    types_version = reader.nextByte()
    if types_version not in [1, 2, 3]:
        raise ValueError(f"Invalid types version (types version: {types_version})")

    proto_table: List[Dict[str, Any]] = []
    string_table: List[str] = []
    size_strings = reader.nextVarInt()
    string_table.extend(reader.nextString() for _ in range(size_strings))
    if types_version >= 3:
        index = reader.nextByte()
        while index != 0:
            index = reader.nextByte()

    size_protos = reader.nextVarInt()
    proto_table.extend(create_empty_proto() for _ in range(size_protos))

    for i in range(size_protos):
        proto = proto_table[i]
        read_proto_data(reader, proto, string_table)

    mainProtoId = reader.nextVarInt()
    if mainProtoId >= len(proto_table):
        raise IndexError(
            f"Index {mainProtoId} out of range for protoTable with length {len(proto_table)}"
        )
    return proto_table[mainProtoId], proto_table, string_table, 6, types_version


def read_proto(
    proto: Dict[str, Any],
    depth: int,
    proto_table: List[Dict[str, Any]],
    string_table: List[str],
    luau_version: int,
) -> str:
    OP_TABLE = get_op_table(luau_version)
    output = ""
    tab_space = "    " * (depth - 1)

    output += f"{tab_space}function({', '.join(['...' if proto['isVarArg'] else ''] + [f'R{i}' for i in range(proto['numParams'])])})\n"

    opcodeToOpname = {info.number: info.name for info in OP_TABLE}
    max_opname_length = max(len(info.name) for info in OP_TABLE)

    codeIndex = 0
    while codeIndex < len(proto["codeTable"]):
        i = proto["codeTable"][codeIndex]
        opc = get_opcode(i)
        A = get_arg_a(i)
        B = get_arg_b(i)
        Bx = get_arg_Bx(i)
        C = get_arg_c(i)
        sBx = get_arg_sBx(i)
        sAx = get_arg_sAx(i)

        op_name = opcodeToOpname.get(opc, "UNKNOWN")
        output += f"{'    ' * depth}[{codeIndex:03}] {op_name:<{max_opname_length}} "

        aux = None
        if any(info.name == op_name and info.get("aux", False) for info in OP_TABLE) and codeIndex + 1 < len(proto["codeTable"]):
            aux = proto["codeTable"][codeIndex + 1]
            codeIndex += 1

        def __CALL_handler(_):
            args = f"R{A+1}" + (f" ... R{A+B-1}" if B > 2 else "")
            returns = f"R{A}" + (f" ... R{A+C}" if C > 1 else "")
            return f"{returns} = R{A}({args})"

        def __CAPTURE_handler(_):
            capture_types = ["VAL", "REF", "UPVAL"]
            capture_type = capture_types[A] if A < len(capture_types) else f"Unknown({A})"
            return f"capture {capture_type} R{B}"

        def __GETIMPORT_handler(_):
            def decompose_import_id(ids: int) -> tuple[int, List[int]]:
                count = ids >> 30
                id1 = (ids >> 20) & 1023 if count > 0 else None
                id2 = (ids >> 10) & 1023 if count > 1 else None
                id3 = ids & 1023 if count > 2 else None
                return count, [x for x in [id1, id2, id3] if x is not None]

            def import_id_to_name(ids: int) -> str:
                imported_path = ""
                _, ids = decompose_import_id(ids)
                for j, id_constant in enumerate(ids):
                    id_constant = proto["kTable"][id_constant]
                    to_append = f".{id_constant['value']}" if j > 0 else id_constant["value"]
                    imported_path += to_append
                return imported_path

            import_id = proto["kTable"][Bx]["value"]
            imported_path = import_id_to_name(import_id)
            return f"R{A} = {imported_path} -- Import ID: {import_id}"

        def jump_if_gen(op: str | None = None, invert: bool = False, k_mode: bool = False):
            op_map = {"EQ": "==", "LE": "<=", "LT": "<"}
            current_A = A
            current_aux = aux
            pre_op = " not " if invert else " "
            jump = opcode_handlers["JUMP"]("JUMP")
            operator = op_map.get(op, op) if op else ""
            after_cond = operator and f" {operator} {k_mode and f'K{current_aux}' or f'R{current_aux}'} " or " "
            return f"if{pre_op}R{current_A}{after_cond}then {jump}"

        def jumpx_if_gen(value: str, curr_aux=None):
            not_flag = (curr_aux >> 31) & 1 if curr_aux is not None else 0
            op = "~=" if not_flag else "=="
            jump = opcode_handlers["JUMPX"]("JUMPX")
            return f"if R{A} {op} {value} then {jump}"

        def __LOADKX_handler(_):
            k = proto["kTable"][aux] if aux < len(proto["kTable"]) else {"type": "nil", "value": "nil"}
            return f"R{A} = {repr(k['value']) if isinstance(k['value'], str) else k['value']}"

        opcode_handlers = {
            "NOP": lambda _: "-- do nothing (no-op / NOP)",
            "BREAK": lambda _: "break",
            "PREPVARARGS": lambda _: f"(adjust vararg params, {A} fixed params)",
            "LOADNIL": lambda _: f"R{A} = nil",
            "LOADB": lambda _: (
                f"R{A} = {bool(B)}; goto [{codeIndex + C + 1}]"
                if C != 0
                else f"R{A} = {bool(B)}"
            ),
            "LOADN": lambda _: f"R{A} = {sBx}",
            "LOADK": lambda _: (
                f"R{A} = {repr(proto['kTable'][Bx]['value']) if isinstance(proto['kTable'][Bx]['value'], str) else proto['kTable'][Bx]['value']}"
                if Bx < len(proto["kTable"])
                else f"R{A} = K{Bx}"
            ),
            "MOVE": lambda _: f"R{A} = R{B}",
            "GETGLOBAL": lambda _, curr_aux=aux, curr_A=A: (
                f"R{curr_A} = _G[{repr(proto['kTable'][curr_aux]['value'])}]"
                if curr_aux is not None and curr_aux < len(proto['kTable'])
                else f"R{curr_A} = _G[Invalid constant index]"
            ),
            "SETGLOBAL": lambda _, curr_aux=aux, curr_A=A: (
                f"_G[{repr(proto['kTable'][curr_aux]['value'])}] = R{curr_A}"
                if curr_aux is not None and curr_aux < len(proto['kTable'])
                else f"_G[Invalid constant index] = R{curr_A}"
            ),
            "GETUPVAL": lambda _: f"R{A} = U{B}",
            "SETUPVAL": lambda _: f"U{B} = R{A}",
            "CLOSEUPVALS": lambda _: f"close upvalues R{A}+",
            "GETIMPORT": __GETIMPORT_handler,
            "GETTABLE": lambda _: f"R{A} = R{B}[R{C}]",
            "SETTABLE": lambda _: f"R{B}[R{C}] = R{A}",
            "GETTABLEKS": lambda _, curr_aux=aux, curr_A=A, curr_B=B: (
                f"R{curr_A} = R{curr_B}[{repr(proto['kTable'][curr_aux]['value'])}]"
                if curr_aux is not None and curr_aux < len(proto['kTable'])
                else f"R{curr_A} = R{curr_B}[Invalid constant index]"
            ),
            "SETTABLEKS": lambda _, curr_aux=aux, curr_A=A, curr_B=B: (
                f"R{curr_B}[{repr(proto['kTable'][curr_aux]['value'])}] = R{curr_A}"
                if curr_aux is not None and curr_aux < len(proto['kTable'])
                else f"R{curr_B}[Invalid constant index] = R{curr_A}"
            ),
            "GETTABLEN": lambda _: f"R{A} = R{B}[{C + 1}]",
            "SETTABLEN": lambda _: f"R{B}[{C + 1}] = R{A}",
            "NEWCLOSURE": lambda _: f"R{A} = closure(proto[{Bx}])",
            "NAMECALL": lambda _, curr_aux=aux, curr_A=A, curr_B=B: (
                f"R{curr_A} = R{curr_B}[{repr(proto['kTable'][curr_aux]['value'])}]; R{curr_A+1} = R{curr_B}"
                if curr_aux is not None and curr_aux < len(proto['kTable'])
                else f"R{curr_A} = R{curr_B}[Invalid constant index]; R{curr_A+1} = R{curr_B}"
            ),
            "CALL": __CALL_handler,
            "RETURN": lambda _: f"return R{A} ..."
            if B == 0
            else "return"
            if B == 1
            else f"return R{A} ... R{A+B-2}",
            "JUMP": lambda _: f"goto [{codeIndex + 1 + sBx}]",
            "JUMPBACK": lambda _: f"goto [{codeIndex + 1 + sBx}]",
            "JUMPX": lambda _: f"goto [{codeIndex + 1 + sAx}]",
            "JUMPXEQKNIL": lambda _, curr_aux=aux: jumpx_if_gen("nil", curr_aux),
            "JUMPXEQKB": lambda _, curr_aux=aux: jumpx_if_gen(
                str(bool(curr_aux & 1)).lower() if curr_aux is not None else "?",
                curr_aux
            ),
            "JUMPXEQKN": lambda _, curr_aux=aux: (
                jumpx_if_gen(
                    (lambda k: repr(k['value']) if isinstance(k['value'], str) else str(k['value']))(
                        proto['kTable'][(curr_aux & 0x7FFFFFFF)]
                    ) if curr_aux is not None and (curr_aux & 0x7FFFFFFF) < len(proto['kTable'])
                    else f"K{curr_aux}",
                    curr_aux
                )
            ),
            "JUMPXEQKS": lambda _, curr_aux=aux: (
                jumpx_if_gen(
                    repr(proto['kTable'][(curr_aux & 0x7FFFFFFF)]['value'])
                    if curr_aux is not None and (curr_aux & 0x7FFFFFFF) < len(proto['kTable'])
                    else f"K{curr_aux}",
                    curr_aux
                )
            ),
            "FASTCALL": lambda _: f"R{A} = builtin[{C}]",
            "FASTCALL1": lambda _: f"R{A} = builtin[{C}](R{B})",
            "FASTCALL2": lambda _: f"R{A} = builtin[{C}](R{B}, R{aux})",
            "FASTCALL2K": lambda _: f"R{A} = builtin[{C}](R{B}, K{aux})",
            "FASTCALL3": lambda _: f"R{A} = builtin[{C}]",
            "COVERAGE": lambda _: "(coverage)",
            "CAPTURE": __CAPTURE_handler,
            "JUMPIFEQK": lambda _: jump_if_gen("==", k_mode=True),
            "FORNPREP": lambda _: f"... goto [{codeIndex + 1 + sBx}]",
            "FORNLOOP": lambda _: f"... goto [{codeIndex + 1 + sBx}]; ...",
            "MINUS": lambda _: f"R{A} = -R{B}",
            "LENGTH": lambda _: f"R{A} = #R{B}",
            "NEWTABLE": lambda _, curr_aux=aux: (
                f"R{A} = {{}} -- hash={0 if B == 0 else 1 << (B - 1)}, array={curr_aux if curr_aux is not None else 0}"
            ),
            "DUPTABLE": lambda _: f"R{A} = K{Bx} -- duplicate",
            "SETLIST": lambda _, curr_aux=aux: (
                f"R{A}[{curr_aux}..{curr_aux+C-1}] = R{B} ... R{B+C-1}"
                if C > 0 and curr_aux is not None
                else f"R{A}[{curr_aux}..] = R{B} ... top"
            ),
            "CONCAT": lambda _: f"R{A} = R{B} .. R{C}",
            "NOT": lambda _: f"R{A} = not R{B}",
            "FORGPREP": lambda _: f"... goto [{codeIndex + 1 + sBx}]",
            "FORGLOOP": lambda _, curr_aux=aux: (
                f"R{A+3}, ..., R{A+2+(curr_aux & 0x7F)} = R{A}(R{A+1}, R{A+2}); "
                f"if R{A+3} ~= nil then R{A+2} = R{A+3}; goto [{codeIndex + 1 + sBx}]"
                if curr_aux is not None
                else f"R{A+3}, ... = R{A}(R{A+1}, R{A+2}); goto [{codeIndex + 1 + sBx}]"
            ),
            "FORGPREP_INEXT": lambda _: f"... goto [{codeIndex + 1 + sBx}]",
            "NATIVECALL": lambda _: "Unimplemented",
            "GETVARARGS": lambda _: (
                f"R{A}, ... = ..."
                if B == 0
                else f"R{A}, ..., R{A+B-2} = ..."
            ),
            "DUPCLOSURE": lambda _: f"R{A} = K{Bx} -- duplicate",
            "LOADKX": __LOADKX_handler,
            "FORGPREP_NEXT": lambda _: f"... goto [{codeIndex + 1 + sBx}]",
        }

        for condition in ["EQ", "LE", "LT", None]:
            opcode_handlers[f"JUMPIF{condition or ''}"] = lambda _, cond=condition: jump_if_gen(cond)
            opcode_handlers[f"JUMPIFNOT{condition or ''}"] = lambda _, cond=condition: jump_if_gen(cond, True)

        for gen_op_name in ["AND", "OR"]:
            def __gen_op_handler(gen_op_name: str):
                op = "and" if gen_op_name.startswith("AND") else "or"
                if gen_op_name.endswith("K"):
                    k = (
                        proto["kTable"][C]
                        if C < len(proto["kTable"])
                        else {"type": "nil", "value": "nil"}
                    )
                    return f"R{A} = R{B} {op} "\
                        f"{repr(k['value']) if isinstance(k['value'], str) else k['value']}"
                else:
                    return f"R{A} = R{B} {op} R{C}"
            opcode_handlers[gen_op_name] = __gen_op_handler
            opcode_handlers[f"{gen_op_name}K"] = __gen_op_handler

        math_ops = {
            "ADD": "+",
            "SUB": "-",
            "MUL": "*",
            "DIV": "/",
            "IDIV": "//",
            "MOD": "%",
            "POW": "^",
        }

        opcode_handlers["ADDRK"] = lambda _: f"R{A} = K{B} + R{C}"
        for gen_op_name in ["SUBRK", "DIVRK"]:
            opcode_handlers[gen_op_name] = lambda op: f"R{A} = K{B} {math_ops[op[:-2]]} R{C}"

        for gen_op_name in ["ADD", "SUB", "MUL", "DIV", "IDIV", "MOD", "POW"]:
            opcode_handlers[gen_op_name] = (
                lambda opcode: f"R{A} = R{B} {math_ops[opcode]} R{C}"
            )

            def __gen_op_handler(opcode):
                op = math_ops[opcode[:-1]]
                k = (
                    proto["kTable"][C]
                    if C < len(proto["kTable"])
                    else {"type": "nil", "value": "nil"}
                )
                return f"R{A} = R{B} {op} {repr(k['value']) if isinstance(k['value'], str) else k['value']}"

            opcode_handlers[f"{gen_op_name}K"] = __gen_op_handler

        if op_name in opcode_handlers:
            output += opcode_handlers[op_name](op_name)
        else:
            output += f"Unknown opcode: {opc}"

        output += "\n"
        codeIndex += 1

    output += "end\n"

    if len(proto["kTable"]) > 0:
        output += "--< Constants >--\n"
        constant_types = {
            LBC_CONSTANT_NIL: lambda k: "nil",
            LBC_CONSTANT_BOOLEAN: lambda k: str(k["value"]).lower(),
            LBC_CONSTANT_NUMBER: lambda k: k["value"],
            LBC_CONSTANT_STRING: lambda k: repr(k["value"]),
            LBC_CONSTANT_IMPORT: lambda k: k["value"],
            LBC_CONSTANT_TABLE: lambda k: k["value"],
            LBC_CONSTANT_CLOSURE: lambda k: k["value"],
            LBC_CONSTANT_VECTOR: lambda k: k["value"],
        }
        for i, k in enumerate(proto["kTable"]):
            value = constant_types.get(
                k["type"], lambda k: f"Unknown constant type: {k['type']}"
            )(k)
            output += f"{'    ' * depth}[{i}] = {value}\n"

    if "sizeProtos" in proto and proto["sizeProtos"] > 0:
        output += "--< Protos >--\n"
        for i, proto_idx in enumerate(proto["pTable"]):
            if proto_idx < len(proto_table):
                child_proto = proto_table[proto_idx]
                output += f"{'    ' * depth}[{i}] = {read_proto(child_proto, depth + 1, proto_table, string_table, luau_version)}\n"
            else:
                output += f"{'    ' * depth}[{i}] = <invalid proto index {proto_idx}>\n"

    if proto["numUpValues"] > 0:
        output += "--< Upvalues >--\n"
        for i in range(proto["numUpValues"]):
            output += f"{'    ' * depth}[{i}] = Upvalue {i}\n"

    return output


def disassemble(bytecode: bytes) -> Tuple[List[str], List[str], int, int, int]:
    output = []
    decompiled_output = []

    if bytecode[0] == 0:
        return [bytecode[1:].decode("utf-8")], [], 0, -1, -1

    mainProto, protoTable, stringTable, luau_version, types_version = deserialize(
        bytecode
    )

    child_proto_indices = set()
    for proto in protoTable:
        for child_idx in proto.get("pTable", []):
            child_proto_indices.add(child_idx)

    protos = 0
    for i, proto in enumerate(protoTable):
        if i in child_proto_indices:
            continue
        output.extend(
            (
                f"--< Proto->{i:03} | Line {proto.get('lineDefined', 0)} >--",
                read_proto(proto, 1, protoTable, stringTable, luau_version),
            )
        )
        decompiled_output.extend(
            (
                f"-- Decompiled Proto->{i:03} --",
                decompile(proto, 1, stringTable, luau_version, protoTable),
            )
        )
        protos += 1

    return output, decompiled_output, protos, luau_version, types_version


def decompose_import_id(ids: int) -> tuple[int, list[int]]:
    count = ids >> 30
    id1 = (ids >> 20) & 1023 if count > 0 else None
    id2 = (ids >> 10) & 1023 if count > 1 else None
    id3 = ids & 1023 if count > 2 else None
    return count, [x for x in [id1, id2, id3] if x is not None]


def import_id_to_name(proto: dict, ids: int) -> str:
    if not isinstance(ids, int):
        return str(ids)
    imported_path = ""
    _, ids = decompose_import_id(ids)
    for i, id_constant in enumerate(ids):
        if id_constant < len(proto.get("kTable", [])):
            entry = proto["kTable"][id_constant]
            name = entry.get("value", f"<const {id_constant}>")
            to_append = f".{name}" if i > 0 else str(name)
            imported_path += to_append
        else:
            imported_path += f"<const {id_constant}>"
    return imported_path


def decompile(
    proto: Dict[str, Any], depth: int, stringTable: List[str], luau_version: int, proto_table: List[Dict[str, Any]] = None
) -> str:
    output = []
    OP_TABLE = get_op_table(luau_version)

    def add_tab_space(d):
        return "    " * d

    # --- Variable Name Mapping ---
    reg_names = {}
    if "debugInfo" in proto and "varInfo" in proto["debugInfo"]:
        for var in proto["debugInfo"]["varInfo"]:
            reg_names[var["reg"]] = var["name"]
            
    uv_names = {}
    if "debugInfo" in proto and "upvalueInfo" in proto["debugInfo"]:
        for i, uv in enumerate(proto["debugInfo"]["upvalueInfo"]):
            uv_names[i] = uv["name"]

    def rn(r):
        return reg_names.get(r, f"R{r}")

    def un(u):
        return uv_names.get(u, f"U{u}")

    # --- Function Signature ---
    params = []
    if "debugInfo" in proto and "varInfo" in proto["debugInfo"]:
        for var in proto["debugInfo"]["varInfo"]:
            if var["startpc"] == 0 and len(params) < proto['numParams']:
                params.append(var["name"])
    
    while len(params) < proto['numParams']:
        params.append(f"R{len(params)}")
        
    if proto['isVarArg']:
        params.append("...")
        
    output.append(f"local function func{depth}({', '.join(params)})")

    opcode_to_opname = {info.number: info.name for info in OP_TABLE}

    def format_constant(k):
        if not isinstance(k, dict):
            return str(k)
        t = k["type"]
        if t == LBC_CONSTANT_NIL:
            return "nil"
        elif t == LBC_CONSTANT_BOOLEAN:
            return str(k["value"]).lower()
        elif t == LBC_CONSTANT_NUMBER:
            return str(k["value"])
        elif t == LBC_CONSTANT_STRING:
            return repr(k["value"])
        elif t == LBC_CONSTANT_VECTOR:
            v = k.get("value")
            if isinstance(v, (list, tuple)):
                return f"vector({', '.join(str(x) for x in v)})"
            return str(v)
        elif t == LBC_CONSTANT_TABLE:
            v = k.get("value", {})
            if isinstance(v, dict):
                sz = v.get("size", 0)
                ids = v.get("ids", [])
                return f"table<size={sz},ids={ids}>"
            return str(v)
        elif t == LBC_CONSTANT_CLOSURE:
            return f"closure({k.get('value', '?')})"
        elif t == LBC_CONSTANT_IMPORT:
            return f"import<{k.get('value', '?')}>"
        else:
            return str(k.get("value", k))

    code_index = 0
    while code_index < len(proto["codeTable"]):
        try:
            i = proto["codeTable"][code_index]
            opc = get_opcode(i)
            opname = opcode_to_opname.get(opc, "UNKNOWN")
            A = get_arg_a(i)
            B = get_arg_b(i)
            Bx = get_arg_Bx(i)
            C = get_arg_c(i)
            sBx = get_arg_sBx(i)
            sAx = get_arg_sAx(i)
            
            aux = (
                proto["codeTable"][code_index + 1]
                if any(info.name == opname and info.get("aux", False) for info in OP_TABLE) \
                and code_index + 1 < len(proto["codeTable"])
                else None
            )
            if aux is not None:
                code_index += 1

            if opname == "LOADNIL":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = nil")
            elif opname == "LOADB":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {bool(B)}")
                if C != 0:
                    output.append(f"{add_tab_space(depth + 1)}goto [{code_index + 1 + C}]")
            elif opname == "LOADN":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {sBx}")
            elif opname == "LOADK":
                if Bx < len(proto["kTable"]):
                    k = proto["kTable"][Bx]
                    output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {format_constant(k)}")
                else:
                    output.append(f"{add_tab_space(depth + 1)}{rn(A)} = <invalid index {Bx}>")
            elif opname == "MOVE":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {rn(B)}")
            elif opname == "GETGLOBAL":
                if aux is not None and aux < len(proto['kTable']):
                    output.append(f"{add_tab_space(depth + 1)}{rn(A)} = _G[{repr(proto['kTable'][aux]['value'])}]")
                else:
                    output.append(f"{add_tab_space(depth + 1)}{rn(A)} = _G[Invalid constant index]")
            elif opname == "SETGLOBAL":
                if aux is not None and aux < len(proto['kTable']):
                    output.append(f"{add_tab_space(depth + 1)}_G[{repr(proto['kTable'][aux]['value'])}] = {rn(A)}")
                else:
                    output.append(f"{add_tab_space(depth + 1)}_G[Invalid string index] = {rn(A)}")
            elif opname == "GETUPVAL":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {un(B)}")
            elif opname == "SETUPVAL":
                output.append(f"{add_tab_space(depth + 1)}{un(B)} = {rn(A)}")
            elif opname == "CLOSEUPVALS":
                output.append(f"{add_tab_space(depth + 1)}close upvalues {rn(A)}+")
            elif opname == "GETIMPORT":
                if Bx < len(proto["kTable"]):
                    import_id = proto["kTable"][Bx]["value"]
                    if isinstance(import_id, int):
                        decomposed = import_id_to_name(proto, import_id)
                        output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {decomposed}")
                    else:
                        output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {repr(import_id)}")
                else:
                    output.append(f"{add_tab_space(depth + 1)}{rn(A)} = <invalid import index {Bx}>")
            elif opname == "GETTABLE":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {rn(B)}[{rn(C)}]")
            elif opname == "SETTABLE":
                output.append(f"{add_tab_space(depth + 1)}{rn(B)}[{rn(C)}] = {rn(A)}")
            elif opname == "GETTABLEKS":
                if aux is not None and aux < len(proto['kTable']):
                    output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {rn(B)}[{repr(proto['kTable'][aux]['value'])}]")
                else:
                    output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {rn(B)}[Invalid string index]")
            elif opname == "SETTABLEKS":
                if aux is not None and aux < len(proto['kTable']):
                    output.append(f"{add_tab_space(depth + 1)}{rn(B)}[{repr(proto['kTable'][aux]['value'])}] = {rn(A)}")
                else:
                    output.append(f"{add_tab_space(depth + 1)}{rn(B)}[Invalid string index] = {rn(A)}")
            elif opname == "GETTABLEN":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {rn(B)}[{C + 1}]")
            elif opname == "SETTABLEN":
                output.append(f"{add_tab_space(depth + 1)}{rn(B)}[{C + 1}] = {rn(A)}")
            elif opname == "NEWCLOSURE":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = closure(proto[{Bx}])")
            elif opname == "NAMECALL":
                if aux is not None and aux < len(proto['kTable']):
                    output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {rn(B)}[{repr(proto['kTable'][aux]['value'])}]; {rn(A+1)} = {rn(B)}")
                else:
                    output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {rn(B)}[Invalid string index]; {rn(A+1)} = {rn(B)}")
            elif opname == "CALL":
                if B == 1:
                    args = ""
                elif B == 0:
                    args = f"{rn(A+1)} ..."
                else:
                    args = f"{rn(A+1)}" + (f" ... {rn(A+B-1)}" if B > 2 else "")
                
                if C == 0:
                    returns = f"{rn(A)} ..."
                elif C == 1:
                    returns = ""
                else:
                    returns = f"{rn(A)}" + (f" ... {rn(A+C-2)}" if C > 2 else "")
                
                call_str = f"{rn(A)}({args})"
                if returns:
                    output.append(f"{add_tab_space(depth + 1)}{returns} = {call_str}")
                else:
                    output.append(f"{add_tab_space(depth + 1)}{call_str}")
            elif opname == "RETURN":
                if B == 0:
                    output.append(f"{add_tab_space(depth + 1)}return {rn(A)} ...")
                elif B == 1:
                    output.append(f"{add_tab_space(depth + 1)}return")
                else:
                    output.append(f"{add_tab_space(depth + 1)}return {rn(A)} ... {rn(A+B-2)}")
            elif opname in ["JUMP", "JUMPBACK"]:
                target = code_index + 1 + sBx  # sBx is already signed for JUMPBACK
                output.append(f"{add_tab_space(depth + 1)}goto [{target}]")
            elif opname in ["JUMPIF", "JUMPIFNOT"]:
                condition = "" if opname == "JUMPIF" else "not "
                output.append(f"{add_tab_space(depth + 1)}if {condition}{rn(A)} then goto [{code_index + 1 + sBx}]")
            elif opname in ["JUMPIFEQ", "JUMPIFLE", "JUMPIFLT", "JUMPIFNOTEQ", "JUMPIFNOTLE", "JUMPIFNOTLT"]:
                op = {
                    "JUMPIFEQ": "==", "JUMPIFLE": "<=", "JUMPIFLT": "<",
                    "JUMPIFNOTEQ": "~=", "JUMPIFNOTLE": ">", "JUMPIFNOTLT": ">=",
                }[opname]
                output.append(f"{add_tab_space(depth + 1)}if {rn(A)} {op} {rn(aux)} then goto [{code_index + 1 + sBx}]")
            elif opname in ["ADD", "SUB", "MUL", "DIV", "MOD", "POW", "ADDK", "SUBK", "MULK", "DIVK", "MODK", "POWK", "ADDRK", "SUBRK", "DIVRK"]:
                op = {
                    "ADD": "+", "SUB": "-", "MUL": "*", "DIV": "/", "MOD": "%", "POW": "^",
                    "ADDK": "+", "SUBK": "-", "MULK": "*", "DIVK": "/", "MODK": "%", "POWK": "^",
                    "ADDRK": "+", "SUBRK": "-", "DIVRK": "/",
                }[opname]
                if opname.endswith("RK"):
                    k = proto["kTable"][B] if B < len(proto["kTable"]) else {"type": "nil", "value": "nil"}
                    output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {format_constant(k)} {op} {rn(C)}")
                elif opname.endswith("K"):
                    k = proto["kTable"][C] if C < len(proto["kTable"]) else {"type": "nil", "value": "nil"}
                    output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {rn(B)} {op} {format_constant(k)}")
                else:
                    output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {rn(B)} {op} {rn(C)}")
            elif opname in ["AND", "OR", "ANDK", "ORK"]:
                op = "and" if opname.startswith("AND") else "or"
                if opname.endswith("K"):
                    k = proto["kTable"][C] if C < len(proto["kTable"]) else {"type": "nil", "value": "nil"}
                    output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {rn(B)} {op} {format_constant(k)}")
                else:
                    output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {rn(B)} {op} {rn(C)}")
            elif opname == "NOT":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = not {rn(B)}")
            elif opname == "NOP":
                output.append(f"{add_tab_space(depth + 1)}nop")
            elif opname == "BREAK":
                output.append(f"{add_tab_space(depth + 1)}break")
            elif opname == "FORNPREP":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = fornprep({rn(A)}, {sBx})")
            elif opname == "FORNLOOP":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = fornloop({rn(A)}, {sBx})")
            elif opname == "MINUS":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = -{rn(B)}")
            elif opname == "LENGTH":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = #{rn(B)}")
            elif opname == "CONCAT":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {rn(B)} .. {rn(C)}")
            elif opname == "JUMPIFEQK":
                k_val = format_constant(proto["kTable"][Bx]) if Bx < len(proto["kTable"]) else repr(Bx)
                output.append(f"{add_tab_space(depth + 1)}if {rn(A)} == {k_val} then goto [{code_index + 1 + sBx}]")
            elif opname == "FASTCALL":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = fastcall({B}, {C})")
            elif opname == "FASTCALL1":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = fastcall1({B}, {rn(C)})")
            elif opname == "FASTCALL2":
                if aux is not None:
                    output.append(f"{add_tab_space(depth + 1)}{rn(A)} = fastcall2({B}, {rn(C)}, {rn(aux)})")
                else:
                    output.append(f"{add_tab_space(depth + 1)}{rn(A)} = fastcall2({B}, {rn(C)}, <invalid register>)")
            elif opname == "FASTCALL2K":
                k = proto["kTable"][aux] if aux < len(proto["kTable"]) else {"type": "nil", "value": "nil"}
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = fastcall2k({B}, {rn(C)}, {format_constant(k)})")
            elif opname == "FORGLOOP":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = forgloop({rn(A)}, {sBx})")
            elif opname == "FORGLOOP_INEXT":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = forgloop_inext({rn(A)}, {sBx})")
            elif opname == "FORGLOOP_NEXT":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = forgloop_next({rn(A)}, {sBx})")
            elif opname == "FORGPREP":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = forgprep({rn(A)}, {sBx})")
            elif opname == "FORGPREP_INEXT":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = forgprep_inext({rn(A)}, {sBx})")
            elif opname == "FORGPREP_NEXT":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = forgprep_next({rn(A)}, {sBx})")
            elif opname == "GETVARARGS":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)}, ... = ..., ({B - 1} args)")
            elif opname == "DUPCLOSURE":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = dupclosure(K{Bx})")
            elif opname == "PREPVARARGS":
                output.append(f"{add_tab_space(depth + 1)}prepare_varargs({A})")
            elif opname == "LOADKX":
                if aux is not None:
                    k = proto["kTable"][aux] if aux < len(proto["kTable"]) else {"type": "nil", "value": "nil"}
                    output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {format_constant(k)}")
                else:
                    output.append(f"{add_tab_space(depth + 1)}{rn(A)} = <invalid constant>")
            elif opname == "JUMPX":
                output.append(f"{add_tab_space(depth + 1)}goto [{code_index + 1 + sAx}]")
            elif opname == "NEWTABLE":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {{}}")
            elif opname == "DUPTABLE":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {{}}")
            elif opname == "SETLIST":
                if C == 0:
                    output.append(f"{add_tab_space(depth + 1)}{rn(A)}[{aux}..] = {rn(B)} ... top")
                else:
                    output.append(f"{add_tab_space(depth + 1)}{rn(A)}[{aux}..{aux+C-1}] = {rn(B)} ... {rn(B+C-1)}")
            elif opname == "CAPTURE":
                if A == 0:
                    output.append(f"{add_tab_space(depth + 1)}capture(upvalue, {rn(B)})")
                else:
                    output.append(f"{add_tab_space(depth + 1)}capture({rn(B)})")
            elif opname == "JUMPXEQKNIL":
                output.append(f"{add_tab_space(depth + 1)}if {rn(A)} == nil then goto [{code_index + 1 + sAx}]")
            elif opname == "JUMPXEQKB":
                not_flag = bool(aux & 0x80000000) if aux is not None else False
                cmp_op = "~=" if not_flag else "=="
                output.append(f"{add_tab_space(depth + 1)}if {rn(A)} {cmp_op} {bool(aux & 1)} then goto [{code_index + 1 + sAx}]")
            elif opname == "JUMPXEQKN":
                k_idx = aux & 0x7FFFFFFF if aux is not None else 0
                not_flag = bool(aux & 0x80000000) if aux is not None else False
                if k_idx < len(proto["kTable"]):
                    k_val = format_constant(proto["kTable"][k_idx])
                    cmp_op = "~=" if not_flag else "=="
                    output.append(f"{add_tab_space(depth + 1)}if {rn(A)} {cmp_op} {k_val} then goto [{code_index + 1 + sAx}]")
                else:
                    output.append(f"{add_tab_space(depth + 1)}if {rn(A)} == K{k_idx} then goto [{code_index + 1 + sAx}]")
            elif opname == "JUMPXEQKS":
                k_idx = aux & 0x7FFFFFFF if aux is not None else None
                not_flag = bool(aux & 0x80000000) if aux is not None else False
                if k_idx is not None and k_idx < len(proto['kTable']):
                    k_val = repr(proto['kTable'][k_idx]['value'])
                    cmp_op = "~=" if not_flag else "=="
                    output.append(f"{add_tab_space(depth + 1)}if {rn(A)} {cmp_op} {k_val} then goto [{code_index + 1 + sAx}]")
                else:
                    output.append(f"{add_tab_space(depth + 1)}if {rn(A)} == <invalid string> then goto [{code_index + 1 + sAx}]")
            elif opname == "IDIV":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {rn(B)} // {rn(C)}")
            elif opname == "IDIVK":
                k = proto["kTable"][C] if C < len(proto["kTable"]) else {"type": "nil", "value": "nil"}
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {rn(B)} // {format_constant(k)}")
            elif opname == "BAND":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {rn(B)} & {rn(C)}")
            elif opname == "BOR":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {rn(B)} | {rn(C)}")
            elif opname == "BXOR":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {rn(B)} ~ {rn(C)}")
            elif opname == "BNOT":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = ~{rn(B)}")
            elif opname == "SHL":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {rn(B)} << {rn(C)}")
            elif opname == "SHR":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {rn(B)} >> {rn(C)}")
            elif opname == "BANDK":
                k = proto["kTable"][C] if C < len(proto["kTable"]) else {"type": "nil", "value": "nil"}
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {rn(B)} & {format_constant(k)}")
            elif opname == "BORK":
                k = proto["kTable"][C] if C < len(proto["kTable"]) else {"type": "nil", "value": "nil"}
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {rn(B)} | {format_constant(k)}")
            elif opname == "BXORK":
                k = proto["kTable"][C] if C < len(proto["kTable"]) else {"type": "nil", "value": "nil"}
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {rn(B)} ~ {format_constant(k)}")
            elif opname == "SHLI":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {rn(B)} << {C}")
            elif opname == "SHRI":
                output.append(f"{add_tab_space(depth + 1)}{rn(A)} = {rn(B)} >> {C}")
            elif opname == "COVERAGE":
                output.append(f"{add_tab_space(depth + 1)}coverage({aux})")
            else:
                output.append(f"{add_tab_space(depth + 1)}UNKNOWN OPCODE: {opname}")
        except Exception as e:
            output.append(f"{add_tab_space(depth + 1)}Error processing opcode: {str(e)}")
        code_index += 1

    if proto_table is not None and "pTable" in proto and len(proto.get("pTable", [])) > 0:
        for i, child_id in enumerate(proto["pTable"]):
            if child_id < len(proto_table):
                output.append(f"\n{add_tab_space(depth + 1)}-- child proto {i}")
                child_proto = proto_table[child_id]
                output.append(decompile(child_proto, depth + 1, stringTable, luau_version, proto_table))

    output.append("end")
    return "\n".join(output)


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python koralys.py <bytecode_file>")
        sys.exit(1)

    with open(sys.argv[1], "rb") as f:
        bytecode = f.read()

    start = time.perf_counter()
    disassembled, decompiled, protos, luau_version, types_version = disassemble(bytecode)
    end = time.perf_counter()

    if DEBUG:
        print("\n".join(disassembled))
    disassembled_extra = "--<@ Disassembled with Koralys' BETA disassembler @>--\n"
    versions = (
        f"Luau version {luau_version}, types version {types_version}"
        if luau_version != -1
        else f"Luau version unknown, types version {types_version}"
        if types_version != -1
        else "Types version unknown, luau version unknown"
    )
    disassembled_extra += f"--<@ Protos: {protos} | {versions} @>--\n"
    disassembled_extra += f"--<@ Time taken: {end - start:.6f}s @>--\n"
    disassembled_str = "\n".join(disassembled)
    full_output = disassembled_extra + disassembled_str
    with open("output.txt", "w", encoding="utf-8") as f:
        f.write(full_output)
    print(f"Disassembled bytecode in {end - start:.6f}s")
    
    flattened_decompiled = []
    for item in decompiled:
        if isinstance(item, list):
            flattened_decompiled.extend(item)
        else:
            flattened_decompiled.append(item)
    decompiled_str = "\n".join(flattened_decompiled)
    with open("decompiled.luau", "w", encoding="utf-8") as f:
        f.write(decompiled_str)
    print("Decompiled disassembly")
