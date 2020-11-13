#[macro_use]
extern crate pest_derive;

use env_logger::Env;

mod memory;
mod compiler;

mod vm;

fn compile() {
    let mut compiler = compiler::MMCompiler::new(); // pass from file data structure

    compiler.process_file("./memyself.txt");   
    compiler.write_obj_file("");
}

fn run() {
    let mut machine = vm::VM::new();
    machine.load_file().unwrap();
    machine.run();
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).format_module_path(false).format_timestamp(None).init();

    compile();

    run();
}