#[macro_use]
extern crate pest_derive;

use env_logger::Env;
use std::env;
use turtle;

mod memory;
mod compiler;

mod vm;

// Compile a program and set an output file
fn compile(in_file: &str, out_file: &str) {
    let mut compiler = compiler::MMCompiler::new(); // pass from file data structure

    compiler.process_file(in_file);   
    compiler.write_obj_file(out_file).unwrap();
}

// Run a file
fn run(file_name: &str) {
    let mut machine = vm::VM::new();
    machine.load_file(file_name).unwrap();
    machine.run();
}

fn main() {
    turtle::start();

    // Start environmental logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).format_module_path(false).format_timestamp(None).init();

    let help = r#"
___  ___      ___  ___               _  __ 
|  \/  |      |  \/  |              | |/ _|
| .  . | ___  | .  . |_   _ ___  ___| | |_ 
| |\/| |/ _ \ | |\/| | | | / __|/ _ \ |  _|
| |  | |  __/ | |  | | |_| \__ \  __/ | |  
\_|  |_/\___| \_|  |_/\__, |___/\___|_|_|  
                    __/ |               
                    |___/                

A simple graphical language.

Built by Ricardo Garza - A00816705

USAGE:
    me_myself [COMMAND] <OPTIONS>

COMMAND:
    compile <in_file> <out_file>    Compile a me_myself program. If not given, <out_file> is "file.obj".
    run <in_file>                   Run a .obj me_myself program. If not given, <in_file> is "file.obj".
    help                            Show this message

"#;

    let args: Vec<String> = env::args().collect();

    // Process args to compile/run
    if args.len() == 1 {
        println!("{}", help);
        return;
    }
    let first_arg = &args[1];
    match first_arg.as_str() {
        "compile" => {
            if args.len() == 3 {
                compile(&args[2], "file.obj");
            } else if args.len() == 4 {
                compile(&args[2], &args[3]);
            } else {
                println!("{}", help);
            }
        }
        "run" => {
            if args.len() == 2 {
                run("file.obj");
            } else if args.len() == 3 {
                run(&args[2]);
            }
        }
        "help" | &_ => {
            println!("{}", help);
        }
    }
}