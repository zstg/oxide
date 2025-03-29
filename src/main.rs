extern crate oxide;

use oxide::gen_ir::gen_ir;
use oxide::gen_x86::gen_x86;
use oxide::irdump::dump_ir;
use oxide::parse::parse;
use oxide::preprocess::Preprocessor;
use oxide::regalloc::alloc_regs;
use oxide::sema::sema;
use oxide::token::tokenize;
use oxide::vectorize::vectorize;

use std::env;
use std::process;

fn usage() -> ! {
    eprintln!("Usage: oxide [-dump-ir1] [-dump-ir2] [-dump-ir3] [-no-vec] <file>");
    process::exit(1)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        usage();
    }

    let mut dump_ir1 = false;
    let mut dump_ir2 = false;
    let mut dump_ir3 = false;
    let mut enable_vectorization = true;
    let mut path = String::new();
    
    // Parse command line arguments
    let mut i = 1;
    while i < args.len() {
        if args[i] == "-dump-ir1" {
            dump_ir1 = true;
        } else if args[i] == "-dump-ir2" {
            dump_ir2 = true;
        } else if args[i] == "-dump-ir3" {
            dump_ir3 = true;
        } else if args[i] == "-no-vec" {
            enable_vectorization = false;
        } else if path.is_empty() {
            path = args[i].clone();
        } else {
            usage();
        }
        i += 1;
    }
    
    if path.is_empty() {
        usage();
    }

    // Tokenize and parse
    let tokens = tokenize(path, &mut Preprocessor::new());
    let nodes = parse(&tokens);
    let (nodes, globals) = sema(nodes);
    let mut fns = gen_ir(nodes);

    if dump_ir1 {
        dump_ir(&fns);
    }

    alloc_regs(&mut fns);

    if dump_ir2 {
        dump_ir(&fns);
    }
    
    // Apply vectorization if enabled
    if enable_vectorization {
        vectorize(&mut fns);
        
        if dump_ir3 {
            dump_ir(&fns);
        }
    }

    gen_x86(globals, fns);
}
