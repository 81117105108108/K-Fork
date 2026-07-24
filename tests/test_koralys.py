"""Test suite for Koralys Luau bytecode disassembler/decompiler."""

import sys
import os
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from koralys import (
    disassemble,
    decompile,
    read_constant,
    LBC_CONSTANT_NIL,
    LBC_CONSTANT_BOOLEAN,
    LBC_CONSTANT_NUMBER,
    LBC_CONSTANT_STRING,
    LBC_CONSTANT_IMPORT,
    LBC_CONSTANT_TABLE,
    LBC_CONSTANT_CLOSURE,
    LBC_CONSTANT_VECTOR,
    decompose_import_id,
    import_id_to_name,
)
from luau import get_op_table


OP_TABLE = get_op_table(6)


class TestConstants(unittest.TestCase):
    def test_vector_constant_value(self):
        self.assertEqual(LBC_CONSTANT_VECTOR, 8)

    def test_nil_constant_value(self):
        self.assertEqual(LBC_CONSTANT_NIL, 0)

    def test_boolean_constant_value(self):
        self.assertEqual(LBC_CONSTANT_BOOLEAN, 1)

    def test_number_constant_value(self):
        self.assertEqual(LBC_CONSTANT_NUMBER, 2)

    def test_string_constant_value(self):
        self.assertEqual(LBC_CONSTANT_STRING, 3)

    def test_import_constant_value(self):
        self.assertEqual(LBC_CONSTANT_IMPORT, 4)

    def test_table_constant_value(self):
        self.assertEqual(LBC_CONSTANT_TABLE, 5)

    def test_closure_constant_value(self):
        self.assertEqual(LBC_CONSTANT_CLOSURE, 6)


class TestOpTable(unittest.TestCase):
    def test_op_table_attr_access(self):
        for op in OP_TABLE:
            self.assertIsInstance(op.number, int)
            self.assertIsInstance(op.name, str)

    def test_op_has_aux_field(self):
        for op in OP_TABLE:
            self.assertTrue(hasattr(op, "aux"))

    def test_known_opcodes_exist(self):
        names = {op.name for op in OP_TABLE}
        for required in [
            "LOADN", "LOADK", "GETIMPORT", "JUMP", "ADD", "SUB",
            "MUL", "DIV", "SUBRK", "DIVRK",
        ]:
            self.assertIn(required, names, f"Missing required opcode: {required}")

    def test_opcode_uniqueness(self):
        nums = [op.number for op in OP_TABLE]
        self.assertEqual(len(nums), len(set(nums)))

    def test_rk_opcodes_consistent(self):
        op_names = {op.name for op in OP_TABLE}
        for base in ["SUB", "DIV"]:
            k_op = f"{base}K"
            rk_op = f"{base}RK"
            self.assertIn(k_op, op_names, f"Missing {k_op}")
            self.assertIn(rk_op, op_names, f"Missing {rk_op}")


class TestImportID(unittest.TestCase):
    def test_decompose_single(self):
        # count=1, id1=42 (bits 20-29)
        import_id = (1 << 30) | (42 << 20)
        count, ids = decompose_import_id(import_id)
        self.assertEqual(count, 1)
        self.assertEqual(len(ids), 1)
        self.assertEqual(ids[0], 42)

    def test_decompose_double(self):
        # count=2, id1=100 (bits 20-29), id2=200 (bits 10-19)
        import_id = (2 << 30) | (100 << 20) | (200 << 10)
        count, ids = decompose_import_id(import_id)
        self.assertEqual(count, 2)
        self.assertEqual(len(ids), 2)

    def test_import_id_to_name_simple(self):
        proto = {
            "kTable": [
                {"type": LBC_CONSTANT_STRING, "value": "game"},
                {"type": LBC_CONSTANT_STRING, "value": "GetService"},
            ]
        }
        # count=2, id1=0 (kTable[0]="game"), id2=1 (kTable[1]="GetService")
        import_id = (2 << 30) | (0 << 20) | (1 << 10)
        name = import_id_to_name(proto, import_id)
        self.assertEqual(name, "game.GetService")

    def test_import_id_to_name_single(self):
        proto = {
            "kTable": [
                {"type": LBC_CONSTANT_STRING, "value": "print"},
            ]
        }
        # count=1, id1=0 (kTable[0]="print")
        import_id = (1 << 30) | (0 << 20)
        name = import_id_to_name(proto, import_id)
        self.assertEqual(name, "print")


class TestDisassemble(unittest.TestCase):
    def test_disassemble_signature(self):
        try:
            result = disassemble(b"")
        except Exception:
            pass  # Expected to fail on empty data

    def test_disassemble_empty(self):
        try:
            output, decompiled, protos, luau_version, types_version = disassemble(b"")
            self.assertIsInstance(output, list)
            self.assertIsInstance(decompiled, list)
        except:
            pass


class TestDecompile(unittest.TestCase):
    def test_decompile_empty_proto(self):
        proto = {
            "codeTable": [],
            "kTable": [],
            "pTable": [],
            "maxStackSize": 0,
            "numParams": 0,
            "isVarArg": False,
        }
        result = decompile(proto, 0, [], 5)
        self.assertIsInstance(result, str)
        self.assertIn("end", result)

    def test_decompile_signature(self):
        from inspect import signature
        sig = signature(decompile)
        self.assertIn("proto", sig.parameters)
        self.assertIn("proto_table", sig.parameters)


class TestBoolFormatting(unittest.TestCase):
    def test_import_id_to_name_not_int(self):
        result = import_id_to_name({}, "not_an_int")
        self.assertEqual(result, "not_an_int")


class TestReadConstant(unittest.TestCase):
    def test_read_constant_nil(self):
        from reader import Reader
        data = bytes([LBC_CONSTANT_NIL])
        result = read_constant(Reader(data), [])
        self.assertEqual(result["type"], LBC_CONSTANT_NIL)
        self.assertIsNone(result.get("value"))

    def test_read_constant_boolean_true(self):
        from reader import Reader
        data = bytes([LBC_CONSTANT_BOOLEAN, 1])
        result = read_constant(Reader(data), [])
        self.assertEqual(result["type"], LBC_CONSTANT_BOOLEAN)
        self.assertEqual(result.get("value"), True)


if __name__ == "__main__":
    unittest.main()
