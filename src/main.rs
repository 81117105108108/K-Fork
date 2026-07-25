use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::time::Instant;
use koralys_rust::disassemble;

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
