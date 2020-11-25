use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::str::FromStr;
use std::collections::HashMap;

use turtle::Turtle;

use log::{debug, error, log_enabled, info, Level};

use crate::memory::{Memory, VarValue, BaseDirs};

#[derive(Debug, Default)]
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
    prog_name: String,
    func_list: HashMap<String, Func>,
    curr_memory: Memory,
    global_memory: Memory,
    quad_list: Vec<Quadruple>,
    ip: usize,
    ip_stack: Vec<usize>,
    memory_stack: Vec<Memory>,
    constants: HashMap<i32, VarValue>,
}

impl VM {
    pub fn new() -> Self {
        let mut new_vm: VM = Default::default();
        new_vm.func_list.insert("Center".to_string(), Default::default());
        new_vm.func_list.insert("Forward".to_string(), Func { locals: (0, 1, 0), ..Default::default() });
        new_vm.func_list.insert("Backward".to_string(), Func { locals: (0, 1, 0), ..Default::default() });
        new_vm.func_list.insert("Left".to_string(), Func { locals: (0, 1, 0), ..Default::default() });
        new_vm.func_list.insert("Right".to_string(), Func { locals: (0, 1, 0), ..Default::default() });
        new_vm.func_list.insert("PenUp".to_string(), Default::default());
        new_vm.func_list.insert("PenDown".to_string(), Default::default());
        new_vm.func_list.insert("Color".to_string(), Func { locals: (0, 3, 0), ..Default::default() });
        new_vm.func_list.insert("Size".to_string(), Func { locals: (0, 1, 0), ..Default::default() });
        new_vm.func_list.insert("Clear".to_string(), Default::default());
        new_vm.func_list.insert("Position".to_string(), Func { locals: (0, 2, 0), ..Default::default() });
        new_vm.func_list.insert("BackgroundColor".to_string(), Func { locals: (0, 3, 0), ..Default::default() });
        new_vm.func_list.insert("StartFill".to_string(), Default::default());
        new_vm.func_list.insert("EndFill".to_string(), Default::default());
        new_vm.func_list.insert("FillColor".to_string(), Func { locals: (0, 3, 0), ..Default::default() });

        new_vm
    }

    // read .obj file and create quadruples, globals, and function table
    pub fn load_file(&mut self, file_name: &str) -> io::Result<()> {
        let file = File::open(file_name)?;
        let lines = BufReader::new(file).lines();

        // Constants
        for line in lines {
            if let Ok(line) = line {
                let mut info = line.split_whitespace();
                match info.next().unwrap() {
                    "P" => {
                        let prog_name = info.next().unwrap();
                        info!("Program {}", prog_name);
                        self.prog_name = prog_name.to_string();
                    }
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
                            "Mod" |
                            "MoreThan" | 
                            "LessThan" |
                            "LessOrEqualThan" |
                            "MoreOrEqualThan" |
                            "Equal" |
                            "NotEqual" |
                            "And" |
                            "Or" |
                            "Assign" | 
                            "Return" => {
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
                            "Param" | "Read" => {
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
                            "EndFunc" | 
                            "EndFuncS" |
                            "Forward" |
                            "Backward" |
                            "Left" |
                            "Right" |
                            "Center" |
                            "PenUp" |
                            "PenDown" |
                            "Color" |
                            "Size" |
                            "Clear" |
                            "Position" |
                            "BackgroundColor" |
                            "StartFill" |
                            "EndFill" |
                            "FillColor" => { // Special function because out is empty
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
        // debug!("{:?}", self);

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
            debug!("Setting {} to {:?}", location, new_val);
            self.global_memory.set_val(location, new_val).unwrap()
        } else if location >= BaseDirs::LocalInt as i32 && location < BaseDirs::TempUpperLim as i32 {
            self.curr_memory.set_val(location, new_val).unwrap()
        } else {
            return Err(format!("Memory location {} not initialized", location))
        };
        Ok(())
    }
    
    // Returns the type of value that is saved at a certain memory location
    fn get_mem_type(&self, location: i32) -> VarValue {
        if location >= BaseDirs::GlobalInt as i32 && location < BaseDirs::GlobalFloat as i32 - 1 ||
        location >= BaseDirs::LocalInt as i32 && location < BaseDirs::LocalFloat as i32 - 1 ||
        location >= BaseDirs::TempInt as i32 && location < BaseDirs::TempFloat as i32 - 1 ||
        location >= BaseDirs::CteInt as i32 && location < BaseDirs::CteFloat as i32 - 1 {
            VarValue::Int(0)
        } else if location >= BaseDirs::GlobalFloat as i32 && location < BaseDirs::GlobalChar as i32 - 1 ||
        location >= BaseDirs::LocalFloat as i32 && location < BaseDirs::LocalChar as i32 - 1 ||
        location >= BaseDirs::TempFloat as i32 && location < BaseDirs::TempChar as i32 - 1 ||
        location >= BaseDirs::CteFloat as i32 && location < BaseDirs::CteChar as i32 - 1 {
            VarValue::Float(0.0)
        } else {
            VarValue::Char("".to_string())
        }
    }
    
    pub fn run(&mut self) {
        let mut turtle = Turtle::new();
        turtle.drawing_mut().set_title(self.prog_name.as_str());
        // turtle.drawing_mut().set_size([400, 400]);
        let mut new_mem: Memory = Default::default();
        let mut param_pos: (i32, i32, i32) = (0,0,0);

        // initialize main memory

        let func_data = self.func_list.get("main").unwrap();
        self.curr_memory.set_new_func(func_data.locals, func_data.temps);
        new_mem.set_new_func(func_data.locals, func_data.temps);

        debug!("Global mem: {:?}", self.global_memory);

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
                        // debug!("Param init, mem: {:?}, {:?}", new_mem, self.global_memory);
                        match param_val {
                            VarValue::Int(_) => {
                                new_mem.set_val(BaseDirs::LocalInt as i32 + param_pos.0, param_val).unwrap();
                                param_pos.0 += 1;
                            }
                            VarValue::Float(_) => {
                                new_mem.set_val(BaseDirs::LocalFloat as i32 + param_pos.1, param_val).unwrap();
                                param_pos.1 += 1;
                            }
                            VarValue::Char(_) => {
                                new_mem.set_val(BaseDirs::LocalChar as i32 + param_pos.2, param_val).unwrap();
                                param_pos.2 += 1;
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
                "EndFuncS" => {
                    param_pos = (0,0,0);
                    self.ip += 1;
                }
                "Gosub" => {
                    self.memory_stack.push(self.curr_memory.clone());

                    self.curr_memory = new_mem.clone();
                    if let OutOp::Str(func_name) = &curr_quad.out_op {
                        let func: &Func = self.func_list.get(func_name).unwrap();
                        self.ip_stack.push(self.ip.clone());
                        self.ip = func.start_loc;
                        param_pos = (0,0,0);

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
                "Mod" => {
                    let lh = curr_quad.lh_op.unwrap();
                    let lh_mem: VarValue = self.get_val(lh).unwrap();
                    let rh = curr_quad.rh_op.unwrap();
                    let rh_mem = self.get_val(rh).unwrap();
                    if let OutOp::Mem(out_mem) = curr_quad.out_op {
                        let out_val = match (lh_mem, rh_mem) {
                            (VarValue::Int(val_l), VarValue::Int(val_r)) => VarValue::Int(val_l % val_r),
                            (VarValue::Float(val_l), VarValue::Int(val_r)) => VarValue::Float(val_l % val_r as f64),
                            (VarValue::Int(val_l), VarValue::Float(val_r)) => VarValue::Float(val_l as f64 % val_r),
                            (VarValue::Float(val_l), VarValue::Float(val_r)) => VarValue::Float(val_l % val_r),
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
                        debug!("Less Than {:?} {:?}", lh_mem, rh_mem);
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
                            (VarValue::Char(val_l), VarValue::Char(val_r)) => VarValue::Bool(val_l == val_r),
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
                            (VarValue::Char(val_l), VarValue::Char(val_r)) => VarValue::Bool(val_l != val_r),
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
                        println!("{}", mem_data);
                    } else if let OutOp::Str(letrero) = &curr_quad.out_op {
                        println!("{}", letrero);
                    } else {
                        unreachable!();
                    }
                    self.ip += 1;
                }
                "Read" => {
                    if let OutOp::Mem(mem_loc) = curr_quad.out_op {
                        let mut input = String::new();
                        match io::stdin().read_line(&mut input) {
                            Ok(n) => {
                                match self.get_mem_type(mem_loc) {
                                    VarValue::Int(_) => { 
                                        match input.trim().parse::<i32>() {
                                            Ok(val) => { self.set_val(mem_loc, VarValue::Int(val)).unwrap() },
                                            Err(err) => panic!("Value cannot be parsed into int {}", input)
                                        }
                                    }
                                    VarValue::Float(_) => { 
                                        match input.trim().parse::<f64>() {
                                            Ok(val) => { self.set_val(mem_loc, VarValue::Float(val)).unwrap() },
                                            Err(err) => panic!("Value cannot be parsed into float {}.", input)
                                        }
                                        
                                    }
                                    VarValue::Char(_) => {
                                        if input.trim().len() == 1 {
                                            self.set_val(mem_loc, VarValue::Char(input)).unwrap();
                                        } else {
                                            panic!("Char must be single character");
                                        }
                                    }
                                    _ => { unreachable!() }
                                }
                            }
                            Err(error) => println!("error: {}", error),
                        }
                        self.ip += 1;
                    } else {
                        unreachable!()
                    }
                }
                "Assign" => {
                    let lh = curr_quad.lh_op.unwrap();
                    let lh_mem: VarValue = self.get_val(lh).unwrap();
                    if let OutOp::Mem(out_mem) = curr_quad.out_op {
                        match (self.get_mem_type(out_mem), lh_mem.clone()) {
                            (VarValue::Int(_), VarValue::Int(_)) => self.set_val(out_mem, lh_mem).unwrap(),
                            (VarValue::Int(_), VarValue::Float(val)) => self.set_val(out_mem, VarValue::Int(val as i32)).unwrap(),
                            (VarValue::Float(_), VarValue::Int(val)) => self.set_val(out_mem, VarValue::Float(val as f64)).unwrap(),
                            (VarValue::Float(_), VarValue::Float(_)) => self.set_val(out_mem, lh_mem).unwrap(),
                            (VarValue::Char(_), VarValue::Int(val)) => self.set_val(out_mem, VarValue::Char(val.to_string())).unwrap(),
                            (VarValue::Char(_), VarValue::Float(val)) => self.set_val(out_mem, VarValue::Char(val.to_string())).unwrap(),
                            (VarValue::Char(_), VarValue::Char(_)) => self.set_val(out_mem, lh_mem).unwrap(),
                            _=> unimplemented!("WHAT DID YOU DO?!?!")

                        }
                        self.ip += 1;
                    } else {
                        unreachable!()
                    }
                }
                "Center" => {
                    debug!("Param center");
                    turtle.home();
                    self.ip += 1;
                }
                "Forward" => {
                    if let VarValue::Float(distance) = new_mem.get_val(BaseDirs::LocalFloat as i32).unwrap() {
                        debug!("Param forward {:?}", distance);
                        turtle.forward(distance);
                        self.ip += 1;
                    } else {
                        unreachable!("Value must be float")
                    }
                }
                "Backward" => {
                    if let VarValue::Float(distance) = new_mem.get_val(BaseDirs::LocalFloat as i32).unwrap() {
                        debug!("Param backward {}", distance);
                        turtle.backward(distance);
                        self.ip += 1;
                    } else {
                        unreachable!("Value must be float")
                    }
                }
                "Left" => {
                    if let VarValue::Float(angle) = new_mem.get_val(BaseDirs::LocalFloat as i32).unwrap() {
                        debug!("Param left {:?}", angle);
                        turtle.left(angle);
                        self.ip += 1;
                    } else {
                        unreachable!("Value must be float")
                    }
                }
                "Right" => {
                    if let VarValue::Float(angle) = new_mem.get_val(BaseDirs::LocalFloat as i32).unwrap() {
                        debug!("Param right {}", angle);
                        turtle.right(angle);
                        self.ip += 1;
                    } else {
                        unreachable!("Value must be float")
                    }
                }
                "PenUp" => {
                    debug!("Param penup");
                    turtle.pen_up();
                    self.ip += 1;
                }
                "PenDown" => {
                    debug!("Param pendown");
                    turtle.pen_down();
                    self.ip += 1;
                }
                "Clear" => {
                    debug!("Param clear");
                    turtle.reset();
                    self.ip += 1;
                }
                "Size" => {
                    if let VarValue::Float(size) = new_mem.get_val(BaseDirs::LocalFloat as i32).unwrap() {
                        debug!("Param size {}", size);
                        turtle.set_pen_size(size);
                        self.ip += 1;
                    } else {
                        unreachable!("Value must be float")
                    }
                }
                "Position" => {
                    let x = new_mem.get_val(BaseDirs::LocalFloat as i32).unwrap();
                    let y = new_mem.get_val(BaseDirs::LocalFloat as i32 + 1).unwrap();
                    match (x, y) {
                        (VarValue::Float(x), VarValue::Float(y)) => {
                            turtle.go_to([x, y]);
                            self.ip += 1;
                        }
                        _=> { unreachable!("RGB must be floats")}
                    }
                }
                "Color" => {
                    let red = new_mem.get_val(BaseDirs::LocalFloat as i32).unwrap();
                    let blue = new_mem.get_val(BaseDirs::LocalFloat as i32 + 1).unwrap();
                    let green = new_mem.get_val(BaseDirs::LocalFloat as i32 + 2).unwrap();
                    match (red, blue, green) {
                        (VarValue::Float(red), VarValue::Float(blue), VarValue::Float(green)) => {
                            turtle.set_pen_color(turtle::Color::rgb(red, green, blue));
                            self.ip += 1;
                        }
                        _=> { unreachable!("RGB must be floats")}
                    }
                }
                "BackgroundColor" => {
                    let red = new_mem.get_val(BaseDirs::LocalFloat as i32).unwrap();
                    let blue = new_mem.get_val(BaseDirs::LocalFloat as i32 + 1).unwrap();
                    let green = new_mem.get_val(BaseDirs::LocalFloat as i32 + 2).unwrap();
                    match (red, blue, green) {
                        (VarValue::Float(red), VarValue::Float(blue), VarValue::Float(green)) => {
                            turtle.drawing_mut().set_background_color(turtle::Color::rgb(red, green, blue));
                            self.ip += 1;
                        }
                        _=> { unreachable!("RGB must be floats")}
                    }
                }
                "FillColor" => {
                    let red = new_mem.get_val(BaseDirs::LocalFloat as i32).unwrap();
                    let blue = new_mem.get_val(BaseDirs::LocalFloat as i32 + 1).unwrap();
                    let green = new_mem.get_val(BaseDirs::LocalFloat as i32 + 2).unwrap();

                    debug!("Setting fill color {:?} {:?} {:?}", red, blue, green);
                    match (red, blue, green) {
                        (VarValue::Float(red), VarValue::Float(blue), VarValue::Float(green)) => {
                            turtle.set_fill_color(turtle::Color::rgb(red, green, blue));
                            self.ip += 1;
                        }
                        _=> { unreachable!("RGB must be floats")}
                    }
                }
                "StartFill" => {
                    turtle.begin_fill();
                    self.ip += 1;
                }
                "EndFill" => {
                    turtle.end_fill();
                    self.ip += 1;
                }
                "EndFunc" => {
                    if self.ip_stack.is_empty() {
                        break;
                    } else {
                        // pop ip_stack, pop mem_stack
                        let last_ip = self.ip_stack.pop().unwrap();
                        let last_mem = self.memory_stack.pop().unwrap();

                        debug!("Before mem {:?}", self.curr_memory);

                        self.ip = last_ip + 1;
                        self.curr_memory = last_mem.clone();
                        debug!("after mem {:?}", self.curr_memory);
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