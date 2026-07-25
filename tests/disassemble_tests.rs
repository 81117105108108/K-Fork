use std::collections::HashSet;
use koralys_rust::{
    decompose_import_id, disassemble, get_op_table, import_id_to_name, Constant, ConstantValue,
    Proto, LBC_CONSTANT_BOOLEAN, LBC_CONSTANT_CLOSURE, LBC_CONSTANT_IMPORT, LBC_CONSTANT_NIL,
    LBC_CONSTANT_NUMBER, LBC_CONSTANT_STRING, LBC_CONSTANT_TABLE, LBC_CONSTANT_VECTOR,
    OP_TABLE_V5, OP_TABLE_V6,
};

#[test]
fn test_constants_values() {
    assert_eq!(LBC_CONSTANT_NIL, 0);
    assert_eq!(LBC_CONSTANT_BOOLEAN, 1);
    assert_eq!(LBC_CONSTANT_NUMBER, 2);
    assert_eq!(LBC_CONSTANT_STRING, 3);
    assert_eq!(LBC_CONSTANT_IMPORT, 4);
    assert_eq!(LBC_CONSTANT_TABLE, 5);
    assert_eq!(LBC_CONSTANT_CLOSURE, 6);
    assert_eq!(LBC_CONSTANT_VECTOR, 8);
}

#[test]
fn test_op_table_uniqueness_and_content() {
    for version in [5, 6] {
        let table = get_op_table(version);
        let mut numbers = HashSet::new();
        let mut names = HashSet::new();

        for op in table {
            assert!(!op.name.is_empty());
            assert!(!op.op_type.is_empty());
            assert!(numbers.insert(op.number), "Duplicate opcode number 0x{:02X} in V{}", op.number, version);
            names.insert(op.name);
        }

        for required in ["LOADN", "LOADK", "GETIMPORT", "JUMP", "ADD", "SUB", "MUL", "DIV", "SUBRK", "DIVRK"] {
            assert!(names.contains(required), "Missing opcode {} in V{}", required, version);
        }
    }
}

#[test]
fn test_v5_v6_difference() {
    let names_v5: HashSet<&str> = OP_TABLE_V5.iter().map(|op| op.name).collect();
    let names_v6: HashSet<&str> = OP_TABLE_V6.iter().map(|op| op.name).collect();

    assert!(names_v5.contains("DEP_FORGLOOP_INEXT"));
    assert!(!names_v6.contains("DEP_FORGLOOP_INEXT"));

    assert!(!names_v5.contains("FASTCALL3"));
    assert!(names_v6.contains("FASTCALL3"));
}

#[test]
fn test_import_id_decomposition() {
    // count=1, id1=42
    let import_id1 = (1 << 30) | (42 << 20);
    let ids1 = decompose_import_id(import_id1);
    assert_eq!(ids1, vec![42]);

    // count=2, id1=100, id2=200
    let import_id2 = (2 << 30) | (100 << 20) | (200 << 10);
    let ids2 = decompose_import_id(import_id2);
    assert_eq!(ids2, vec![100, 200]);

    // count=3, id1=5, id2=10, id3=15
    let import_id3 = (3 << 30) | (5 << 20) | (10 << 10) | 15;
    let ids3 = decompose_import_id(import_id3);
    assert_eq!(ids3, vec![5, 10, 15]);
}

#[test]
fn test_import_id_to_name() {
    let proto = Proto {
        k_table: vec![
            Constant { value: ConstantValue::String("game".to_string()) },
            Constant { value: ConstantValue::String("GetService".to_string()) },
            Constant { value: ConstantValue::String("Workspace".to_string()) },
        ],
        ..Default::default()
    };

    let import_id = (2 << 30) | (0 << 20) | (1 << 10);
    assert_eq!(import_id_to_name(&proto, import_id), "game.GetService");

    let single_id = (1 << 30) | (2 << 20);
    assert_eq!(import_id_to_name(&proto, single_id), "Workspace");
}

#[test]
fn test_disassemble_empty_and_error() {
    let (dis, decomp, protos, luau_ver, types_ver) = disassemble(&[]);
    assert!(dis.is_empty());
    assert!(decomp.is_empty());
    assert_eq!(protos, 0);
    assert_eq!(luau_ver, -1);
    assert_eq!(types_ver, -1);

    let (dis, decomp, protos, luau_ver, types_ver) = disassemble(&[0, b'E', b'r', b'r', b'o', b'r']);
    assert_eq!(dis, vec!["Error"]);
    assert!(decomp.is_empty());
    assert_eq!(protos, 0);
    assert_eq!(luau_ver, -1);
    assert_eq!(types_ver, -1);
}
