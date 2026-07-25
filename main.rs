use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::time::Instant;

// LUAU CONSTANTS & OPCODES

const LBC_CONSTANT_NIL: u8 = 0;
const LBC_CONSTANT_BOOLEAN: u8 = 1;
const LBC_CONSTANT_NUMBER: u8 = 2;
const LBC_CONSTANT_STRING: u8 = 3;
const LBC_CONSTANT_IMPORT: u8 = 4;
const LBC_CONSTANT_TABLE: u8 = 5;
const LBC_CONSTANT_CLOSURE: u8 = 6;
const LBC_CONSTANT_VECTOR: u8 = 8;

#[derive(Debug, Clone)]
pub struct BytecodeOp {
    pub name: &'static str,
    pub op_type: &'static str,
    pub number: u8,
    pub aux: bool,
}

pub const OP_TABLE_V5: &[BytecodeOp] = &[
    BytecodeOp { name: "NOP", op_type: "none", number: 0x00, aux: false },
    BytecodeOp { name: "BREAK", op_type: "none", number: 0xE3, aux: false },
    BytecodeOp { name: "LOADNIL", op_type: "iA", number: 0xC6, aux: false },
    BytecodeOp { name: "LOADB", op_type: "iABC", number: 0xA9, aux: false },
    BytecodeOp { name: "LOADN", op_type: "iABx", number: 0x8C, aux: false },
    BytecodeOp { name: "LOADK", op_type: "iABx", number: 0x6F, aux: false },
    BytecodeOp { name: "MOVE", op_type: "iAB", number: 0x52, aux: false },
    BytecodeOp { name: "GETGLOBAL", op_type: "iAC", number: 0x35, aux: true },
    BytecodeOp { name: "SETGLOBAL", op_type: "iAC", number: 0x18, aux: true },
    BytecodeOp { name: "GETUPVAL", op_type: "iAB", number: 0xFB, aux: false },
    BytecodeOp { name: "SETUPVAL", op_type: "iAB", number: 0xDE, aux: false },
    BytecodeOp { name: "CLOSEUPVALS", op_type: "iA", number: 0xC1, aux: false },
    BytecodeOp { name: "GETIMPORT", op_type: "iABx", number: 0xA4, aux: true },
    BytecodeOp { name: "GETTABLE", op_type: "iABC", number: 0x87, aux: false },
    BytecodeOp { name: "SETTABLE", op_type: "iABC", number: 0x6A, aux: false },
    BytecodeOp { name: "GETTABLEKS", op_type: "iABC", number: 0x4D, aux: true },
    BytecodeOp { name: "SETTABLEKS", op_type: "iABC", number: 0x30, aux: true },
    BytecodeOp { name: "GETTABLEN", op_type: "iABC", number: 0x13, aux: false },
    BytecodeOp { name: "SETTABLEN", op_type: "iABC", number: 0xF6, aux: false },
    BytecodeOp { name: "NEWCLOSURE", op_type: "iABx", number: 0xD9, aux: false },
    BytecodeOp { name: "NAMECALL", op_type: "iABC", number: 0xBC, aux: true },
    BytecodeOp { name: "CALL", op_type: "iABC", number: 0x9F, aux: false },
    BytecodeOp { name: "RETURN", op_type: "iAB", number: 0x82, aux: false },
    BytecodeOp { name: "JUMP", op_type: "isBx", number: 0x65, aux: false },
    BytecodeOp { name: "JUMPBACK", op_type: "isBx", number: 0x48, aux: false },
    BytecodeOp { name: "JUMPIF", op_type: "iAsBx", number: 0x2B, aux: false },
    BytecodeOp { name: "JUMPIFNOT", op_type: "iAsBx", number: 0x0E, aux: false },
    BytecodeOp { name: "JUMPIFEQ", op_type: "iAsBx", number: 0xF1, aux: true },
    BytecodeOp { name: "JUMPIFLE", op_type: "iAsBx", number: 0xD4, aux: true },
    BytecodeOp { name: "JUMPIFLT", op_type: "iAsBx", number: 0xB7, aux: true },
    BytecodeOp { name: "JUMPIFNOTEQ", op_type: "iAsBx", number: 0x9A, aux: true },
    BytecodeOp { name: "JUMPIFNOTLE", op_type: "iAsBx", number: 0x7D, aux: true },
    BytecodeOp { name: "JUMPIFNOTLT", op_type: "iAsBx", number: 0x60, aux: true },
    BytecodeOp { name: "ADD", op_type: "iABC", number: 0x43, aux: false },
    BytecodeOp { name: "SUB", op_type: "iABC", number: 0x26, aux: false },
    BytecodeOp { name: "MUL", op_type: "iABC", number: 0x09, aux: false },
    BytecodeOp { name: "DIV", op_type: "iABC", number: 0xEC, aux: false },
    BytecodeOp { name: "MOD", op_type: "iABC", number: 0xCF, aux: false },
    BytecodeOp { name: "POW", op_type: "iABC", number: 0xB2, aux: false },
    BytecodeOp { name: "ADDK", op_type: "iABC", number: 0x95, aux: false },
    BytecodeOp { name: "SUBK", op_type: "iABC", number: 0x78, aux: false },
    BytecodeOp { name: "MULK", op_type: "iABC", number: 0x5B, aux: false },
    BytecodeOp { name: "DIVK", op_type: "iABC", number: 0x3E, aux: false },
    BytecodeOp { name: "MODK", op_type: "iABC", number: 0x21, aux: false },
    BytecodeOp { name: "POWK", op_type: "iABC", number: 0x04, aux: false },
    BytecodeOp { name: "AND", op_type: "iABC", number: 0xE7, aux: false },
    BytecodeOp { name: "OR", op_type: "iABC", number: 0xCA, aux: false },
    BytecodeOp { name: "ANDK", op_type: "iABC", number: 0xAD, aux: false },
    BytecodeOp { name: "ORK", op_type: "iABC", number: 0x90, aux: false },
    BytecodeOp { name: "CONCAT", op_type: "iABC", number: 0x73, aux: false },
    BytecodeOp { name: "NOT", op_type: "iAB", number: 0x56, aux: false },
    BytecodeOp { name: "MINUS", op_type: "iAB", number: 0x39, aux: false },
    BytecodeOp { name: "LENGTH", op_type: "iAB", number: 0x1C, aux: false },
    BytecodeOp { name: "NEWTABLE", op_type: "iAB", number: 0xFF, aux: true },
    BytecodeOp { name: "DUPTABLE", op_type: "iABx", number: 0xE2, aux: false },
    BytecodeOp { name: "SETLIST", op_type: "iABC", number: 0xC5, aux: true },
    BytecodeOp { name: "FORNPREP", op_type: "iABx", number: 0xA8, aux: false },
    BytecodeOp { name: "FORNLOOP", op_type: "iABx", number: 0x8B, aux: false },
    BytecodeOp { name: "FORGLOOP", op_type: "iABx", number: 0x6E, aux: true },
    BytecodeOp { name: "FORGPREP_INEXT", op_type: "none", number: 0x51, aux: false },
    BytecodeOp { name: "DEP_FORGLOOP_INEXT", op_type: "none", number: 0x34, aux: false },
    BytecodeOp { name: "FORGPREP_NEXT", op_type: "none", number: 0x17, aux: false },
    BytecodeOp { name: "NATIVECALL", op_type: "none", number: 0xFA, aux: false },
    BytecodeOp { name: "GETVARARGS", op_type: "iAB", number: 0xDD, aux: false },
    BytecodeOp { name: "DUPCLOSURE", op_type: "iABx", number: 0xC0, aux: false },
    BytecodeOp { name: "PREPVARARGS", op_type: "iA", number: 0xA3, aux: false },
    BytecodeOp { name: "LOADKX", op_type: "iA", number: 0x86, aux: false },
    BytecodeOp { name: "JUMPX", op_type: "isAx", number: 0x69, aux: false },
    BytecodeOp { name: "FASTCALL", op_type: "iAC", number: 0x4C, aux: false },
    BytecodeOp { name: "COVERAGE", op_type: "isAx", number: 0x2F, aux: false },
    BytecodeOp { name: "CAPTURE", op_type: "iAB", number: 0x12, aux: false },
    BytecodeOp { name: "SUBRK", op_type: "iABx", number: 0xF5, aux: true },
    BytecodeOp { name: "DIVRK", op_type: "iABx", number: 0xD8, aux: true },
    BytecodeOp { name: "FASTCALL1", op_type: "iABC", number: 0xBB, aux: false },
    BytecodeOp { name: "FASTCALL2", op_type: "iABC", number: 0x9E, aux: true },
    BytecodeOp { name: "FASTCALL2K", op_type: "iABC", number: 0x81, aux: true },
    BytecodeOp { name: "FORGPREP", op_type: "iAB", number: 0x64, aux: false },
    BytecodeOp { name: "JUMPXEQKNIL", op_type: "iAsAx", number: 0x47, aux: true },
    BytecodeOp { name: "JUMPXEQKB", op_type: "iAsAx", number: 0x2A, aux: true },
    BytecodeOp { name: "JUMPXEQKN", op_type: "iAsAx", number: 0x0D, aux: true },
    BytecodeOp { name: "JUMPXEQKS", op_type: "iAsAx", number: 0xF0, aux: true },
    BytecodeOp { name: "IDIV", op_type: "iABC", number: 0xD3, aux: false },
    BytecodeOp { name: "IDIVK", op_type: "iABC", number: 0xB6, aux: false },
    BytecodeOp { name: "COUNT", op_type: "none", number: 0x99, aux: false },
];

pub const OP_TABLE_V6: &[BytecodeOp] = &[
    BytecodeOp { name: "NOP", op_type: "none", number: 0x00, aux: false },
    BytecodeOp { name: "BREAK", op_type: "none", number: 0xE3, aux: false },
    BytecodeOp { name: "LOADNIL", op_type: "iA", number: 0xC6, aux: false },
    BytecodeOp { name: "LOADB", op_type: "iABC", number: 0xA9, aux: false },
    BytecodeOp { name: "LOADN", op_type: "iABx", number: 0x8C, aux: false },
    BytecodeOp { name: "LOADK", op_type: "iABx", number: 0x6F, aux: false },
    BytecodeOp { name: "MOVE", op_type: "iAB", number: 0x52, aux: false },
    BytecodeOp { name: "GETGLOBAL", op_type: "iAC", number: 0x35, aux: true },
    BytecodeOp { name: "SETGLOBAL", op_type: "iAC", number: 0x18, aux: true },
    BytecodeOp { name: "GETUPVAL", op_type: "iAB", number: 0xFB, aux: false },
    BytecodeOp { name: "SETUPVAL", op_type: "iAB", number: 0xDE, aux: false },
    BytecodeOp { name: "CLOSEUPVALS", op_type: "iA", number: 0xC1, aux: false },
    BytecodeOp { name: "GETIMPORT", op_type: "iABx", number: 0xA4, aux: true },
    BytecodeOp { name: "GETTABLE", op_type: "iABC", number: 0x87, aux: false },
    BytecodeOp { name: "SETTABLE", op_type: "iABC", number: 0x6A, aux: false },
    BytecodeOp { name: "GETTABLEKS", op_type: "iABC", number: 0x4D, aux: true },
    BytecodeOp { name: "SETTABLEKS", op_type: "iABC", number: 0x30, aux: true },
    BytecodeOp { name: "GETTABLEN", op_type: "iABC", number: 0x13, aux: false },
    BytecodeOp { name: "SETTABLEN", op_type: "iABC", number: 0xF6, aux: false },
    BytecodeOp { name: "NEWCLOSURE", op_type: "iABx", number: 0xD9, aux: false },
    BytecodeOp { name: "NAMECALL", op_type: "iABC", number: 0xBC, aux: true },
    BytecodeOp { name: "CALL", op_type: "iABC", number: 0x9F, aux: false },
    BytecodeOp { name: "RETURN", op_type: "iAB", number: 0x82, aux: false },
    BytecodeOp { name: "JUMP", op_type: "isBx", number: 0x65, aux: false },
    BytecodeOp { name: "JUMPBACK", op_type: "isBx", number: 0x48, aux: false },
    BytecodeOp { name: "JUMPIF", op_type: "iAsBx", number: 0x2B, aux: false },
    BytecodeOp { name: "JUMPIFNOT", op_type: "iAsBx", number: 0x0E, aux: false },
    BytecodeOp { name: "JUMPIFEQ", op_type: "iAsBx", number: 0xF1, aux: true },
    BytecodeOp { name: "JUMPIFLE", op_type: "iAsBx", number: 0xD4, aux: true },
    BytecodeOp { name: "JUMPIFLT", op_type: "iAsBx", number: 0xB7, aux: true },
    BytecodeOp { name: "JUMPIFNOTEQ", op_type: "iAsBx", number: 0x9A, aux: true },
    BytecodeOp { name: "JUMPIFNOTLE", op_type: "iAsBx", number: 0x7D, aux: true },
    BytecodeOp { name: "JUMPIFNOTLT", op_type: "iAsBx", number: 0x60, aux: true },
    BytecodeOp { name: "ADD", op_type: "iABC", number: 0x43, aux: false },
    BytecodeOp { name: "SUB", op_type: "iABC", number: 0x26, aux: false },
    BytecodeOp { name: "MUL", op_type: "iABC", number: 0x09, aux: false },
    BytecodeOp { name: "DIV", op_type: "iABC", number: 0xEC, aux: false },
    BytecodeOp { name: "MOD", op_type: "iABC", number: 0xCF, aux: false },
    BytecodeOp { name: "POW", op_type: "iABC", number: 0xB2, aux: false },
    BytecodeOp { name: "ADDK", op_type: "iABC", number: 0x95, aux: false },
    BytecodeOp { name: "SUBK", op_type: "iABC", number: 0x78, aux: false },
    BytecodeOp { name: "MULK", op_type: "iABC", number: 0x5B, aux: false },
    BytecodeOp { name: "DIVK", op_type: "iABC", number: 0x3E, aux: false },
    BytecodeOp { name: "MODK", op_type: "iABC", number: 0x21, aux: false },
    BytecodeOp { name: "POWK", op_type: "iABC", number: 0x04, aux: false },
    BytecodeOp { name: "AND", op_type: "iABC", number: 0xE7, aux: false },
    BytecodeOp { name: "OR", op_type: "iABC", number: 0xCA, aux: false },
    BytecodeOp { name: "ANDK", op_type: "iABC", number: 0xAD, aux: false },
    BytecodeOp { name: "ORK", op_type: "iABC", number: 0x90, aux: false },
    BytecodeOp { name: "CONCAT", op_type: "iABC", number: 0x73, aux: false },
    BytecodeOp { name: "NOT", op_type: "iAB", number: 0x56, aux: false },
    BytecodeOp { name: "MINUS", op_type: "iAB", number: 0x39, aux: false },
    BytecodeOp { name: "LENGTH", op_type: "iAB", number: 0x1C, aux: false },
    BytecodeOp { name: "NEWTABLE", op_type: "iAB", number: 0xFF, aux: true },
    BytecodeOp { name: "DUPTABLE", op_type: "iABx", number: 0xE2, aux: false },
    BytecodeOp { name: "SETLIST", op_type: "iABC", number: 0xC5, aux: true },
    BytecodeOp { name: "FORNPREP", op_type: "iABx", number: 0xA8, aux: false },
    BytecodeOp { name: "FORNLOOP", op_type: "iABx", number: 0x8B, aux: false },
    BytecodeOp { name: "FORGLOOP", op_type: "iABx", number: 0x6E, aux: true },
    BytecodeOp { name: "FORGPREP_INEXT", op_type: "none", number: 0x51, aux: false },
    BytecodeOp { name: "FORGPREP_NEXT", op_type: "none", number: 0x17, aux: false },
    BytecodeOp { name: "NATIVECALL", op_type: "none", number: 0xFA, aux: false },
    BytecodeOp { name: "GETVARARGS", op_type: "iAB", number: 0xDD, aux: false },
    BytecodeOp { name: "DUPCLOSURE", op_type: "iABx", number: 0xC0, aux: false },
    BytecodeOp { name: "PREPVARARGS", op_type: "iA", number: 0xA3, aux: false },
    BytecodeOp { name: "LOADKX", op_type: "iA", number: 0x86, aux: false },
    BytecodeOp { name: "JUMPX", op_type: "isAx", number: 0x69, aux: false },
    BytecodeOp { name: "FASTCALL", op_type: "iAC", number: 0x4C, aux: false },
    BytecodeOp { name: "COVERAGE", op_type: "isAx", number: 0x2F, aux: false },
    BytecodeOp { name: "CAPTURE", op_type: "iAB", number: 0x12, aux: false },
    BytecodeOp { name: "SUBRK", op_type: "iABx", number: 0xF5, aux: true },
    BytecodeOp { name: "DIVRK", op_type: "iABx", number: 0xD8, aux: true },
    BytecodeOp { name: "FASTCALL1", op_type: "iABC", number: 0xBB, aux: false },
    BytecodeOp { name: "FASTCALL2", op_type: "iABC", number: 0x9E, aux: true },
    BytecodeOp { name: "FASTCALL2K", op_type: "iABC", number: 0x81, aux: true },
    BytecodeOp { name: "FORGPREP", op_type: "iAB", number: 0x64, aux: false },
    BytecodeOp { name: "JUMPXEQKNIL", op_type: "iAsAx", number: 0x47, aux: true },
    BytecodeOp { name: "JUMPXEQKB", op_type: "iAsAx", number: 0x2A, aux: true },
    BytecodeOp { name: "JUMPXEQKN", op_type: "iAsAx", number: 0x0D, aux: true },
    BytecodeOp { name: "JUMPXEQKS", op_type: "iAsAx", number: 0xF0, aux: true },
    BytecodeOp { name: "IDIV", op_type: "iABC", number: 0xD3, aux: false },
    BytecodeOp { name: "IDIVK", op_type: "iABC", number: 0xB6, aux: false },
    BytecodeOp { name: "FASTCALL3", op_type: "iABC", number: 0x34, aux: true },
    BytecodeOp { name: "COUNT", op_type: "none", number: 0x99, aux: false },
];

fn get_op_table(version: u8) -> &'static [BytecodeOp] {
    match version {
        5 => OP_TABLE_V5,
        6 => OP_TABLE_V6,
        _ => OP_TABLE_V5,
    }
}

// Bit manipulation functions
fn get_opcode(i: u32) -> u8 {
    ((i.wrapping_mul(227)) & 0xFF) as u8
}
fn get_arg_a(i: u32) -> u8 { ((i >> 8) & 0xFF) as u8 }
fn get_arg_b(i: u32) -> u8 { ((i >> 16) & 0xFF) as u8 }
fn get_arg_c(i: u32) -> u8 { ((i >> 24) & 0xFF) as u8 }
fn get_arg_bx(i: u32) -> u32 { i >> 16 }
fn get_arg_sbx(i: u32) -> i32 {
    let d = (i >> 16) & 0xFFFF;
    if d >= 0x8000 { (d - 0x10000) as i32 } else { d as i32 }
}
fn get_arg_sax(i: u32) -> i32 {
    let d = i >> 8;
    if d & 0x800000 != 0 { (d - 0x1000000) as i32 } else { d as i32 }
}

// READER

pub struct Reader<'a> {
    bytecode: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(bytecode: &'a [u8]) -> Self {
        Reader { bytecode, pos: 0 }
    }

    pub fn can_read(&self, n: usize) -> bool {
        self.pos + n <= self.bytecode.len()
    }

    pub fn next_byte(&mut self) -> u8 {
        if !self.can_read(1) {
            panic!("Attempted to read byte at position {}, but bytecode length is {}", self.pos, self.bytecode.len());
        }
        let value = self.bytecode[self.pos];
        self.pos += 1;
        value
    }

    pub fn next_var_int(&mut self) -> usize {
        let mut result = 0;
        let mut shift = 0;
        for _ in 0..5 {
            if !self.can_read(1) {
                panic!("Unexpected end of bytecode while reading VarInt at position {}", self.pos);
            }
            let b = self.next_byte();
            result |= ((b & 0x7F) as usize) << shift;
            if (b & 0x80) == 0 {
                return result;
            }
            shift += 7;
        }
        panic!("VarInt at position {} is too long (max 5 bytes)", self.pos);
    }

    pub fn next_string(&mut self) -> String {
        let length = self.next_var_int();
        if !self.can_read(length) {
            panic!("Attempted to read string of length {} at position {}", length, self.pos);
        }
        let result = String::from_utf8_lossy(&self.bytecode[self.pos..self.pos + length]).to_string();
        self.pos += length;
        result
    }

    pub fn next_float(&mut self) -> f32 {
        if !self.can_read(4) {
            panic!("Unexpected end of bytecode while reading float");
        }
        let bytes: [u8; 4] = self.bytecode[self.pos..self.pos + 4].try_into().unwrap();
        self.pos += 4;
        f32::from_le_bytes(bytes)
    }

    pub fn next_double(&mut self) -> f64 {
        if !self.can_read(8) {
            panic!("Unexpected end of bytecode while reading double");
        }
        let bytes: [u8; 8] = self.bytecode[self.pos..self.pos + 8].try_into().unwrap();
        self.pos += 8;
        f64::from_le_bytes(bytes)
    }

    pub fn next_int(&mut self) -> u32 {
        let b = [self.next_byte(), self.next_byte(), self.next_byte(), self.next_byte()];
        ((b[3] as u32) << 24) | ((b[2] as u32) << 16) | ((b[1] as u32) << 8) | (b[0] as u32)
    }
}

// DATA STRUCTURES

#[derive(Debug, Clone)]
pub enum ConstantValue {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Import(i32),
    Table { size: usize, ids: Vec<usize> },
    Closure(usize),
    Vector([f32; 4]),
}

#[derive(Debug, Clone)]
pub struct Constant {
    pub value: ConstantValue,
}

#[derive(Debug, Clone)]
pub struct VarInfo {
    pub name: String,
    pub start_pc: usize,
    pub end_pc: usize,
    pub reg: u8,
}

#[derive(Debug, Clone)]
pub struct DebugInfo {
    pub var_info: Vec<VarInfo>,
    pub upvalue_info: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Proto {
    pub max_stack_size: u8,
    pub num_params: u8,
    pub num_upvalues: u8,
    pub is_var_arg: bool,
    pub flags: u8,
    pub type_info: Vec<u8>,
    pub code_table: Vec<u32>,
    pub k_table: Vec<Constant>,
    pub p_table: Vec<usize>,
    pub line_defined: usize,
    pub source: String,
    pub debug_info: Option<DebugInfo>,
}

// DESERIALIZATION

fn read_proto_source(reader: &mut Reader, string_table: &[String]) -> String {
    let proto_source_id = reader.next_var_int();
    if proto_source_id > 0 && proto_source_id - 1 < string_table.len() {
        string_table[proto_source_id - 1].clone()
    } else {
        "Invalid source index".to_string()
    }
}

fn read_constant(reader: &mut Reader, string_table: &[String]) -> Constant {
    let const_type = reader.next_byte();
    let value = match const_type {
        LBC_CONSTANT_NIL => ConstantValue::Nil,
        LBC_CONSTANT_BOOLEAN => ConstantValue::Boolean(reader.next_byte() == 1),
        LBC_CONSTANT_NUMBER => ConstantValue::Number(reader.next_double()),
        LBC_CONSTANT_STRING => {
            let raw_index = reader.next_var_int();
            let index = raw_index.wrapping_sub(1);
            ConstantValue::String(if index < string_table.len() {
                string_table[index].clone()
            } else {
                "Invalid string index".to_string()
            })
        }
        LBC_CONSTANT_IMPORT => ConstantValue::Import(reader.next_int() as i32),
        LBC_CONSTANT_TABLE => {
            let size = reader.next_var_int();
            let ids: Vec<usize> = (0..size).map(|_| reader.next_var_int()).collect();
            ConstantValue::Table { size, ids }
        }
        LBC_CONSTANT_CLOSURE => ConstantValue::Closure(reader.next_var_int() + 1),
        LBC_CONSTANT_VECTOR => ConstantValue::Vector([
            reader.next_float(),
            reader.next_float(),
            reader.next_float(),
            reader.next_float(),
        ]),
        _ => panic!("Unrecognized constant type: {}", const_type),
    };
    Constant { value }
}

fn read_proto_data(reader: &mut Reader, string_table: &[String]) -> Proto {
    let max_stack_size = reader.next_byte();
    let num_params = reader.next_byte();
    let num_upvalues = reader.next_byte();
    let is_var_arg = reader.next_byte() == 1;
    let flags = reader.next_byte();

    let type_size = reader.next_var_int();
    let type_info: Vec<u8> = (0..type_size).map(|_| reader.next_byte()).collect();

    let size_code = reader.next_var_int();
    let code_table: Vec<u32> = (0..size_code).map(|_| reader.next_int()).collect();

    let size_consts = reader.next_var_int();
    let k_table: Vec<Constant> = (0..size_consts).map(|_| read_constant(reader, string_table)).collect();

    let size_protos = reader.next_var_int();
    let p_table: Vec<usize> = (0..size_protos).map(|_| reader.next_var_int()).collect();

    let line_defined = reader.next_var_int();
    let source = read_proto_source(reader, string_table);

    let has_line_info = reader.next_byte() == 1;
    if has_line_info {
        let comp_key = reader.next_byte();
        for _ in 0..size_code { reader.next_byte(); }
        let intervals = ((size_code - 1) >> comp_key) + 1;
        for _ in 0..intervals { reader.next_int(); }
    }

    let mut debug_info = None;
    let has_debug_info = reader.next_byte() == 1;
    if has_debug_info {
        let mut var_info = Vec::new();
        let size_vars = reader.next_var_int();
        for _ in 0..size_vars {
            let var_name_idx = reader.next_var_int().wrapping_sub(1);
            let var_name = if var_name_idx < string_table.len() {
                string_table[var_name_idx].clone()
            } else {
                format!("<var {}>", var_name_idx)
            };
            let start_pc = reader.next_var_int();
            let end_pc = reader.next_var_int();
            let reg = reader.next_byte();
            var_info.push(VarInfo { name: var_name, start_pc, end_pc, reg });
        }

        let mut upvalue_info = Vec::new();
        let size_upvalues = reader.next_var_int();
        for _ in 0..size_upvalues {
            let uv_name_idx = reader.next_var_int().wrapping_sub(1);
            let uv_name = if uv_name_idx < string_table.len() {
                string_table[uv_name_idx].clone()
            } else {
                format!("<upvalue {}>", uv_name_idx)
            };
            upvalue_info.push(uv_name);
        }

        debug_info = Some(DebugInfo { var_info, upvalue_info });
    }

    Proto {
        max_stack_size,
        num_params,
        num_upvalues,
        is_var_arg,
        flags,
        type_info,
        code_table,
        k_table,
        p_table,
        line_defined,
        source,
        debug_info,
    }
}

fn deserialize(bytecode: &[u8]) -> (Proto, Vec<Proto>, Vec<String>, u8, u8) {
    let mut reader = Reader::new(bytecode);
    let version = reader.next_byte();
    match version {
        5 => deserialize_v5(&mut reader),
        6 => deserialize_v6(&mut reader),
        _ => panic!("Unsupported bytecode version: {}", version),
    }
}

fn deserialize_v5(reader: &mut Reader) -> (Proto, Vec<Proto>, Vec<String>, u8, u8) {
    let types_version = reader.next_byte();
    if types_version < 1 || types_version > 3 {
        panic!("Invalid types version: {}", types_version);
    }

    let size_strings = reader.next_var_int();
    let string_table: Vec<String> = (0..size_strings).map(|_| reader.next_string()).collect();

    if types_version >= 3 {
        let mut index = reader.next_byte();
        while index != 0 { index = reader.next_byte(); }
    }

    let size_protos = reader.next_var_int();
    let mut proto_table: Vec<Proto> = Vec::with_capacity(size_protos);
    for _ in 0..size_protos {
        proto_table.push(read_proto_data(reader, &string_table));
    }

    let main_proto_id = reader.next_var_int();
    if main_proto_id >= proto_table.len() {
        panic!("Main proto index out of range");
    }

    let main_proto = proto_table[main_proto_id].clone();
    (main_proto, proto_table, string_table, 5, types_version)
}

fn deserialize_v6(reader: &mut Reader) -> (Proto, Vec<Proto>, Vec<String>, u8, u8) {
    let types_version = reader.next_byte();
    if types_version < 1 || types_version > 3 {
        panic!("Invalid types version: {}", types_version);
    }

    let size_strings = reader.next_var_int();
    let string_table: Vec<String> = (0..size_strings).map(|_| reader.next_string()).collect();

    if types_version >= 3 {
        let mut index = reader.next_byte();
        while index != 0 { index = reader.next_byte(); }
    }

    let size_protos = reader.next_var_int();
    let mut proto_table: Vec<Proto> = Vec::with_capacity(size_protos);
    for _ in 0..size_protos {
        proto_table.push(read_proto_data(reader, &string_table));
    }

    let main_proto_id = reader.next_var_int();
    if main_proto_id >= proto_table.len() {
        panic!("Main proto index out of range");
    }

    let main_proto = proto_table[main_proto_id].clone();
    (main_proto, proto_table, string_table, 6, types_version)
}

// DISASSEMBLY & DECOMPILATION HELPERS

fn fmt_bool(b: bool) -> &'static str {
    if b { "true" } else { "false" }
}

fn format_constant(k: &Constant) -> String {
    match &k.value {
        ConstantValue::Nil => "nil".to_string(),
        ConstantValue::Boolean(b) => fmt_bool(*b).to_string(),
        ConstantValue::Number(n) => n.to_string(),
        ConstantValue::String(s) => format!("{:?}", s),
        ConstantValue::Vector(v) => format!("vector({}, {}, {}, {})", v[0], v[1], v[2], v[3]),
        ConstantValue::Table { size, ids } => format!("table<size={},ids={:?}>", size, ids),
        ConstantValue::Closure(c) => format!("closure({})", c),
        ConstantValue::Import(i) => format!("import<{}>", i),
    }
}

fn decompose_import_id(ids: i32) -> Vec<i32> {
    let count = ids >> 30;
    let mut res = vec![];
    if count > 0 { res.push((ids >> 20) & 1023); }
    if count > 1 { res.push((ids >> 10) & 1023); }
    if count > 2 { res.push(ids & 1023); }
    res
}

fn import_id_to_name(proto: &Proto, ids: i32) -> String {
    if ids == 0 { return "0".to_string(); }
    let mut imported_path = String::new();
    let id_constants = decompose_import_id(ids);
    for (i, id_constant) in id_constants.iter().enumerate() {
        if (*id_constant as usize) < proto.k_table.len() {
            let entry = &proto.k_table[*id_constant as usize];
            let name = if let ConstantValue::String(s) = &entry.value {
                s.clone()
            } else {
                format!("<const {}>", id_constant)
            };
            if i > 0 { imported_path.push('.'); }
            imported_path.push_str(&name);
        } else {
            imported_path.push_str(&format!("<const {}>", id_constant));
        }
    }
    imported_path
}

// DISASSEMBLER (read_proto)

fn read_proto(proto: &Proto, depth: usize, proto_table: &[Proto], _string_table: &[String], luau_version: u8) -> String {
    let op_table = get_op_table(luau_version);
    let mut output = String::new();
    let tab_space = "    ".repeat(depth - 1);

    let params: Vec<String> = (0..proto.num_params).map(|i| format!("R{}", i)).collect();
    let mut params = params;
    if proto.is_var_arg { params.push("...".to_string()); }

    output.push_str(&format!("{}function({})\n", tab_space, params.join(", ")));

    let opcode_to_opname: HashMap<u8, &str> = op_table.iter().map(|op| (op.number, op.name)).collect();
    let max_opname_length = op_table.iter().map(|op| op.name.len()).max().unwrap_or(0);

    let mut code_index = 0;
    while code_index < proto.code_table.len() {
        let i = proto.code_table[code_index];
        let opc = get_opcode(i);
        let a = get_arg_a(i);
        let b = get_arg_b(i);
        let bx = get_arg_bx(i);
        let c = get_arg_c(i);
        let s_bx = get_arg_sbx(i);
        let s_ax = get_arg_sax(i);

        let op_name = opcode_to_opname.get(&opc).copied().unwrap_or("UNKNOWN");
        output.push_str(&format!("{}[{:03}] {:<width$} ", "    ".repeat(depth), code_index, op_name, width = max_opname_length));

        let mut aux = None;
        if let Some(info) = op_table.iter().find(|op| op.name == op_name) {
            if info.aux && code_index + 1 < proto.code_table.len() {
                aux = Some(proto.code_table[code_index + 1]);
                code_index += 1;
            }
        }

        let aux_val = aux.unwrap_or(0);
        let res = match op_name {
            "NOP" => "-- do nothing (no-op / NOP)".to_string(),
            "BREAK" => "break".to_string(),
            "PREPVARARGS" => format!("(adjust vararg params, {} fixed params)", a),
            "LOADNIL" => format!("R{} = nil", a),
            "LOADB" => if c != 0 {
                format!("R{} = {}; goto [{}]", a, fmt_bool(b != 0), code_index + 1 + c as usize)
            } else {
                format!("R{} = {}", a, fmt_bool(b != 0))
            },
            "LOADN" => format!("R{} = {}", a, s_bx),
            "LOADK" => if (bx as usize) < proto.k_table.len() {
                format!("R{} = {}", a, format_constant(&proto.k_table[bx as usize]))
            } else {
                format!("R{} = K{}", a, bx)
            },
            "MOVE" => format!("R{} = R{}", a, b),
            "GETGLOBAL" => if (aux_val as usize) < proto.k_table.len() {
                format!("R{} = _G[{}]", a, format_constant(&proto.k_table[aux_val as usize]))
            } else {
                format!("R{} = _G[Invalid constant index]", a)
            },
            "SETGLOBAL" => if (aux_val as usize) < proto.k_table.len() {
                format!("_G[{}] = R{}", format_constant(&proto.k_table[aux_val as usize]), a)
            } else {
                format!("_G[Invalid constant index] = R{}", a)
            },
            "GETUPVAL" => format!("R{} = U{}", a, b),
            "SETUPVAL" => format!("U{} = R{}", b, a),
            "CLOSEUPVALS" => format!("close upvalues R{}+", a),
            "GETIMPORT" => {
                if (bx as usize) < proto.k_table.len() {
                    if let ConstantValue::Import(import_id) = &proto.k_table[bx as usize].value {
                        let path = import_id_to_name(proto, *import_id);
                        format!("R{} = {} -- Import ID: {}", a, path, import_id)
                    } else { format!("R{} = <invalid import type>", a) }
                } else { format!("R{} = <invalid import index>", a) }
            },
            "GETTABLE" => format!("R{} = R{}[R{}]", a, b, c),
            "SETTABLE" => format!("R{}[R{}] = R{}", b, c, a),
            "GETTABLEKS" => if (aux_val as usize) < proto.k_table.len() {
                format!("R{} = R{}[{}]", a, b, format_constant(&proto.k_table[aux_val as usize]))
            } else {
                format!("R{} = R{}[Invalid constant index]", a, b)
            },
            "SETTABLEKS" => if (aux_val as usize) < proto.k_table.len() {
                format!("R{}[{}] = R{}", b, format_constant(&proto.k_table[aux_val as usize]), a)
            } else {
                format!("R{}[Invalid constant index] = R{}", b, a)
            },
            "GETTABLEN" => format!("R{} = R{}[{}]", a, b, c + 1),
            "SETTABLEN" => format!("R{}[{}] = R{}", b, c + 1, a),
            "NEWCLOSURE" => format!("R{} = closure(proto[{}])", a, bx),
            "NAMECALL" => if (aux_val as usize) < proto.k_table.len() {
                format!("R{} = R{}[{}]; R{} = R{}", a, b, format_constant(&proto.k_table[aux_val as usize]), a + 1, b)
            } else {
                format!("R{} = R{}[Invalid constant index]; R{} = R{}", a, b, a + 1, b)
            },
            "CALL" => {
                let args = if b == 1 { "".to_string() } else if b == 0 { format!("R{} ...", a + 1) } else { format!("R{} ... R{}", a + 1, a + b - 1) };
                let rets = if c == 1 { "".to_string() } else if c == 0 { format!("R{} ...", a) } else { format!("R{} ... R{}", a, a + c - 1) };
                if rets.is_empty() { format!("R{}({})", a, args) } else { format!("{} = R{}({})", rets, a, args) }
            },
            "RETURN" => if b == 0 { format!("return R{} ...", a) } else if b == 1 { "return".to_string() } else { format!("return R{} ... R{}", a, a + b - 2) },
            "JUMP" => format!("goto [{}]", code_index as i64 + 1 + s_bx as i64),
            "JUMPBACK" => format!("goto [{}]", code_index as i64 + 1 + s_bx as i64),
            "JUMPX" => format!("goto [{}]", code_index as i64 + 1 + s_ax as i64),
            "JUMPIF" => format!("if R{} then goto [{}]", a, code_index as i64 + 1 + s_bx as i64),
            "JUMPIFNOT" => format!("if not R{} then goto [{}]", a, code_index as i64 + 1 + s_bx as i64),
            "JUMPIFEQ" => format!("if R{} == R{} then goto [{}]", a, aux_val, code_index as i64 + 1 + s_bx as i64),
            "JUMPIFLE" => format!("if R{} <= R{} then goto [{}]", a, aux_val, code_index as i64 + 1 + s_bx as i64),
            "JUMPIFLT" => format!("if R{} < R{} then goto [{}]", a, aux_val, code_index as i64 + 1 + s_bx as i64),
            "JUMPIFNOTEQ" => format!("if R{} ~= R{} then goto [{}]", a, aux_val, code_index as i64 + 1 + s_bx as i64),
            "JUMPIFNOTLE" => format!("if R{} > R{} then goto [{}]", a, aux_val, code_index as i64 + 1 + s_bx as i64),
            "JUMPIFNOTLT" => format!("if R{} >= R{} then goto [{}]", a, aux_val, code_index as i64 + 1 + s_bx as i64),
            "ADD" | "SUB" | "MUL" | "DIV" | "MOD" | "POW" | "IDIV" => {
                let op = match op_name { "ADD" => "+", "SUB" => "-", "MUL" => "*", "DIV" => "/", "MOD" => "%", "POW" => "^", "IDIV" => "//", _ => unreachable!() };
                format!("R{} = R{} {} R{}", a, b, op, c)
            }
            "ADDK" | "SUBK" | "MULK" | "DIVK" | "MODK" | "POWK" | "IDIVK" => {
                let op = match op_name { "ADDK" => "+", "SUBK" => "-", "MULK" => "*", "DIVK" => "/", "MODK" => "%", "POWK" => "^", "IDIVK" => "//", _ => unreachable!() };
                let k = if (c as usize) < proto.k_table.len() { format_constant(&proto.k_table[c as usize]) } else { format!("K{}", c) };
                format!("R{} = R{} {} {}", a, b, op, k)
            }
            "SUBRK" | "DIVRK" => {
                let op = if op_name == "SUBRK" { "-" } else { "/" };
                let k = if (b as usize) < proto.k_table.len() { format_constant(&proto.k_table[b as usize]) } else { format!("K{}", b) };
                format!("R{} = {} {} R{}", a, k, op, c)
            }
            "AND" | "OR" => {
                let op = if op_name == "AND" { "and" } else { "or" };
                format!("R{} = R{} {} R{}", a, b, op, c)
            }
            "ANDK" | "ORK" => {
                let op = if op_name == "ANDK" { "and" } else { "or" };
                let k = if (c as usize) < proto.k_table.len() { format_constant(&proto.k_table[c as usize]) } else { format!("K{}", c) };
                format!("R{} = R{} {} {}", a, b, op, k)
            }
            "CONCAT" => format!("R{} = R{} .. R{}", a, b, c),
            "NOT" => format!("R{} = not R{}", a, b),
            "MINUS" => format!("R{} = -R{}", a, b),
            "LENGTH" => format!("R{} = #R{}", a, b),
            "NEWTABLE" => format!("R{} = {{}} -- hash={}, array={}", a, if b == 0 { 0 } else { 1 << (b - 1) }, aux_val),
            "DUPTABLE" => format!("R{} = K{} -- duplicate", a, bx),
            "SETLIST" => if c > 0 {
                format!("R{}[{}..{}] = R{} ... R{}", a, aux_val, aux_val + c as u32 - 1, b, b + c - 1)
            } else {
                format!("R{}[{}..] = R{} ... top", a, aux_val, b)
            },
            "FORNPREP" | "FORNLOOP" | "FORGPREP" | "FORGPREP_INEXT" | "FORGPREP_NEXT" => format!("... goto [{}]", code_index as i64 + 1 + s_bx as i64),
            "FORGLOOP" => format!("R{}, ..., R{} = R{}(R{}, R{}); if R{} ~= nil then R{} = R{}; goto [{}]", a+3, a+2+(aux_val & 0x7F), a, a+1, a+2, a+3, a+2, a+3, code_index as i64 + 1 + s_bx as i64),
            "NATIVECALL" => "Unimplemented".to_string(),
            "GETVARARGS" => if b == 0 { format!("R{}, ... = ...", a) } else { format!("R{}, ..., R{} = ...", a, a + b - 2) },
            "DUPCLOSURE" => format!("R{} = K{} -- duplicate", a, bx),
            "LOADKX" => if (aux_val as usize) < proto.k_table.len() {
                format!("R{} = {}", a, format_constant(&proto.k_table[aux_val as usize]))
            } else {
                format!("R{} = <invalid constant>", a)
            },
            "FASTCALL" => format!("R{} = builtin[{}]", a, c),
            "FASTCALL1" => format!("R{} = builtin[{}](R{})", a, c, b),
            "FASTCALL2" => format!("R{} = builtin[{}](R{}, R{})", a, c, b, aux_val),
            "FASTCALL2K" => format!("R{} = builtin[{}](R{}, K{})", a, c, b, aux_val),
            "FASTCALL3" => format!("R{} = builtin[{}]", a, c),
            "COVERAGE" => "(coverage)".to_string(),
            "CAPTURE" => {
                let capture_types = ["VAL", "REF", "UPVAL"];
                let cap_type = if (a as usize) < capture_types.len() { capture_types[a as usize] } else { "Unknown" };
                format!("capture {} R{}", cap_type, b)
            }
            "JUMPXEQKNIL" => format!("if R{} == nil then goto [{}]", a, code_index as i64 + 1 + s_ax as i64),
            "JUMPXEQKB" => format!("if R{} {} {} then goto [{}]", a, if aux_val & 0x80000000 != 0 { "~=" } else { "==" }, fmt_bool(aux_val & 1 != 0), code_index as i64 + 1 + s_ax as i64),
            "JUMPXEQKN" => {
                let k_idx = (aux_val & 0x7FFFFFFF) as usize;
                let val = if k_idx < proto.k_table.len() { format_constant(&proto.k_table[k_idx]) } else { format!("K{}", k_idx) };
                format!("if R{} {} {} then goto [{}]", a, if aux_val & 0x80000000 != 0 { "~=" } else { "==" }, val, code_index as i64 + 1 + s_ax as i64)
            }
            "JUMPXEQKS" => {
                let k_idx = (aux_val & 0x7FFFFFFF) as usize;
                let val = if k_idx < proto.k_table.len() { format_constant(&proto.k_table[k_idx]) } else { format!("K{}", k_idx) };
                format!("if R{} {} {} then goto [{}]", a, if aux_val & 0x80000000 != 0 { "~=" } else { "==" }, val, code_index as i64 + 1 + s_ax as i64)
            }
            _ => format!("Unknown opcode: {}", opc),
        };
        output.push_str(&res);
        output.push('\n');
        code_index += 1;
    }

    output.push_str("end\n");

    if !proto.k_table.is_empty() {
        output.push_str("--< Constants >--\n");
        for (i, k) in proto.k_table.iter().enumerate() {
            output.push_str(&format!("{}[{}] = {}\n", "    ".repeat(depth), i, format_constant(k)));
        }
    }

    if !proto.p_table.is_empty() {
        output.push_str("--< Protos >--\n");
        for (i, proto_idx) in proto.p_table.iter().enumerate() {
            if *proto_idx < proto_table.len() {
                let child_proto = &proto_table[*proto_idx];
                output.push_str(&format!("{}[{}] = {}\n", "    ".repeat(depth), i, read_proto(child_proto, depth + 1, proto_table, _string_table, luau_version)));
            } else {
                output.push_str(&format!("{}[{}] = <invalid proto index {}>\n", "    ".repeat(depth), i, proto_idx));
            }
        }
    }

    if proto.num_upvalues > 0 {
        output.push_str("--< Upvalues >--\n");
        for i in 0..proto.num_upvalues {
            output.push_str(&format!("{}[{}] = Upvalue {}\n", "    ".repeat(depth), i, i));
        }
    }

    output
}

// DECOMPILER (decompile)

fn decompile(proto: &Proto, depth: usize, _string_table: &[String], luau_version: u8, proto_table: &[Proto]) -> String {
    let op_table = get_op_table(luau_version);
    let mut output: Vec<String> = Vec::new();
    let tab = "    ".repeat(depth + 1);

    let mut reg_names: HashMap<u8, String> = HashMap::new();
    let mut uv_names: HashMap<u8, String> = HashMap::new();

    if let Some(di) = &proto.debug_info {
        for var in &di.var_info {
            reg_names.insert(var.reg, var.name.clone());
        }
        for (i, uv) in di.upvalue_info.iter().enumerate() {
            uv_names.insert(i as u8, uv.clone());
        }
    }

    let rn = |r: u8| reg_names.get(&r).cloned().unwrap_or(format!("R{}", r));
    let un = |u: u8| uv_names.get(&u).cloned().unwrap_or(format!("U{}", u));

    // Function Signature
    let mut params: Vec<String> = Vec::new();
    if let Some(di) = &proto.debug_info {
        for var in &di.var_info {
            if var.start_pc == 0 && params.len() < proto.num_params as usize {
                params.push(var.name.clone());
            }
        }
    }
    while params.len() < proto.num_params as usize {
        params.push(format!("R{}", params.len()));
    }
    if proto.is_var_arg {
        params.push("...".to_string());
    }
    output.push(format!("local function func{}({})", depth, params.join(", ")));

    let opcode_to_opname: HashMap<u8, &str> = op_table.iter().map(|op| (op.number, op.name)).collect();

    let mut code_index = 0;
    while code_index < proto.code_table.len() {
        let i = proto.code_table[code_index];
        let opc = get_opcode(i);
        let opname = opcode_to_opname.get(&opc).copied().unwrap_or("UNKNOWN");
        let a = get_arg_a(i);
        let b = get_arg_b(i);
        let bx = get_arg_bx(i);
        let c = get_arg_c(i);
        let s_bx = get_arg_sbx(i);
        let s_ax = get_arg_sax(i);

        let mut aux = None;
        if let Some(info) = op_table.iter().find(|op| op.name == opname) {
            if info.aux && code_index + 1 < proto.code_table.len() {
                aux = Some(proto.code_table[code_index + 1]);
                code_index += 1;
            }
        }

        let aux_val = aux.unwrap_or(0);

        let res = match opname {
            "LOADNIL" => format!("{}{} = nil", tab, rn(a)),
            "LOADB" => {
                let mut s = format!("{}{} = {}", tab, rn(a), fmt_bool(b != 0));
                if c != 0 {
                    s.push_str(&format!("\n{}goto [{}]", tab, code_index + 1 + c as usize));
                }
                s
            },
            "LOADN" => format!("{}{} = {}", tab, rn(a), s_bx),
            "LOADK" => if (bx as usize) < proto.k_table.len() {
                format!("{}{} = {}", tab, rn(a), format_constant(&proto.k_table[bx as usize]))
            } else {
                format!("{}{} = <invalid index {}>", tab, rn(a), bx)
            },
            "MOVE" => format!("{}{} = {}", tab, rn(a), rn(b)),
            "GETGLOBAL" => if (aux_val as usize) < proto.k_table.len() {
                format!("{}{} = _G[{}]", tab, rn(a), format_constant(&proto.k_table[aux_val as usize]))
            } else {
                format!("{}{} = _G[Invalid constant index]", tab, rn(a))
            },
            "SETGLOBAL" => if (aux_val as usize) < proto.k_table.len() {
                format!("{}_G[{}] = {}", tab, format_constant(&proto.k_table[aux_val as usize]), rn(a))
            } else {
                format!("{}_G[Invalid string index] = {}", tab, rn(a))
            },
            "GETUPVAL" => format!("{}{} = {}", tab, rn(a), un(b)),
            "SETUPVAL" => format!("{}{} = {}", tab, un(b), rn(a)),
            "CLOSEUPVALS" => format!("{}close upvalues {}+", tab, rn(a)),
            "GETIMPORT" => if (bx as usize) < proto.k_table.len() {
                if let ConstantValue::Import(id) = &proto.k_table[bx as usize].value {
                    format!("{}{} = {}", tab, rn(a), import_id_to_name(proto, *id))
                } else { format!("{}{} = <invalid import type>", tab, rn(a)) }
            } else {
                format!("{}{} = <invalid import index {}>", tab, rn(a), bx)
            },
            "GETTABLE" => format!("{}{} = {}[{}]", tab, rn(a), rn(b), rn(c)),
            "SETTABLE" => format!("{}{}[{}] = {}", tab, rn(b), rn(c), rn(a)),
            "GETTABLEKS" => if (aux_val as usize) < proto.k_table.len() {
                format!("{}{} = {}[{}]", tab, rn(a), rn(b), format_constant(&proto.k_table[aux_val as usize]))
            } else {
                format!("{}{} = {}[Invalid string index]", tab, rn(a), rn(b))
            },
            "SETTABLEKS" => if (aux_val as usize) < proto.k_table.len() {
                format!("{}{}[{}] = {}", tab, rn(b), format_constant(&proto.k_table[aux_val as usize]), rn(a))
            } else {
                format!("{}{}[Invalid string index] = {}", tab, rn(b), rn(a))
            },
            "GETTABLEN" => format!("{}{} = {}[{}]", tab, rn(a), rn(b), c + 1),
            "SETTABLEN" => format!("{}{}[{}] = {}", tab, rn(b), c + 1, rn(a)),
            "NEWCLOSURE" => format!("{}{} = closure(proto[{}])", tab, rn(a), bx),
            "NAMECALL" => if (aux_val as usize) < proto.k_table.len() {
                format!("{}{} = {}[{}]; {} = {}", tab, rn(a), rn(b), format_constant(&proto.k_table[aux_val as usize]), rn(a+1), rn(b))
            } else {
                format!("{}{} = {}[Invalid string index]; {} = {}", tab, rn(a), rn(b), rn(a+1), rn(b))
            },
            "CALL" => {
                let args = if b == 1 { "".to_string() } else if b == 0 { format!("{} ...", rn(a+1)) } else { format!("{}", rn(a+1)) + if b > 2 { format!(" ... {}", rn(a+b-1)).as_str() } else { "" }.to_string().as_str().to_string() };
                let rets = if c == 1 { "".to_string() } else if c == 0 { format!("{} ...", rn(a)) } else { format!("{}", rn(a)) + if c > 2 { format!(" ... {}", rn(a+c-2)).as_str() } else { "" }.to_string().as_str().to_string() };
                let call_str = format!("{}({})", rn(a), args);
                if !rets.is_empty() { format!("{}{} = {}", tab, rets, call_str) } else { format!("{}{}", tab, call_str) }
            },
            "RETURN" => if b == 0 { format!("{}return {} ...", tab, rn(a)) } else if b == 1 { format!("{}return", tab) } else { format!("{}return {} ... {}", tab, rn(a), rn(a+b-2)) },
            "JUMP" | "JUMPBACK" => format!("{}goto [{}]", tab, code_index as i64 + 1 + s_bx as i64),
            "JUMPIF" => format!("{}if {} then goto [{}]", tab, rn(a), code_index as i64 + 1 + s_bx as i64),
            "JUMPIFNOT" => format!("{}if not {} then goto [{}]", tab, rn(a), code_index as i64 + 1 + s_bx as i64),
            "JUMPIFEQ" | "JUMPIFLE" | "JUMPIFLT" | "JUMPIFNOTEQ" | "JUMPIFNOTLE" | "JUMPIFNOTLT" => {
                let op = match opname { "JUMPIFEQ" => "==", "JUMPIFLE" => "<=", "JUMPIFLT" => "<", "JUMPIFNOTEQ" => "~=", "JUMPIFNOTLE" => ">", "JUMPIFNOTLT" => ">=", _ => unreachable!() };
                format!("{}if {} {} {} then goto [{}]", tab, rn(a), op, rn(aux_val as u8), code_index as i64 + 1 + s_bx as i64)
            },
            "ADD" | "SUB" | "MUL" | "DIV" | "MOD" | "POW" | "ADDK" | "SUBK" | "MULK" | "DIVK" | "MODK" | "POWK" | "ADDRK" | "SUBRK" | "DIVRK" => {
                let op = match opname {
                    "ADD" | "ADDK" | "ADDRK" => "+", "SUB" | "SUBK" | "SUBRK" => "-", "MUL" | "MULK" => "*",
                    "DIV" | "DIVK" | "DIVRK" => "/", "MOD" | "MODK" => "%", "POW" | "POWK" => "^", _ => unreachable!()
                };
                if opname.ends_with("RK") {
                    let k = if (b as usize) < proto.k_table.len() { format_constant(&proto.k_table[b as usize]) } else { "nil".to_string() };
                    format!("{}{} = {} {} {}", tab, rn(a), k, op, rn(c))
                } else if opname.ends_with("K") {
                    let k = if (c as usize) < proto.k_table.len() { format_constant(&proto.k_table[c as usize]) } else { "nil".to_string() };
                    format!("{}{} = {} {} {}", tab, rn(a), rn(b), op, k)
                } else {
                    format!("{}{} = {} {} {}", tab, rn(a), rn(b), op, rn(c))
                }
            },
            "AND" | "OR" | "ANDK" | "ORK" => {
                let op = if opname.starts_with("AND") { "and" } else { "or" };
                if opname.ends_with("K") {
                    let k = if (c as usize) < proto.k_table.len() { format_constant(&proto.k_table[c as usize]) } else { "nil".to_string() };
                    format!("{}{} = {} {} {}", tab, rn(a), rn(b), op, k)
                } else {
                    format!("{}{} = {} {} {}", tab, rn(a), rn(b), op, rn(c))
                }
            },
            "NOT" => format!("{}{} = not {}", tab, rn(a), rn(b)),
            "NOP" => format!("{}nop", tab),
            "BREAK" => format!("{}break", tab),
            "FORNPREP" => format!("{}{} = fornprep({}, {})", tab, rn(a), rn(a), s_bx),
            "FORNLOOP" => format!("{}{} = fornloop({}, {})", tab, rn(a), rn(a), s_bx),
            "MINUS" => format!("{}{} = -{}", tab, rn(a), rn(b)),
            "LENGTH" => format!("{}{} = #{}", tab, rn(a), rn(b)),
            "CONCAT" => format!("{}{} = {} .. {}", tab, rn(a), rn(b), rn(c)),
            "JUMPIFEQK" => {
                let k_val = if (bx as usize) < proto.k_table.len() { format_constant(&proto.k_table[bx as usize]) } else { format!("{:?}", bx) };
                format!("{}if {} == {} then goto [{}]", tab, rn(a), k_val, code_index as i64 + 1 + s_bx as i64)
            },
            "FASTCALL" => format!("{}{} = fastcall({}, {})", tab, rn(a), b, c),
            "FASTCALL1" => format!("{}{} = fastcall1({}, {})", tab, rn(a), b, rn(c)),
            "FASTCALL2" => if aux.is_some() { format!("{}{} = fastcall2({}, {}, {})", tab, rn(a), b, rn(c), rn(aux_val as u8)) } else { format!("{}{} = fastcall2({}, {}, <invalid register>)", tab, rn(a), b, rn(c)) },
            "FASTCALL2K" => {
                let k = if (aux_val as usize) < proto.k_table.len() { format_constant(&proto.k_table[aux_val as usize]) } else { "nil".to_string() };
                format!("{}{} = fastcall2k({}, {}, {})", tab, rn(a), b, rn(c), k)
            },
            "FORGLOOP" => format!("{}{} = forgloop({}, {})", tab, rn(a), rn(a), s_bx),
            "FORGLOOP_INEXT" => format!("{}{} = forgloop_inext({}, {})", tab, rn(a), rn(a), s_bx),
            "FORGLOOP_NEXT" => format!("{}{} = forgloop_next({}, {})", tab, rn(a), rn(a), s_bx),
            "FORGPREP" => format!("{}{} = forgprep({}, {})", tab, rn(a), rn(a), s_bx),
            "FORGPREP_INEXT" => format!("{}{} = forgprep_inext({}, {})", tab, rn(a), rn(a), s_bx),
            "FORGPREP_NEXT" => format!("{}{} = forgprep_next({}, {})", tab, rn(a), rn(a), s_bx),
            "GETVARARGS" => format!("{}{}, ... = ..., ({} args)", tab, rn(a), b - 1),
            "DUPCLOSURE" => format!("{}{} = dupclosure(K{})", tab, rn(a), bx),
            "PREPVARARGS" => format!("{}prepare_varargs({})", tab, a),
            "LOADKX" => if (aux_val as usize) < proto.k_table.len() {
                format!("{}{} = {}", tab, rn(a), format_constant(&proto.k_table[aux_val as usize]))
            } else {
                format!("{}{} = <invalid constant>", tab, rn(a))
            },
            "JUMPX" => format!("{}goto [{}]", tab, code_index as i64 + 1 + s_ax as i64),
            "NEWTABLE" => format!("{}{} = {{}}", tab, rn(a)),
            "DUPTABLE" => format!("{}{} = {{}}", tab, rn(a)),
            "SETLIST" => if c == 0 { format!("{}{}[{}..] = {} ... top", tab, rn(a), aux_val, rn(b)) } else { format!("{}{}[{}..{}] = {} ... {}", tab, rn(a), aux_val, aux_val + c as u32 - 1, rn(b), rn(b + c - 1)) },
            "CAPTURE" => if a == 0 { format!("{}capture(upvalue, {})", tab, rn(b)) } else { format!("{}capture({})", tab, rn(b)) },
            "JUMPXEQKNIL" => format!("{}if {} == nil then goto [{}]", tab, rn(a), code_index as i64 + 1 + s_ax as i64),
            "JUMPXEQKB" => format!("{}if {} {} {} then goto [{}]", tab, rn(a), if aux_val & 0x80000000 != 0 { "~=" } else { "==" }, fmt_bool(aux_val & 1 != 0), code_index as i64 + 1 + s_ax as i64),
            "JUMPXEQKN" => {
                let k_idx = (aux_val & 0x7FFFFFFF) as usize;
                if k_idx < proto.k_table.len() {
                    let k_val = format_constant(&proto.k_table[k_idx]);
                    format!("{}if {} {} {} then goto [{}]", tab, rn(a), if aux_val & 0x80000000 != 0 { "~=" } else { "==" }, k_val, code_index as i64 + 1 + s_ax as i64)
                } else {
                    format!("{}if {} == K{} then goto [{}]", tab, rn(a), k_idx, code_index as i64 + 1 + s_ax as i64)
                }
            },
            "JUMPXEQKS" => {
                let k_idx = (aux_val & 0x7FFFFFFF) as usize;
                if k_idx < proto.k_table.len() {
                    let k_val = format_constant(&proto.k_table[k_idx]);
                    format!("{}if {} {} {} then goto [{}]", tab, rn(a), if aux_val & 0x80000000 != 0 { "~=" } else { "==" }, k_val, code_index as i64 + 1 + s_ax as i64)
                } else {
                    format!("{}if {} == <invalid string> then goto [{}]", tab, rn(a), code_index as i64 + 1 + s_ax as i64)
                }
            },
            "IDIV" => format!("{}{} = {} // {}", tab, rn(a), rn(b), rn(c)),
            "IDIVK" => {
                let k = if (c as usize) < proto.k_table.len() { format_constant(&proto.k_table[c as usize]) } else { "nil".to_string() };
                format!("{}{} = {} // {}", tab, rn(a), rn(b), k)
            },
            "BAND" => format!("{}{} = {} & {}", tab, rn(a), rn(b), rn(c)),
            "BOR" => format!("{}{} = {} | {}", tab, rn(a), rn(b), rn(c)),
            "BXOR" => format!("{}{} = {} ~ {}", tab, rn(a), rn(b), rn(c)),
            "BNOT" => format!("{}{} = ~{}", tab, rn(a), rn(b)),
            "SHL" => format!("{}{} = {} << {}", tab, rn(a), rn(b), rn(c)),
            "SHR" => format!("{}{} = {} >> {}", tab, rn(a), rn(b), rn(c)),
            "BANDK" => {
                let k = if (c as usize) < proto.k_table.len() { format_constant(&proto.k_table[c as usize]) } else { "nil".to_string() };
                format!("{}{} = {} & {}", tab, rn(a), rn(b), k)
            },
            "BORK" => {
                let k = if (c as usize) < proto.k_table.len() { format_constant(&proto.k_table[c as usize]) } else { "nil".to_string() };
                format!("{}{} = {} | {}", tab, rn(a), rn(b), k)
            },
            "BXORK" => {
                let k = if (c as usize) < proto.k_table.len() { format_constant(&proto.k_table[c as usize]) } else { "nil".to_string() };
                format!("{}{} = {} ~ {}", tab, rn(a), rn(b), k)
            },
            "SHLI" => format!("{}{} = {} << {}", tab, rn(a), rn(b), c),
            "SHRI" => format!("{}{} = {} >> {}", tab, rn(a), rn(b), c),
            "COVERAGE" => format!("{}coverage({})", tab, aux_val),
            _ => format!("{}UNKNOWN OPCODE: {}", tab, opname),
        };
        output.push(res);
        code_index += 1;
    }

    // Handle child protos recursively
    if !proto.p_table.is_empty() {
        for (i, &child_id) in proto.p_table.iter().enumerate() {
            if child_id < proto_table.len() {
                output.push(format!("\n{}-- child proto {}", tab, i));
                let child_proto = &proto_table[child_id];
                output.push(decompile(child_proto, depth + 1, _string_table, luau_version, proto_table));
            }
        }
    }

    output.push("end".to_string());
    output.join("\n")
}

// MAIN ENTRY POINT

fn disassemble(bytecode: &[u8]) -> (Vec<String>, Vec<String>, usize, i32, i32) {
    if bytecode.is_empty() { return (vec![], vec![], 0, -1, -1); }
    if bytecode[0] == 0 {
        return (vec![String::from_utf8_lossy(&bytecode[1..]).to_string()], vec![], 0, -1, -1);
    }

    let (_main_proto, proto_table, string_table, luau_version, types_version) = deserialize(bytecode);

    let mut child_proto_indices = HashSet::new();
    for proto in &proto_table {
        for &child_idx in &proto.p_table {
            child_proto_indices.insert(child_idx);
        }
    }

    let mut output = Vec::new();
    let mut decompiled_output = Vec::new();
    let mut protos = 0;

    for (i, proto) in proto_table.iter().enumerate() {
        if child_proto_indices.contains(&i) { continue; }

        output.push(format!("--< Proto->{:03} | Line {} >--", i, proto.line_defined));
        output.push(read_proto(proto, 1, &proto_table, &string_table, luau_version));

        decompiled_output.push(format!("-- Decompiled Proto->{:03} --", i));
        decompiled_output.push(decompile(proto, 1, &string_table, luau_version, &proto_table));

        protos += 1;
    }

    (output, decompiled_output, protos, luau_version as i32, types_version as i32)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <bytecode_file>", args[0]);
        std::process::exit(1);
    }

    let mut file = File::open(&args[1]).expect("Failed to open file");
    let mut bytecode = Vec::new();
    file.read_to_end(&mut bytecode).expect("Failed to read file");

    let start = Instant::now();
    let (disassembled, decompiled, protos, luau_version, types_version) = disassemble(&bytecode);
    let duration = start.elapsed();

    let disassembled_extra = "--<@ Disassembled with Koralys' BETA disassembler @>--\n".to_string();
    let versions = if luau_version != -1 {
        format!("Luau version {}, types version {}", luau_version, types_version)
    } else if types_version != -1 {
        format!("Luau version unknown, types version {}", types_version)
    } else {
        "Types version unknown, luau version unknown".to_string()
    };

    let mut full_output = disassembled_extra;
    full_output.push_str(&format!("--<@ Protos: {} | {} @>--\n", protos, versions));
    full_output.push_str(&format!("--<@ Time taken: {:.6}s @>--\n", duration.as_secs_f64()));
    full_output.push_str(&disassembled.join("\n"));

    let mut out_file = File::create("output.txt").expect("Failed to create output.txt");
    out_file.write_all(full_output.as_bytes()).expect("Failed to write output.txt");

    println!("Disassembled bytecode in {:.6}s", duration.as_secs_f64());

    let decompiled_str = decompiled.join("\n");
    let mut decomp_file = File::create("decompiled.luau").expect("Failed to create decompiled.luau");
    decomp_file.write_all(decompiled_str.as_bytes()).expect("Failed to write decompiled.luau");

    println!("Decompiled disassembly");
}
