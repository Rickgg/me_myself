use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::str::FromStr;
use std::collections::HashMap;
use std::error::Error;

use log::{debug, error, log_enabled, info, Level};

use crate::memory::{Memory, VarValue, BaseDirs};

#[derive(Debug)]
struct Func {
    locals: (i32, i32, i32),
    temps: (i32, i32, i32, i32),
    start_loc: usize
}

#[derive(Debug)]
enum OutOp {
    Str(String),
    Pos(usize),
    Mem(i32),
    None
}

#[derive(Debug)]
struct Quadruple {
    op: String,
    lh_op: Option<i32>,
    rh_op: Option<i32>,
    out_op: OutOp
}

#[derive(Default, Debug)]
pub struct VM {
    func_list: HashMap<String, Func>,
    curr_memory: Memory,
    global_memory: Memory,
    quad_list: Vec<Quadruple>,
    ip: usize,
    ip_stack: Vec<usize>,
    memory_stack: Vec<Memory>,
    constants: HashMap<i32, VarValue>,
    param_pos: (i32, i32, i32)
}

impl VM {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn load_file(&mut self) -> io::Result<()> {
        let file = File::open("file.obj")?;
        let lines = BufReader::new(file).lines();

        // Constants
        for line in lines {
            if let Ok(line) = line {
                let mut info = line.split_whitespace();
                match info.next().unwrap() {
                    "C" => {
                        let value = info.next().unwrap();
                        let location: i32 = info.next().unwrap().parse().unwrap();
                        let r#type = info.next().unwrap();
                        let final_val = match r#type {
                            "Int" => VarValue::Int(value.parse().unwrap()),
                            "Float" => VarValue::Float(value.parse().unwrap()),
                            "Char" => VarValue::Char(value.to_string()),
                            &_ => { unreachable!()}
                        };
                        self.constants.insert(location, final_val);
                    }
                    "G" => {
                        let globals: Vec<i32> = info.map(|s| i32::from_str(s).expect("Can't parse number")).collect();
                        self.global_memory.set_globals(globals[0], globals[1], globals[2]);
                    }
                    "F" => {
                        let func_name = info.next().unwrap();
                        let func_start: usize = info.next().unwrap().parse().expect("Can't parse number");
                        let func_params: Vec<i32> = info.map(|s| i32::from_str(s).expect("Can't parse number")).collect(); 
                        
                        let new_func = Func {
                            locals: (func_params[0], func_params[1], func_params[2]),
                            temps: (func_params[3], func_params[4], func_params[5], func_params[6]),
                            start_loc: func_start
                        };

                        self.func_list.insert(func_name.to_string(), new_func);
                    }
                    "A" => {
                        let op = info.next().unwrap();
                        match op {
                            // Actions that use number as out_op
                            "Sum" | 
                            "Sub" | 
                            "Mult" | 
                            "Div" | 
                            "MoreThan" | 
                            "LessThan" |
                            "LessOrEqualThan" |
                            "MoreOrEqualThan" |
                            "Equal" |
                            "NotEqual" |
                            "And" |
                            "Or" |
                            "Assign" | "Return" => {
                                let params: Vec<i32> = info.map(|s| i32::from_str(s).unwrap()).collect();
                                let lh_op = Some(params[0]);
                                let rh_op = Some(params[1]);
                                let out_op = OutOp::Mem(params[2] as i32);
                                let new_quad = Quadruple {
                                    op: op.to_string(),
                                    lh_op: lh_op,
                                    rh_op: rh_op,
                                    out_op: out_op
                                };
                                self.quad_list.push(new_quad);
                            }
                            "GotoF" => {
                                let params: Vec<i32> = info.map(|s| i32::from_str(s).unwrap()).collect();
                                let lh_op = Some(params[0]);
                                let out_op = OutOp::Pos(params[2] as usize);
                                let new_quad = Quadruple {
                                    op: op.to_string(),
                                    lh_op: lh_op,
                                    rh_op: None,
                                    out_op: out_op
                                };
                                self.quad_list.push(new_quad);
                            }
                            "Goto" => {
                                let params: Vec<i32> = info.map(|s| i32::from_str(s).unwrap()).collect();
                                let out_op = OutOp::Pos(params[2] as usize);
                                let new_quad = Quadruple {
                                    op: op.to_string(),
                                    lh_op: None,
                                    rh_op: None,
                                    out_op: out_op
                                };
                                self.quad_list.push(new_quad);
                            }
                            "Era" | "Gosub" => {
                                let out_op = OutOp::Str(info.nth(2).unwrap().to_string());
                                let new_quad = Quadruple {
                                    op: op.to_string(),
                                    lh_op: None,
                                    rh_op: None,
                                    out_op: out_op
                                };
                                self.quad_list.push(new_quad);
                            }
                            "Param" => {
                                let out_op = OutOp::Mem(info.nth(2).unwrap().parse::<i32>().unwrap());
                                let new_quad = Quadruple {
                                    op: op.to_string(),
                                    lh_op: None,
                                    rh_op: None,
                                    out_op: out_op
                                };
                                self.quad_list.push(new_quad);
                            }
                            "Print" => {
                                let info = info.skip(2);
                                let print_vec: Vec<String> = info.map(|s| s.to_string()).collect();
                                let print_out = print_vec.join(" ");
                                if &print_out[0..1] == "\"" {
                                    let new_quad = Quadruple {
                                        op: op.to_string(),
                                        lh_op: None,
                                        rh_op: None,
                                        out_op: OutOp::Str(print_out)
                                    };
                                    self.quad_list.push(new_quad);
                                } else {
                                    let new_quad = Quadruple {
                                        op: op.to_string(),
                                        lh_op: None,
                                        rh_op: None,
                                        out_op: OutOp::Mem(print_out.parse().unwrap())
                                    };
                                    self.quad_list.push(new_quad);
                                }
                            }
                            "EndFunc" => { // Special function because out is empty
                                let new_quad = Quadruple {
                                    op: op.to_string(),
                                    lh_op: None,
                                    rh_op: None,
                                    out_op: OutOp::None
                                };
                                self.quad_list.push(new_quad);
                            }
                            &_ => {error!("Missing action {}", op); unimplemented!()}
                        }
                    }
                    &_ => { error!("Missing {}", info.nth(0).unwrap()); }
                }
            }
        }

        
        debug!("{:?}", self);

        Ok(())
    }

    fn get_val(&self, location: i32) -> Result<VarValue, String> {
        if location >= BaseDirs::GlobalInt as i32 && location < BaseDirs::GlobalUpperLim as i32 {
            self.global_memory.get_val(location)
        } else if location >= BaseDirs::LocalInt as i32 && location < BaseDirs::TempUpperLim as i32 {
            self.curr_memory.get_val(location)
        } else if location >= BaseDirs::CteInt as i32 && location < BaseDirs::CteUpperLim as i32 {
            match self.constants.get(&location) {
                Some(val) => Ok(val.clone()),
                None => Err(format!("Memory location {} not initialized", location))
            }
        } else {
            Err(format!("Memory location {} not initialized", location))
        }
    }

    fn set_val(&mut self, location: i32, new_val: VarValue) -> Result<(), String> {
        if location >= BaseDirs::GlobalInt as i32 && location < BaseDirs::GlobalUpperLim as i32 {
            self.global_memory.set_val(location, new_val).unwrap()
        } else if location >= BaseDirs::LocalInt as i32 && location < BaseDirs::TempUpperLim as i32 {
            self.curr_memory.set_val(location, new_val).unwrap()
        } else {
            return Err(format!("Memory location {} not initialized", location))
        };
        Ok(())
    }

    pub fn run(&mut self) {
        let mut new_mem: Memory = Default::default();

        // initialize main memory

        let func_data = self.func_list.get("main").unwrap();
        self.curr_memory.set_new_func(func_data.locals, func_data.temps);

        loop {
            let curr_quad: &Quadruple = self.quad_list.get(self.ip).unwrap();
            debug!("Current quad {}: {:?}", self.ip, curr_quad);
            // println!("Current quad {}: {:?}", self.ip, curr_quad);
            match curr_quad.op.as_str() {
                "Goto" => {
                    if let OutOp::Pos(next_pos) = curr_quad.out_op {
                        self.ip = next_pos;
                    } else {
                        unreachable!();
                    }
                }
                "GotoF" => {
                    let lh = curr_quad.lh_op.unwrap();
                    let lh_mem: VarValue = self.curr_memory.get_val(lh).unwrap();
                    if let VarValue::Bool(comp_val) = lh_mem {
                        if !comp_val {
                            // Jump on false
                            if let OutOp::Pos(next_pos) = curr_quad.out_op {
                                self.ip = next_pos;
                            } else {
                                unreachable!();
                            }
                        } else {
                            self.ip += 1;
                        }
                    } else {
                        unreachable!();
                    }
                }
                "Param" => {
                    if let OutOp::Mem(param) = &curr_quad.out_op {
                        let param_val: VarValue = self.get_val(*param).unwrap();
                        debug!("Param init, mem: {:?}", new_mem);
                        match param_val {
                            VarValue::Int(_) => {
                                new_mem.set_val(BaseDirs::LocalInt as i32 + self.param_pos.0, param_val).unwrap();
                                self.param_pos.0 += 1;
                            }
                            VarValue::Float(_) => {
                                new_mem.set_val(BaseDirs::LocalFloat as i32 + self.param_pos.1, param_val).unwrap();
                                self.param_pos.1 += 1;
                            }
                            VarValue::Char(_) => {
                                new_mem.set_val(BaseDirs::LocalChar as i32 + self.param_pos.2, param_val).unwrap();
                                self.param_pos.2 += 1;
                            }
                            _ => unreachable!()
                        }
                        self.ip += 1;
                    } else {
                        unreachable!()
                    }
                }
                "Era" => {
                    if let OutOp::Str(func_name) = &curr_quad.out_op {
                        new_mem.clear();
                        let func_data = self.func_list.get(func_name).unwrap();
                        new_mem.set_new_func(func_data.locals, func_data.temps);
                        self.ip += 1;
                    } else {
                        unreachable!()
                    }
                }
                "Gosub" => {
                    self.memory_stack.push(self.curr_memory.clone());

                    self.curr_memory = new_mem.clone();
                    if let OutOp::Str(func_name) = &curr_quad.out_op {
                        let func: &Func = self.func_list.get(func_name).unwrap();
                        self.ip_stack.push(self.ip.clone());
                        self.ip = func.start_loc;
                        self.param_pos = (0,0,0);

                    } else {
                        unreachable!()
                    }
                }
                "Return" => {
                    let lh = curr_quad.lh_op.unwrap();
                    let lh_mem: VarValue = self.get_val(lh).unwrap();
                    if let OutOp::Mem(out_mem) = curr_quad.out_op {
                        self.set_val(out_mem, lh_mem).unwrap();
                        self.ip += 1;
                    } else {
                        unreachable!()
                    }
                }
                "Sum" => {
                    let lh = curr_quad.lh_op.unwrap();
                    let lh_mem: VarValue = self.get_val(lh).unwrap();
                    let rh = curr_quad.rh_op.unwrap();
                    let rh_mem = self.get_val(rh).unwrap();
                    if let OutOp::Mem(out_mem) = curr_quad.out_op {
                        let out_val = match (lh_mem, rh_mem) {
                            (VarValue::Int(val_l), VarValue::Int(val_r)) => VarValue::Int(val_l + val_r),
                            (VarValue::Float(val_l), VarValue::Int(val_r)) => VarValue::Float(val_l + val_r as f64),
                            (VarValue::Int(val_l), VarValue::Float(val_r)) => VarValue::Float(val_l as f64 + val_r),
                            (VarValue::Float(val_l), VarValue::Float(val_r)) => VarValue::Float(val_l + val_r),
                            _ => unreachable!()
                        };
                        self.set_val(out_mem, out_val).unwrap();
                        self.ip += 1;
                    } else {
                        unreachable!()
                    }
                }
                "Sub" => {
                    let lh = curr_quad.lh_op.unwrap();
                    let lh_mem: VarValue = self.get_val(lh).unwrap();
                    let rh = curr_quad.rh_op.unwrap();
                    let rh_mem = self.get_val(rh).unwrap();
                    if let OutOp::Mem(out_mem) = curr_quad.out_op {
                        let out_val = match (lh_mem, rh_mem) {
                            (VarValue::Int(val_l), VarValue::Int(val_r)) => VarValue::Int(val_l - val_r),
                            (VarValue::Float(val_l), VarValue::Int(val_r)) => VarValue::Float(val_l - val_r as f64),
                            (VarValue::Int(val_l), VarValue::Float(val_r)) => VarValue::Float(val_l as f64 - val_r),
                            (VarValue::Float(val_l), VarValue::Float(val_r)) => VarValue::Float(val_l - val_r),
                            _ => unreachable!()
                        };
                        self.set_val(out_mem, out_val).unwrap();
                        self.ip += 1;
                    } else {
                        unreachable!()
                    }
                }
                "Mult" => {
                    let lh = curr_quad.lh_op.unwrap();
                    let lh_mem: VarValue = self.get_val(lh).unwrap();
                    let rh = curr_quad.rh_op.unwrap();
                    let rh_mem = self.get_val(rh).unwrap();
                    if let OutOp::Mem(out_mem) = curr_quad.out_op {
                        let out_val = match (lh_mem, rh_mem) {
                            (VarValue::Int(val_l), VarValue::Int(val_r)) => VarValue::Int(val_l * val_r),
                            (VarValue::Float(val_l), VarValue::Int(val_r)) => VarValue::Float(val_l * val_r as f64),
                            (VarValue::Int(val_l), VarValue::Float(val_r)) => VarValue::Float(val_l as f64 * val_r),
                            (VarValue::Float(val_l), VarValue::Float(val_r)) => VarValue::Float(val_l * val_r),
                            _ => unreachable!()
                        };
                        self.set_val(out_mem, out_val).unwrap();
                        self.ip += 1;
                    } else {
                        unreachable!()
                    }
                }
                "Div" => {
                    let lh = curr_quad.lh_op.unwrap();
                    let lh_mem: VarValue = self.get_val(lh).unwrap();
                    let rh = curr_quad.rh_op.unwrap();
                    let rh_mem = self.get_val(rh).unwrap();
                    if let OutOp::Mem(out_mem) = curr_quad.out_op {
                        let out_val = match (lh_mem, rh_mem) {
                            (VarValue::Int(val_l), VarValue::Int(val_r)) => VarValue::Int(val_l / val_r),
                            (VarValue::Float(val_l), VarValue::Int(val_r)) => VarValue::Float(val_l / val_r as f64),
                            (VarValue::Int(val_l), VarValue::Float(val_r)) => VarValue::Float(val_l as f64 / val_r),
                            (VarValue::Float(val_l), VarValue::Float(val_r)) => VarValue::Float(val_l / val_r),
                            _ => unreachable!()
                        };
                        self.set_val(out_mem, out_val).unwrap();
                        self.ip += 1;
                    } else {
                        unreachable!()
                    }
                }
                "MoreThan" => {
                    let lh = curr_quad.lh_op.unwrap();
                    let lh_mem: VarValue = self.get_val(lh).unwrap();
                    let rh = curr_quad.rh_op.unwrap();
                    let rh_mem = self.get_val(rh).unwrap();
                    if let OutOp::Mem(out_mem) = curr_quad.out_op {
                        let out_val = match (lh_mem, rh_mem) {
                            (VarValue::Int(val_l), VarValue::Int(val_r)) => VarValue::Bool(val_l > val_r),
                            (VarValue::Float(val_l), VarValue::Int(val_r)) => VarValue::Bool(val_l > val_r as f64),
                            (VarValue::Int(val_l), VarValue::Float(val_r)) => VarValue::Bool(val_l as f64 > val_r),
                            (VarValue::Float(val_l), VarValue::Float(val_r)) => VarValue::Bool(val_l > val_r),
                            _ => unreachable!()
                        };
                        self.set_val(out_mem, out_val).unwrap();
                        self.ip += 1;
                    } else {
                        unreachable!()
                    }
                }
                "LessThan" => {
                    let lh = curr_quad.lh_op.unwrap();
                    let lh_mem: VarValue = self.get_val(lh).unwrap();
                    let rh = curr_quad.rh_op.unwrap();
                    let rh_mem = self.get_val(rh).unwrap();
                    if let OutOp::Mem(out_mem) = curr_quad.out_op {
                        let out_val = match (lh_mem, rh_mem) {
                            (VarValue::Int(val_l), VarValue::Int(val_r)) => VarValue::Bool(val_l < val_r),
                            (VarValue::Float(val_l), VarValue::Int(val_r)) => VarValue::Bool(val_l < val_r as f64),
                            (VarValue::Int(val_l), VarValue::Float(val_r)) => VarValue::Bool((val_l as f64) < val_r),
                            (VarValue::Float(val_l), VarValue::Float(val_r)) => VarValue::Bool(val_l < val_r),
                            _ => unreachable!()
                        };
                        self.set_val(out_mem, out_val).unwrap();
                        self.ip += 1;
                    } else {
                        unreachable!()
                    }
                }
                "Equal" => {
                    let lh = curr_quad.lh_op.unwrap();
                    let lh_mem: VarValue = self.get_val(lh).unwrap();
                    let rh = curr_quad.rh_op.unwrap();
                    let rh_mem = self.get_val(rh).unwrap();
                    if let OutOp::Mem(out_mem) = curr_quad.out_op {
                        let out_val = match (lh_mem, rh_mem) {
                            (VarValue::Int(val_l), VarValue::Int(val_r)) => VarValue::Bool(val_l == val_r),
                            (VarValue::Float(val_l), VarValue::Int(val_r)) => VarValue::Bool(val_l == val_r as f64),
                            (VarValue::Int(val_l), VarValue::Float(val_r)) => VarValue::Bool((val_l as f64) == val_r),
                            (VarValue::Float(val_l), VarValue::Float(val_r)) => VarValue::Bool(val_l == val_r),
                            _ => unreachable!()
                        };
                        self.set_val(out_mem, out_val).unwrap();
                        self.ip += 1;
                    } else {
                        unreachable!()
                    }
                }
                "NotEqual" => {
                    let lh = curr_quad.lh_op.unwrap();
                    let lh_mem: VarValue = self.get_val(lh).unwrap();
                    let rh = curr_quad.rh_op.unwrap();
                    let rh_mem = self.get_val(rh).unwrap();
                    if let OutOp::Mem(out_mem) = curr_quad.out_op {
                        let out_val = match (lh_mem, rh_mem) {
                            (VarValue::Int(val_l), VarValue::Int(val_r)) => VarValue::Bool(val_l != val_r),
                            (VarValue::Float(val_l), VarValue::Int(val_r)) => VarValue::Bool(val_l != val_r as f64),
                            (VarValue::Int(val_l), VarValue::Float(val_r)) => VarValue::Bool((val_l as f64) != val_r),
                            (VarValue::Float(val_l), VarValue::Float(val_r)) => VarValue::Bool(val_l != val_r),
                            _ => unreachable!()
                        };
                        self.set_val(out_mem, out_val).unwrap();
                        self.ip += 1;
                    } else {
                        unreachable!()
                    }
                }
                "MoreOrEqualThan" => {
                    let lh = curr_quad.lh_op.unwrap();
                    let lh_mem: VarValue = self.get_val(lh).unwrap();
                    let rh = curr_quad.rh_op.unwrap();
                    let rh_mem = self.get_val(rh).unwrap();
                    if let OutOp::Mem(out_mem) = curr_quad.out_op {
                        let out_val = match (lh_mem, rh_mem) {
                            (VarValue::Int(val_l), VarValue::Int(val_r)) => VarValue::Bool(val_l >= val_r),
                            (VarValue::Float(val_l), VarValue::Int(val_r)) => VarValue::Bool(val_l >= val_r as f64),
                            (VarValue::Int(val_l), VarValue::Float(val_r)) => VarValue::Bool((val_l as f64) >= val_r),
                            (VarValue::Float(val_l), VarValue::Float(val_r)) => VarValue::Bool(val_l >= val_r),
                            _ => unreachable!()
                        };
                        self.set_val(out_mem, out_val).unwrap();
                        self.ip += 1;
                    } else {
                        unreachable!()
                    }
                }
                "LessOrEqualThan" => {
                    let lh = curr_quad.lh_op.unwrap();
                    let lh_mem: VarValue = self.get_val(lh).unwrap();
                    let rh = curr_quad.rh_op.unwrap();
                    let rh_mem = self.get_val(rh).unwrap();
                    if let OutOp::Mem(out_mem) = curr_quad.out_op {
                        let out_val = match (lh_mem, rh_mem) {
                            (VarValue::Int(val_l), VarValue::Int(val_r)) => VarValue::Bool(val_l <= val_r),
                            (VarValue::Float(val_l), VarValue::Int(val_r)) => VarValue::Bool(val_l <= val_r as f64),
                            (VarValue::Int(val_l), VarValue::Float(val_r)) => VarValue::Bool((val_l as f64) <= val_r),
                            (VarValue::Float(val_l), VarValue::Float(val_r)) => VarValue::Bool(val_l <= val_r),
                            _ => unreachable!()
                        };
                        self.set_val(out_mem, out_val).unwrap();
                        self.ip += 1;
                    } else {
                        unreachable!()
                    }
                }
                "And" => {
                    let lh = curr_quad.lh_op.unwrap();
                    let lh_mem: VarValue = self.get_val(lh).unwrap();
                    let rh = curr_quad.rh_op.unwrap();
                    let rh_mem = self.get_val(rh).unwrap();
                    if let OutOp::Mem(out_mem) = curr_quad.out_op {
                        let out_val = match (lh_mem, rh_mem) {
                            (VarValue::Bool(val_l), VarValue::Bool(val_r)) => VarValue::Bool(val_l && val_r),
                            _ => unreachable!()
                        };
                        self.set_val(out_mem, out_val).unwrap();
                        self.ip += 1;
                    } else {
                        unreachable!()
                    }
                }
                "Or" => {
                    let lh = curr_quad.lh_op.unwrap();
                    let lh_mem: VarValue = self.get_val(lh).unwrap();
                    let rh = curr_quad.rh_op.unwrap();
                    let rh_mem = self.get_val(rh).unwrap();
                    if let OutOp::Mem(out_mem) = curr_quad.out_op {
                        let out_val = match (lh_mem, rh_mem) {
                            (VarValue::Bool(val_l), VarValue::Bool(val_r)) => VarValue::Bool(val_l || val_r),
                            _ => unreachable!()
                        };
                        self.set_val(out_mem, out_val).unwrap();
                        self.ip += 1;
                    } else {
                        unreachable!()
                    }
                }
                "Print" => {
                    if let OutOp::Mem(mem_loc) = curr_quad.out_op {
                        let mem_data = self.get_val(mem_loc).unwrap();
                        println!("{:?}", mem_data);
                    } else if let OutOp::Str(letrero) = &curr_quad.out_op {
                        println!("{}", letrero);
                    } else {
                        unreachable!();
                    }
                    self.ip += 1;
                }
                "Assign" => {
                    let lh = curr_quad.lh_op.unwrap();
                    let lh_mem: VarValue = self.get_val(lh).unwrap();
                    if let OutOp::Mem(out_mem) = curr_quad.out_op {
                        self.set_val(out_mem, lh_mem).unwrap();
                        self.ip += 1;
                    } else {
                        unreachable!()
                    }
                }
                "EndFunc" => {
                    if self.ip_stack.is_empty() {
                        break;
                    } else {
                        // pop ip_stack, pop mem_stack
                        let last_ip = self.ip_stack.pop().unwrap();
                        let last_mem = self.memory_stack.pop().unwrap();

                        debug!("{:?}", self.curr_memory);

                        self.ip = last_ip + 1;
                        self.curr_memory = last_mem.clone();
                        debug!("{:?}", self.curr_memory);
                    }
                }
                &_ => {
                    error!("Unknown action: {}", curr_quad.op.as_str());
                    unimplemented!("Missing action");
                }
            }
        }
        debug!("{:?}", self.curr_memory);
    }
}