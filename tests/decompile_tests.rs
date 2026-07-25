use koralys_rust::{
    decompile, Constant, ConstantValue, DebugInfo, Proto, VarInfo,
};

fn encode_inst(op: u8, a: u8, b: u8, c: u8) -> u32 {
    // Opcode is encoded via multiplication table: (i * 227) & 0xFF == op
    // To encode opcode number N: we find i in 0..256 such that (i * 227) & 0xFF == N
    let mut inst_op = 0u32;
    for i in 0..256u32 {
        if ((i.wrapping_mul(227)) & 0xFF) as u8 == op {
            inst_op = i;
            break;
        }
    }
    inst_op | ((a as u32) << 8) | ((b as u32) << 16) | ((c as u32) << 24)
}

fn encode_inst_bx(op: u8, a: u8, bx: u32) -> u32 {
    let mut inst_op = 0u32;
    for i in 0..256u32 {
        if ((i.wrapping_mul(227)) & 0xFF) as u8 == op {
            inst_op = i;
            break;
        }
    }
    inst_op | ((a as u32) << 8) | (bx << 16)
}

fn encode_inst_sbx(op: u8, a: u8, sbx: i32) -> u32 {
    let bx = if sbx < 0 {
        (sbx + 0x10000) as u32
    } else {
        sbx as u32
    };
    encode_inst_bx(op, a, bx)
}

#[test]
fn test_decompile_empty_proto() {
    let proto = Proto::default();
    let result = decompile(&proto, 0, &[], 5, &[]);
    assert!(result.contains("local function func0()"));
    assert!(result.contains("end"));
}

#[test]
fn test_decompile_parameters_and_varargs() {
    let proto = Proto {
        num_params: 3,
        is_var_arg: true,
        ..Default::default()
    };
    let result = decompile(&proto, 0, &[], 5, &[]);
    assert!(result.contains("local function func0(R0, R1, R2, ...)"));
}

#[test]
fn test_decompile_debug_info() {
    let proto = Proto {
        num_params: 2,
        num_upvalues: 1,
        debug_info: Some(DebugInfo {
            var_info: vec![
                VarInfo { name: "player".to_string(), start_pc: 0, end_pc: 10, reg: 0 },
                VarInfo { name: "health".to_string(), start_pc: 0, end_pc: 10, reg: 1 },
                VarInfo { name: "temp".to_string(), start_pc: 1, end_pc: 5, reg: 2 },
            ],
            upvalue_info: vec!["Workspace".to_string()],
        }),
        ..Default::default()
    };
    let result = decompile(&proto, 0, &[], 5, &[]);
    assert!(result.contains("local function func0(player, health)"));
}

#[test]
fn test_decompile_constant_loads() {
    // Opcode numbers from OP_TABLE_V5
    // LOADNIL: 0xC6, LOADB: 0xA9, LOADN: 0x8C, LOADK: 0x6F
    let code = vec![
        encode_inst(0xC6, 0, 0, 0),        // LOADNIL R0
        encode_inst(0xA9, 1, 1, 0),        // LOADB R1 true
        encode_inst_sbx(0x8C, 2, 42),      // LOADN R2 42
        encode_inst_bx(0x6F, 3, 0),        // LOADK R3 K0
    ];

    let proto = Proto {
        code_table: code,
        k_table: vec![Constant { value: ConstantValue::String("hello world".to_string()) }],
        ..Default::default()
    };

    let result = decompile(&proto, 0, &[], 5, &[]);
    assert!(result.contains("R0 = nil"));
    assert!(result.contains("R1 = true"));
    assert!(result.contains("R2 = 42"));
    assert!(result.contains("R3 = \"hello world\""));
}

#[test]
fn test_decompile_arithmetic_ops() {
    // ADD: 0x43, SUB: 0x26, MUL: 0x09, DIV: 0xEC
    let code = vec![
        encode_inst(0x43, 0, 1, 2), // ADD R0 = R1 + R2
        encode_inst(0x26, 3, 0, 1), // SUB R3 = R0 - R1
        encode_inst(0x09, 4, 3, 2), // MUL R4 = R3 * R2
        encode_inst(0xEC, 5, 4, 1), // DIV R5 = R4 / R1
    ];

    let proto = Proto {
        code_table: code,
        ..Default::default()
    };

    let result = decompile(&proto, 0, &[], 5, &[]);
    assert!(result.contains("R0 = R1 + R2"));
    assert!(result.contains("R3 = R0 - R1"));
    assert!(result.contains("R4 = R3 * R2"));
    assert!(result.contains("R5 = R4 / R1"));
}

#[test]
fn test_decompile_globals_and_upvalues() {
    // GETUPVAL: 0xFB, SETUPVAL: 0xDE, GETGLOBAL: 0x35 (aux), SETGLOBAL: 0x18 (aux)
    let code = vec![
        encode_inst(0xFB, 0, 1, 0),        // GETUPVAL R0 = U1
        encode_inst(0xDE, 0, 2, 0),        // SETUPVAL U2 = R0
        encode_inst(0x35, 1, 0, 0), 0,     // GETGLOBAL R1 = _G[K0]
        encode_inst(0x18, 1, 0, 0), 0,     // SETGLOBAL _G[K0] = R1
    ];

    let proto = Proto {
        code_table: code,
        k_table: vec![Constant { value: ConstantValue::String("print".to_string()) }],
        ..Default::default()
    };

    let result = decompile(&proto, 0, &[], 5, &[]);
    assert!(result.contains("R0 = U1"));
    assert!(result.contains("U2 = R0"));
    assert!(result.contains("R1 = _G[\"print\"]"));
    assert!(result.contains("_G[\"print\"] = R1"));
}

#[test]
fn test_decompile_table_ops() {
    // NEWTABLE: 0xFF (aux=0), GETTABLE: 0x87, SETTABLE: 0x6A
    let code = vec![
        encode_inst(0xFF, 0, 0, 0), 0,     // NEWTABLE R0 = {}
        encode_inst(0x87, 1, 0, 2),        // GETTABLE R1 = R0[R2]
        encode_inst(0x6A, 3, 0, 1),        // SETTABLE R0[R1] = R3
    ];

    let proto = Proto {
        code_table: code,
        ..Default::default()
    };

    let result = decompile(&proto, 0, &[], 5, &[]);
    assert!(result.contains("R0 = {}"));
    assert!(result.contains("R1 = R0[R2]"));
    assert!(result.contains("R0[R1] = R3"));
}

#[test]
fn test_decompile_control_flow() {
    // JUMP: 0x65, JUMPIF: 0x2B, JUMPIFNOT: 0x0E
    let code = vec![
        encode_inst_sbx(0x2B, 0, 2),       // JUMPIF R0 goto [pc+1+2] -> [3]
        encode_inst_sbx(0x0E, 1, 1),       // JUMPIFNOT R1 goto [pc+1+1] -> [3]
        encode_inst_sbx(0x65, 0, 0),       // JUMP goto [pc+1+0] -> [3]
    ];

    let proto = Proto {
        code_table: code,
        ..Default::default()
    };

    let result = decompile(&proto, 0, &[], 5, &[]);
    assert!(result.contains("if R0 then goto [3]"));
    assert!(result.contains("if not R1 then goto [3]"));
    assert!(result.contains("goto [3]"));
}

#[test]
fn test_decompile_calls_and_returns() {
    // CALL: 0x9F, RETURN: 0x82
    let code = vec![
        encode_inst(0x9F, 0, 3, 2),        // CALL R0 = R0(R1, R2)
        encode_inst(0x82, 0, 2, 0),        // RETURN return R0
    ];

    let proto = Proto {
        code_table: code,
        ..Default::default()
    };

    let result = decompile(&proto, 0, &[], 5, &[]);
    assert!(result.contains("R0 = R0(R1 ... R2)"));
    assert!(result.contains("return R0"));
}

#[test]
fn test_decompile_child_proto() {
    let child = Proto {
        num_params: 1,
        code_table: vec![encode_inst(0x82, 0, 1, 0)], // return
        ..Default::default()
    };

    let parent = Proto {
        code_table: vec![
            encode_inst_bx(0xD9, 0, 0), // NEWCLOSURE R0 = closure(proto[0])
        ],
        p_table: vec![1],
        ..Default::default()
    };

    let proto_table = vec![parent, child];
    let result = decompile(&proto_table[0], 0, &[], 5, &proto_table);
    assert!(result.contains("R0 = closure(proto[0])"));
    assert!(result.contains("-- child proto 0"));
}
