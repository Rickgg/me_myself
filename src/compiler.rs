use pest::Parser;
use std::fs;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::fmt;
use std::fs::File;
use std::io::prelude::*;

use log::{debug, error, log_enabled, info, Level, warn};

use crate::memory::BaseDirs;

#[derive(Parser)]
#[grammar = "memyself.pest"]
pub struct MMIParser;

#[derive(Debug, PartialEq, Copy, Clone)]
enum VarType {
  Int,
  Float,
  Char,
  Bool,
  Void
}

impl Default for VarType {
    fn default() -> Self { VarType::Void }
}

#[derive(Debug, Clone)]
struct Var {
    Type: VarType,
    Location: String
}

type VarHash = HashMap<String, Var>;

#[derive(Debug, Default)]
struct Func {
    name: String,
    ret_type: VarType,
    var_table: HashMap<String, Var>,
    param_list: Vec<VarType>,
    start_loc: usize,
    local_vars: (i32, i32, i32), // int, float, char
    temp_vars: (i32, i32, i32, i32) // int, float, char, bool
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum Actions {
    Sum, // Done
    Sub, // Done
    Mult, // Done
    Div, // Done
    MoreThan, // Done
    LessThan, // Done
    MoreOrEqualThan, // Done
    LessOrEqualThan, // Done
    Equal, // Done
    NotEqual, // Done
    And, // Done
    Or, // Done
    ParentStart, // not necessary, only for compiler
    Print, // done
    Read,
    GotoF, // Done
    Goto, // Done
    Era, // Done
    EndFunc,
    Assign, // Done
    Gosub,
    Param,
    Return,
    // MISSING IMPLEMENTATION
    Center,
    Forward,
    Backward,
    Left,
    Right,
    Point,
    Circle,
    Arc,
    PenUp,
    PenDown,
    Color,
    Size,
    Clear,
    // MAYBE?
}

// #[derive(Debug)]
struct Quadruple {
  op: Actions,
  lh_op: Option<Var>,
  rh_op: Option<Var>,
  out_op: Var
}

impl fmt::Display for Quadruple {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let lh_op = match &self.lh_op {
            Some(var) => format!("[{} {:?}]", var.Location, var.Type),
            None => format!("_")
        };
        let rh_op = match &self.rh_op {
            Some(var) => format!("[{} {:?}]", var.Location, var.Type),
            None => format!("_")
        };
        let out_op = format!("[{} {:?}]", self.out_op.Location, self.out_op.Type);
        write!(f, "{:?} {} {} {}", self.op, lh_op, rh_op, out_op)
    }
}

fn semantic_cube(action: Actions, l_op: &Option<Var>, r_op: &Option<Var>) -> Result<VarType, String> {
    // debug!("{:?} {:?} {:?}", action, l_op, r_op);
    let l_op = match l_op {
        Some(x) => x.Type,
        None => return Err(String::from("Variable izq faltante"))
    };
    let r_op = match r_op {
        Some(x) => x.Type,
        None => return Err(String::from("Variable der faltante"))
    };
    match action {
        Actions::Sum | 
        Actions::Sub | 
        Actions::Mult | 
        Actions::Div => {
            if l_op == VarType::Int && r_op == VarType::Int { Ok(VarType::Int) }
            else if l_op == VarType::Float && r_op == VarType::Int || 
            l_op == VarType::Int && r_op == VarType::Float || 
            l_op == VarType::Float && r_op == VarType::Float {
                Ok(VarType::Float)
            }
            else {
                return Err(format!("Incompatible types: {:?} and {:?}, {:?}", l_op, r_op, action));
            }
        },
        Actions::Assign => {
            if l_op == VarType::Int && r_op == VarType::Int { Ok(VarType::Int)}
            else if l_op == VarType::Float && r_op == VarType::Float ||
                l_op == VarType::Float && r_op == VarType::Int { Ok(VarType::Float)}
            else {
                return Err(format!("Incompatible types: {:?} and {:?}, {:?}", l_op, r_op, action));
            }
        }
        Actions::Equal |
        Actions::LessThan |
        Actions::MoreThan |
        Actions::NotEqual |
        Actions::LessOrEqualThan |
        Actions::MoreOrEqualThan => {
            if l_op == VarType::Int && r_op == VarType::Int ||
            l_op == VarType::Float && r_op == VarType::Float ||
            l_op == VarType::Int && r_op == VarType::Float ||
            l_op == VarType::Float && r_op == VarType::Int{
                Ok(VarType::Bool)
            } else {
                return Err(format!("Incompatible types: {:?} and {:?}, {:?}", l_op, r_op, action));
            }
        },
        Actions::And |
        Actions::Or => {
            if l_op == VarType::Bool && r_op == VarType::Bool { Ok(VarType::Bool) }
            else {
                return Err(format!("Incompatible types: {:?} and {:?}, {:?}", l_op, r_op, action));
            }
        }
        _ => { unreachable!(); }
    }
}

#[derive(Debug)]
struct Constant {
    Value: String,
    Type: VarType,
    Location: String
}

#[derive(Default)]
pub struct MMCompiler {
    op_vec: Vec<Var>,
    oper_vec: Vec<Actions>,
    jump_vec: Vec<usize>,
    quadruples: Vec<Quadruple>,
    global_vars: HashMap<String, Var>,
    local_vars: HashMap<String, Var>,
    constants: Vec<Constant>,
    function_table: HashMap<String, Func>,
    special_functions: HashMap<String, Vec<VarType>>,
    current_func: String,

    // Memory position counters
    global_locs: (i32, i32, i32),
    cte_locs: (i32, i32, i32)
}

impl MMCompiler {
    pub fn new() -> MMCompiler {
        let mut new_comp: MMCompiler = Default::default();

        new_comp.special_functions.insert("Center".to_string(), vec![]);
        new_comp.special_functions.insert("Forward".to_string(), vec![VarType::Float]);
        new_comp.special_functions.insert("Backward".to_string(), vec![VarType::Float]);
        new_comp.special_functions.insert("Left".to_string(), vec![VarType::Float]);
        new_comp.special_functions.insert("Right".to_string(), vec![VarType::Float]);
        // new_comp.special_functions.insert("Point".to_string(), vec![VarType::Float]);
        new_comp.special_functions.insert("Circle".to_string(), vec![VarType::Float]);
        new_comp.special_functions.insert("Arc".to_string(), vec![VarType::Float]);
        new_comp.special_functions.insert("PenUp".to_string(), vec![]);
        new_comp.special_functions.insert("PenDown".to_string(), vec![]);
        new_comp.special_functions.insert("Color".to_string(), vec![VarType::Float, VarType::Float, VarType::Float]);
        new_comp.special_functions.insert("Size".to_string(), vec![VarType::Float]);
        new_comp.special_functions.insert("Clear".to_string(), vec![]);


        new_comp
    }

    fn gen_quad (&mut self, action: Actions, lh_op: Option<Var>, rh_op: Option<Var>, out_op: Var) {
        let new_quad = Quadruple {
            op: action,
            lh_op,
            rh_op,
            out_op
        };
        self.quadruples.push(new_quad);
    }

    fn fill_goto(&mut self, pos: usize) {
        let jump_pos = self.jump_vec.pop().unwrap();
        if let Some(quad) = self.quadruples.get_mut(jump_pos) {
            quad.out_op.Location = pos.to_string();
        }
    }

    fn find_var(&self, var_name: &str) -> Result<Var, String> {
        // Get var from var_table and compare types
        let curr_fun = self.function_table.get(self.current_func.as_str()).unwrap();
        if let Some(var) = curr_fun.var_table.get(var_name) {
            Ok(var.clone())
        } else if let Some(var) = self.global_vars.get(var_name) { 
            Ok(var.clone())
        } else {
            return Err(String::from(format!("Variable {} not declared yet.", var_name)));
        }
    }

    fn process_vars(&mut self, data: pest::iterators::Pair<Rule>, scope_global: bool) -> Result<VarHash, String> {
        let mut current_tipo: &str = "";
        let mut var_map: VarHash = HashMap::new();

        for var in data.into_inner() {
            match var.as_rule() {
                Rule::tipo => {
                    current_tipo = var.as_str();
                }
                Rule::id => {
                    let value = match current_tipo {
                        "int" => { 
                            let new_loc = if scope_global {
                                let new_loc = BaseDirs::GlobalInt as i32 + self.global_locs.0;
                                self.global_locs.0 += 1;
                                new_loc
                            } else {
                                let mut current_func = self.function_table.get_mut(self.current_func.as_str()).unwrap();
                                let new_loc = BaseDirs::LocalInt as i32 + current_func.local_vars.0;
                                current_func.local_vars.0 += 1;
                                new_loc
                            };

                            (VarType::Int, new_loc)
                        }
                        "float" =>  {
                            let new_loc = if scope_global {
                                let new_loc = BaseDirs::GlobalFloat as i32 + self.global_locs.1;
                                self.global_locs.1 += 1;
                                new_loc
                            } else {
                                let mut current_func = self.function_table.get_mut(self.current_func.as_str()).unwrap();
                                let new_loc = BaseDirs::LocalFloat as i32 + current_func.local_vars.1;
                                current_func.local_vars.1 += 1;
                                new_loc
                            };

                            (VarType::Float, new_loc)
                        },
                        "char" => {
                            let new_loc = if scope_global {
                                let new_loc = BaseDirs::GlobalChar as i32 + self.global_locs.2;
                                self.global_locs.2 += 1;
                                new_loc
                            } else {
                                let mut current_func = self.function_table.get_mut(self.current_func.as_str()).unwrap();
                                let new_loc = BaseDirs::LocalChar as i32 + current_func.local_vars.2;
                                current_func.local_vars.2 += 1;
                                new_loc
                            };

                            (VarType::Char, new_loc)
                        },
                        _ => { unreachable!() }
                    };

                    match var_map.entry(var.as_str().to_string()) {
                        Vacant(entry) => { entry.insert(Var { Location: value.1.to_string(), Type: value.0 }); }
                        Occupied(_) => return Err(format!("Variable {} has already been declared", var.as_str()))
                    }
                }
                _ => {}
            }
        }

        Ok(var_map)
    }

    fn process_factor(&mut self, data: pest::iterators::Pair<Rule>) -> Result<(), String> {
        // println!("Factor {}", data.as_str());
        for field in data.into_inner(){
            match field.as_rule() {
                Rule::llamada_op => {
                    // Add "parenthesis" to give more precedence to functions :D
                    self.oper_vec.push(Actions::ParentStart);
                    let mut llamada_fields = field.into_inner();
                    // Solve arguments, call expresion, assign from global to temporal and continue

                    let func_name = llamada_fields.next().unwrap().as_str();

                    if !self.function_table.contains_key(func_name) {
                        return Err(format!("Function {} is being called but has not been declared.", func_name).to_string());
                    }

                    let param_list = self.function_table.get(func_name).unwrap().param_list.clone();

                    self.gen_quad(Actions::Era, None, None, Var { Location: func_name.to_string(), Type: VarType::Void });

                    if let Some(args) = llamada_fields.next() {
                        let mut param_count = 0;
                        for (i, arg) in args.into_inner().enumerate() {
                            self.process_expresion(arg)?;
                            // POP from op_vec
                            let param = self.op_vec.pop().unwrap().clone();

                            if i >= param_list.len() {
                                return Err(format!("Wrong number of arguments. Expected: {}. Got: {}", param_list.len(), param_count + 1));
                            }

                            if param_list[i] != param.Type {
                                return Err(format!("Parameter {} in call of {} is of incompatible types. Expected: {:?}. Got: {:?}", i, func_name, param_list[i].clone(), param.Type.clone()));
                            }

                            self.gen_quad(Actions::Param, None, None, param);
                            param_count += 1;
                        }
                        if param_count < param_list.len() {
                            return Err(format!("Wrong number of arguments. Expected: {}. Got: {}", param_list.len(), param_count));
                        }
                    }

                    self.gen_quad(Actions::Gosub, None, None, Var { Location: func_name.to_string(), Type: VarType::Void });

                    // Left hand: global function location, out: temporal
                    let lh_op = self.global_vars.get(func_name).unwrap().clone();
                    
                    let calling_func_type = self.function_table.get(func_name).unwrap().ret_type;

                    let current_func = self.function_table.get_mut(&self.current_func).unwrap();

                    let temp_loc = match calling_func_type {
                            VarType::Int => {
                                let new_loc = BaseDirs::TempInt as i32 + current_func.temp_vars.0; 
                                current_func.temp_vars.0 += 1; 
                                new_loc 
                            }
                            VarType::Float => { 
                                let new_loc = BaseDirs::TempFloat as i32 + current_func.temp_vars.1; 
                                current_func.temp_vars.1 += 1; 
                                new_loc }
                            VarType::Char => { 
                                let new_loc = BaseDirs::TempChar as i32 + current_func.temp_vars.2; 
                                current_func.temp_vars.0 += 2;
                                new_loc 
                            }
                            _ => { unreachable!() }
                        };
                    let new_loc = temp_loc;
                    let ret_type = current_func.ret_type;
                    drop(temp_loc);

                    self.op_vec.push(Var { Type: calling_func_type, Location: new_loc.to_string() });
                    self.gen_quad(Actions::Assign, Some(lh_op), None, Var { Type: ret_type, Location: new_loc.to_string() });

                    self.oper_vec.pop();
                },
                Rule::expresion => {
                    self.oper_vec.push(Actions::ParentStart);
                    self.process_expresion(field)?;
                    self.oper_vec.pop();
                },
                Rule::var_cte => {
                    let cte = field.into_inner().next().unwrap();
                    // here get type as_rule
                    let new_cte = match cte.as_rule() {
                        Rule::id => { // get type from var table
                            let var_data = self.find_var(cte.as_str())?;
                            (var_data.Type, var_data.Location)
                        }
                        Rule::float => {
                            let const_loc = BaseDirs::CteFloat as i32 + self.cte_locs.0;
                            self.cte_locs.0 += 1;
                            self.constants.push(Constant { Location: const_loc.to_string(),Value: cte.as_str().to_string(), Type: VarType::Float});
                            (VarType::Float, const_loc.to_string())
                        }
                        Rule::int => {
                            let const_loc = BaseDirs::CteInt as i32 + self.cte_locs.1;
                            self.cte_locs.1 += 1;
                            self.constants.push(Constant { Location: const_loc.to_string(), Value: cte.as_str().to_string(), Type: VarType::Int});
                            (VarType::Int, const_loc.to_string())
                        }
                        _=> { unreachable!() }
                    };
                    self.op_vec.push(Var { Type: new_cte.0, Location: new_cte.1 });
                    // println!("{:?} {:?}", cte.as_rule(), cte.as_str());
                }
                _=>{}
            }
        }
        Ok(())
    }

    fn process_termino(&mut self, data: pest::iterators::Pair<Rule>) -> Result<(), String> {
        // println!("Termino {}", data.as_str());

        for field in data.into_inner() {
            match field.as_rule() {
                Rule::factor => {
                    self.process_factor(field)?;
                    if let Some(val) = self.oper_vec.last().cloned() {
                        if val == Actions::Mult || val == Actions::Div {
                            self.oper_vec.pop();
                            let rh_op = self.op_vec.pop();
                            let lh_op = self.op_vec.pop();
                            let out_type = semantic_cube(val, &lh_op, &rh_op)?;
                            if out_type != VarType::Int && out_type != VarType::Float {
                                return Err(String::from("Incompatible types, se necesita numérico para comparación"));
                            }
                            let mut current_func = self.function_table.get_mut(self.current_func.as_str()).unwrap();

                            let temp_loc = match out_type {
                                VarType::Int => {
                                    let new_loc = BaseDirs::TempInt as i32 + current_func.temp_vars.0; 
                                    current_func.temp_vars.0 += 1; 
                                    new_loc 
                                }
                                VarType::Float => { 
                                    let new_loc = BaseDirs::TempFloat as i32 + current_func.temp_vars.1; 
                                    current_func.temp_vars.1 += 1; 
                                    new_loc }
                                VarType::Char => { 
                                    let new_loc = BaseDirs::TempChar as i32 + current_func.temp_vars.2; 
                                    current_func.temp_vars.0 += 2;
                                    new_loc }
                                _ => { unreachable!() }
                            };

                            self.op_vec.push(Var { Location: temp_loc.to_string(), Type: out_type });
                            self.gen_quad(val, lh_op, rh_op, Var { Location: temp_loc.to_string(), Type: out_type });
                        }
                    }
                }
                Rule::fact_op => {
                    match field.as_str() {
                        "*" => self.oper_vec.push(Actions::Mult),
                        "/" => self.oper_vec.push(Actions::Div),
                        &_ => {}
                    };
                }
                Rule::termino => {
                    self.process_termino(field)?;
                }
                _=>{}
            }
        }
        Ok(())
    }

    fn process_exp(&mut self, data: pest::iterators::Pair<Rule>) -> Result<(), String> {
        // println!("Exp {}", data.as_str());
        for field in data.into_inner() {
            match field.as_rule() {
                Rule::termino => { 
                    self.process_termino(field)?;
                    if let Some(val) = self.oper_vec.last().cloned(){
                        if val == Actions::Sum || val == Actions::Sub {
                            self.oper_vec.pop();
                            let rh_op = self.op_vec.pop();
                            let lh_op = self.op_vec.pop();
                            let out_type = semantic_cube(val, &lh_op, &rh_op)?;
                            if out_type != VarType::Int && out_type != VarType::Float {
                                return Err(String::from("Incompatible types, se necesita numérico para suma o resta"));
                            }
                            let mut current_func = self.function_table.get_mut(self.current_func.as_str()).unwrap();

                            let temp_loc = match out_type {
                                VarType::Int => { 
                                    let new_loc = BaseDirs::TempInt as i32 + current_func.temp_vars.0; 
                                    current_func.temp_vars.0 += 1; 
                                    new_loc 
                                }
                                VarType::Float => { 
                                    let new_loc = BaseDirs::TempInt as i32 + current_func.temp_vars.1; 
                                    current_func.temp_vars.1 += 1; 
                                    new_loc }
                                VarType::Char => { 
                                    let new_loc = BaseDirs::TempInt as i32 + current_func.temp_vars.2; 
                                    current_func.temp_vars.0 += 2;
                                    new_loc }
                                _ => { unreachable!() }
                            };
                            self.op_vec.push(Var { Location: temp_loc.to_string(), Type: out_type });
                            self.gen_quad(val, lh_op, rh_op, Var { Location: temp_loc.to_string(), Type: out_type });
                        }
                    }
                }
                Rule::op => { 
                    match field.as_str() {
                        "+" => self.oper_vec.push(Actions::Sum),
                        "-" => self.oper_vec.push(Actions::Sub),
                        &_ => {}
                    };
                }
                Rule::exp => { 
                    self.process_exp(field)?;
                }
                _=>{}
            }
        }
        Ok(())
    }

    fn process_exp_comp(&mut self, data: pest::iterators::Pair<Rule>) -> Result<(), String> {

        for field in data.into_inner() {
            match field.as_rule() {
                Rule::exp => {
                    self.process_exp(field)?;
                    if let Some(val) = self.oper_vec.last().cloned() {
                        if val == Actions::MoreThan || val == Actions::LessThan || val == Actions::Equal || val == Actions::NotEqual || val == Actions::MoreOrEqualThan || val == Actions::LessOrEqualThan {
                            self.oper_vec.pop();
                            let rh_op = self.op_vec.pop();
                            let lh_op = self.op_vec.pop();
                            let out_type = semantic_cube(val, &lh_op, &rh_op)?;
                            if out_type != VarType::Bool {
                                return Err(String::from("Incompatible types, se necesita booleano para comparación"));
                            }

                            let mut current_func = self.function_table.get_mut(self.current_func.as_str()).unwrap();

                            let temp_loc = BaseDirs::TempBool as i32 + current_func.temp_vars.3;
                            current_func.temp_vars.3 += 1;

                            self.op_vec.push(Var { Location: temp_loc.to_string(), Type: out_type });
                            self.gen_quad(val, lh_op, rh_op, Var { Location: temp_loc.to_string(), Type: out_type });
                        }
                    }
                },
                Rule::comp => {
                    match field.as_str() {
                        ">" => self.oper_vec.push(Actions::MoreThan),
                        "<" => self.oper_vec.push(Actions::LessThan),
                        "==" => self.oper_vec.push(Actions::Equal),
                        "<>" => self.oper_vec.push(Actions::NotEqual),
                        "<=" => self.oper_vec.push(Actions::LessOrEqualThan),
                        ">=" => self.oper_vec.push(Actions::MoreOrEqualThan),
                        _ => {}
                    };
                },
                _ => {}
            }
        }
        Ok(())
    }

    fn process_expresion(&mut self, data: pest::iterators::Pair<Rule>) -> Result<(), String> {    

        for field in data.into_inner() {
            match field.as_rule() {
                Rule::exp_comp => {
                    self.process_exp_comp(field)?;
                    if let Some(val) = self.oper_vec.last().cloned() {
                        if val == Actions::And || val == Actions::Or {
                            self.oper_vec.pop();
                            let rh_op = self.op_vec.pop();
                            let lh_op = self.op_vec.pop();
                            let out_type = semantic_cube(val, &lh_op, &rh_op)?;
                            if out_type != VarType::Bool {
                                return Err(String::from("Incompatible types, se necesita booleano para comparación"));
                            }

                            let mut current_func = self.function_table.get_mut(self.current_func.as_str()).unwrap();
                            let temp_loc = BaseDirs::TempBool as i32 + current_func.temp_vars.3;
                            current_func.temp_vars.3 += 1;

                            self.op_vec.push(Var { Location: temp_loc.to_string(), Type: out_type.clone() });
                            self.gen_quad(val, lh_op, rh_op, Var { Location: temp_loc.to_string(), Type: out_type });
                        }
                    }
                }
                Rule::cond => {
                    match field.as_str() {
                        "&" => self.oper_vec.push(Actions::And),
                        "||" => self.oper_vec.push(Actions::Or),
                        &_ => {}
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn process_statute(&mut self, data: pest::iterators::Pair<Rule>) -> Result<(), String> {
        let mut fields = data.into_inner();
        
        let estatuto = fields.next().unwrap();
        match estatuto.as_rule() {
            Rule::asignacion => {
                let mut assign_fields = estatuto.into_inner();
                let var_name = assign_fields.next().unwrap().as_str();
                self.process_expresion(assign_fields.next().unwrap())?;
                
                let lh_op = self.op_vec.pop();

                let final_var = self.find_var(var_name)?;

                semantic_cube(Actions::Assign, &Some(final_var.clone()), &lh_op)?;
                self.gen_quad(Actions::Assign, lh_op, None, final_var);
            },
            Rule::retorno => {
                let mut return_fields = estatuto.into_inner();
                let expresion = return_fields.next().unwrap();

                self.process_expresion(expresion)?;
                let out_op = self.op_vec.pop().unwrap();
                let func: &Func = self.function_table.get(self.current_func.as_str()).unwrap();
                let return_loc = self.global_vars.get(self.current_func.as_str()).unwrap();
                if func.ret_type != out_op.Type {
                    return Err("Return types are different.".to_string());
                }
                self.gen_quad(Actions::Return, Some(out_op), None, return_loc.clone());
            },
            Rule::lectura => {
                let read_fields = estatuto.into_inner();
                for field in read_fields {
                    let var = self.find_var(field.as_str())?;
                    self.gen_quad(Actions::Read, None, None, var);
                }
            },
            Rule::decision => { // IF
                let decision_fields = estatuto.into_inner();
                for field in decision_fields {
                match field.as_rule() {
                    Rule::expresion => { 
                        self.process_expresion(field)?;
                        // Add GotoF
                        let last_quad = self.quadruples.last().unwrap();
                        if last_quad.out_op.Type != VarType::Bool {
                            return Err(String::from("Variable debe ser boolean para generar for"));
                        }
                        self.jump_vec.push(self.quadruples.len());
                        self.gen_quad(Actions::GotoF, Some(last_quad.out_op.clone()), None, Var {Location: String::from(""), Type: VarType::Int});
                    }
                    Rule::estatuto => {
                        self.process_statute(field)?;
                    }
                    Rule::elseIf => {
                        self.fill_goto(self.quadruples.len() + 1);
                        self.jump_vec.push(self.quadruples.len());
                        self.gen_quad(Actions::Goto, None, None, Var {Location: String::from(""), Type: VarType::Int});
                        // Add Goto to jump false if section
                        for field in field.into_inner() {
                            self.process_statute(field)?;
                        }
                    }
                    _ => {}
                }
                }
                self.fill_goto(self.quadruples.len());
            },
            Rule::condicion => { // While
                let condicion_fields = estatuto.into_inner();
                for field in condicion_fields {
                    match  field.as_rule() {
                        Rule::expresion => {
                            self.jump_vec.push(self.quadruples.len());
                            self.process_expresion(field)?;

                            let lh_op = self.op_vec.pop();
                            let last_quad = self.quadruples.last().unwrap();
                            if last_quad.out_op.Type != VarType::Bool {
                                return Err(String::from("Variable debe ser boolean para generar While"));
                            }
                            self.jump_vec.push(self.quadruples.len());
                            self.gen_quad(Actions::GotoF, lh_op, None, Var {Location: String::from(""), Type: VarType::Int});

                        },
                        Rule::estatuto => {
                            self.process_statute(field)?;
                        },
                        _=> {}
                    }
                }

                self.fill_goto(self.quadruples.len() + 1);

                if let Some(jump_pos) = self.jump_vec.pop() {
                    self.gen_quad(Actions::Goto, None, None, Var { Location: jump_pos.to_string(), Type: VarType::Int });
                }
            },
            Rule::no_condicion => { // For
                // let mut no_condicion_fields = estatuto.into_inner();

                // let control_var = no_condicion_fields.next().unwrap();
                // let control_exp = no_condicion_fields.next().unwrap();

                // let mut current_func = self.function_table.get_mut(self.current_func.as_str()).unwrap();
                // let new_loc = BaseDirs::TempInt as i32 + current_func.temp_vars.0;
                // current_func.temp_vars.0 += 1;

                // let VC = Var {Location: new_loc.to_string(), Type: VarType::Int};

                // new_loc = BaseDirs::TempInt as i32 + current_func.temp_vars.0;
                // current_func.temp_vars.0 += 1;

                // let VF = Var {Location: new_loc.to_string(), Type: VarType::Int};

                // {
                //     self.process_expresion(control_exp)?;
                // }

                // let lh_op = self.op_vec.pop();
                // let control_var = self.find_var(control_var.as_str()).unwrap();
                // self.gen_quad(Actions::Assign, lh_op, None, control_var);

                // // Generate quad for control variable
                // let last_op = self.quadruples.last().unwrap().out_op.clone();
                // if !(last_op.Type == VarType::Int) {
                //     return Err(String::from("Variable control debe ser numerica para generar For"))
                // }
                // self.gen_quad(Actions::Assign, Some(last_op), None, VC);

                // let final_exp = no_condicion_fields.next().unwrap();

                // self.process_expresion(final_exp)?;

                // // Generate quad for final variable
                // let last_op = self.op_vec.pop().unwrap();
                // if last_op.Type != VarType::Int && last_op.Type != VarType::Float {
                //     return Err(String::from("Variable final debe ser numerica para generar For"))
                // }
                // self.gen_quad(Actions::Assign, Some(last_op), None, VF);

                // new_loc = BaseDirs::TempInt as i32 + current_func.temp_vars.0;
                // current_func.temp_vars.0 += 1;
                // let compare_var = Var { Location: new_loc.to_string(), Type: VarType::Int };

                // self.gen_quad(Actions::LessThan, Some(VC), Some(VF), compare_var);
                // let goto_start = self.quadruples.len() - 1;

                // let lh_op = self.op_vec.pop();
                // self.gen_quad(Actions::GotoF, lh_op, None, Var { Location: String::from(""), Type: VarType::Int });
                // self.jump_vec.push(self.quadruples.len() - 1);

                // // Process statutes inside for
                // for field in no_condicion_fields {
                //     self.process_statute(field)?;
                // }

                // self.gen_quad(Actions::Sum, Some(VF.clone()), Some(Var { Location: String::from("1"), Type: VarType::Int }), VC);

                // // Generate quad for control variable
                // self.gen_quad(Actions::Assign, Some(VC.clone()), None, control_var);
                
                // // // Generate quad for final GOTO
                // self.gen_quad(Actions::Goto, None,None, Var { Location: goto_start.to_string(), Type: VarType::Int });

                // self.fill_goto(self.quadruples.len());
            },
            Rule::escritura => {
                let write_fields = estatuto.into_inner();
                for field in write_fields {
                    match field.as_rule() {
                        Rule::string => {
                            self.gen_quad(Actions::Print, None, None, Var { Location: field.as_str().to_string(), Type: VarType::Void });
                        }
                        Rule::expresion => {
                            self.process_expresion(field)?;
                            if let Some(out_op) = self.op_vec.pop() {
                                self.gen_quad(Actions::Print, None, None, out_op);
                            }
                        }
                        _ => {}
                    }
                }
            },
            Rule::llamada => {
                let mut llamada_fields = estatuto.into_inner();

                let func_name = llamada_fields.next().unwrap().as_str().to_string();

                let mut params: Vec<VarType> = Default::default();
                let mut is_special: bool = false;
                if let Some(param_list) = self.function_table.get(func_name.as_str()) {
                    params = param_list.param_list.clone();
                    is_special = false;
                } else if let Some(param_list) = self.special_functions.get(func_name.as_str()) {
                    params = param_list.clone();
                    is_special = true;
                }

                if !is_special {
                    // en vm crear una segunda memoria
                    self.gen_quad(Actions::Era, None, None, Var { Location: func_name.clone(), Type: VarType::Void });
                }

                let mut param_count = 0;
                if let Some(args) = llamada_fields.next() {
                    for (i, arg) in args.into_inner().enumerate() {
                        self.process_expresion(arg)?;
                        // POP from op_vec
                        let param = self.op_vec.pop().unwrap().clone();

                        if i >= params.len() {
                            return Err(format!("Wrong number of arguments. Expected: {}. Got: {}", params.len(), param_count + 1));
                        }

                        if params[i] != param.Type {
                            return Err(format!("Parameter {} in call of {} is of incompatible types. Expected: {:?}. Got: {:?}", i, func_name, params[i].clone(), param.Type.clone()));
                        }

                        self.gen_quad(Actions::Param, None, None, param);
                        param_count += 1;
                    }
                    if param_count < params.len() {
                        return Err(format!("Wrong number of arguments. Expected: {}. Got: {}", params.len(), param_count));
                    }
                } else if param_count != params.len() {
                    return Err(format!("Wrong number of arguments. Expected: {}. Got: {}", params.len(), param_count));
                }

                if !is_special {
                    self.gen_quad(Actions::Gosub, None, None, Var { Location: func_name.clone(), Type: VarType::Void });
                } else {
                    match func_name.as_str() {
                        "Center" => self.gen_quad(Actions::Center, None, None, Var { Location: func_name.clone(), Type: VarType::Void }),
                        "Forward" => self.gen_quad(Actions::Forward, None, None, Var { Location: func_name.clone(), Type: VarType::Void }),
                        "Backward" => self.gen_quad(Actions::Backward, None, None, Var { Location: func_name.clone(), Type: VarType::Void }),
                        "Left" => self.gen_quad(Actions::Left, None, None, Var { Location: func_name.clone(), Type: VarType::Void }),
                        "Right" => self.gen_quad(Actions::Right, None, None, Var { Location: func_name.clone(), Type: VarType::Void }),
                        "Point" => self.gen_quad(Actions::Point, None, None, Var { Location: func_name.clone(), Type: VarType::Void }),
                        "Circle" => self.gen_quad(Actions::Circle, None, None, Var { Location: func_name.clone(), Type: VarType::Void }),
                        "Arc" => self.gen_quad(Actions::Arc, None, None, Var { Location: func_name.clone(), Type: VarType::Void }),
                        "PenUp" => self.gen_quad(Actions::PenUp, None, None, Var { Location: func_name.clone(), Type: VarType::Void }),
                        "PenDown" => self.gen_quad(Actions::PenDown, None, None, Var { Location: func_name.clone(), Type: VarType::Void }),
                        "Color" => self.gen_quad(Actions::Color, None, None, Var { Location: func_name.clone(), Type: VarType::Void }),
                        "Size" => self.gen_quad(Actions::Size, None, None, Var { Location: func_name.clone(), Type: VarType::Void }),
                        "Clear" => self.gen_quad(Actions::Clear, None, None, Var { Location: func_name.clone(), Type: VarType::Void }),
                        _ => error!("Unknown special function {}", func_name.clone())
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn process_function(&mut self, data: pest::iterators::Pair<Rule>) -> Result<(), String> {
        let mut fields = data.into_inner();
        let return_type: &str = fields.next().unwrap().as_str();
        let func_return_type = match return_type {
            "int" => VarType::Int,
            "float" => VarType::Float,
            "char" => VarType::Char,
            "void" | &_ => VarType::Void,
        };
        let func_name = fields.next().unwrap().as_str();
        self.current_func = func_name.to_string();

        match func_return_type {
            VarType::Int => {
                let new_loc = BaseDirs::GlobalInt as i32 + self.global_locs.0;
                self.global_locs.0 += 1;
                self.global_vars.insert(func_name.to_string(), Var { Type: VarType::Int, Location: new_loc.to_string() });
            }
            VarType::Float => {
                let new_loc = BaseDirs::GlobalFloat as i32 + self.global_locs.1;
                self.global_locs.1 += 1;
                self.global_vars.insert(func_name.to_string(), Var { Type: VarType::Float, Location: new_loc.to_string() });
            }
            VarType::Char => {
                let new_loc = BaseDirs::GlobalChar as i32 + self.global_locs.2;
                self.global_locs.2 += 1;
                self.global_vars.insert(func_name.to_string(), Var { Type: VarType::Char, Location: new_loc.to_string() });
            }
            _ => {}
        }

        // debug!("Processing function {}", func_name);

        let new_func = Func {
            name: func_name.to_string(),
            ret_type: func_return_type.clone(),
            var_table: HashMap::new(),
            param_list: Vec::new(),
            start_loc: self.quadruples.len(),
            local_vars: (0,0,0),
            temp_vars: (0,0,0,0)
        };

        match self.function_table.entry(func_name.to_string())  {
            Vacant(entry) => entry.insert(new_func),
            Occupied(_) => return Err(format!("Function {} has already been declared.", func_name))
        };

        debug!("Funcs: {:?}", self.function_table);

        for field in fields {
            match field.as_rule() {
                Rule::args => { 
                    let mut current_func = self.function_table.get_mut(self.current_func.as_str()).unwrap();

                    // Process args and add to args vec
                    let args = field.into_inner();
                    for arg in args {
                        let mut arg = arg.into_inner();
                        let arg_type = arg.next().unwrap();
                        let arg_id = arg.next().unwrap().as_str();
                        let arg_data = match arg_type.as_str() {
                            "int" => { 
                                let new_loc = BaseDirs::LocalInt as i32 + current_func.local_vars.0; 
                                current_func.local_vars.0 += 1; 
                                (VarType::Int, new_loc) 
                            }
                            "float" => { 
                                let new_loc = BaseDirs::LocalInt as i32 + current_func.local_vars.1; 
                                current_func.local_vars.1 += 1; 
                                (VarType::Float, new_loc) 
                            }
                            "char" => { 
                                let new_loc = BaseDirs::LocalInt as i32 + current_func.local_vars.2; 
                                current_func.local_vars.2 += 1; 
                                (VarType::Char, new_loc) 
                            }
                            &_ => unreachable!()
                        };
                        match self.local_vars.entry(arg_id.to_string()) {
                            Vacant(entry) => {
                                entry.insert(Var { Type: arg_data.0, Location: arg_data.1.to_string() });
                                current_func.param_list.push(arg_data.0);
                            },
                            Occupied(_) => return Err(format!("Param {} has already been declared.", arg_id))
                        }
                    }
                }
                Rule::vars => {
                    let vars = self.process_vars(field, false)?;
                    for (var_name, var_data) in vars {
                        let name = var_name.as_str();
                        match self.local_vars.entry(name.to_string()) {
                            Vacant(entry) => { entry.insert(var_data); } 
                            Occupied(_) => return Err(format!("Variable {} has already been declared as parameter.", name))
                        }
                    }
                    // Insert local variables into function declaration
                }
                Rule::estatuto => {
                    let mut current_func = self.function_table.get_mut(self.current_func.as_str()).unwrap();
                    current_func.var_table = self.local_vars.clone();

                    // Process statutes
                    self.process_statute(field)?;
                }
                _ => {}
            }
        }
        self.gen_quad(Actions::EndFunc, None, None, Var { Location: String::from(""), Type: VarType::Void });

        // Reset local variables
        self.local_vars.clear();
        
        // reset all counters
        Ok(())
    }

    fn process_rules(&mut self, data: pest::iterators::Pair<Rule>) -> Result<(), String> {
        match data.as_rule() {
        Rule::programa => {
            // println!("{:?} {:?}", data.as_rule(), data.as_str());
            let mut fields = data.into_inner();

            // Get program ID
            let program_id = fields.next().unwrap();
            debug!("Program ID: {:?}", program_id.as_str());

            // self.gen_quad(Actions::Era, None, None, Var { Location: "main".to_string(), Type: VarType::Void });
            self.gen_quad(Actions::Goto, None, None, Var{ Location: "".to_string(), Type: VarType::Void});

            for field in fields {
                match field.as_rule() {
                    Rule::vars => {
                        let var_map = self.process_vars(field, true)?;
                        self.global_vars = var_map;
                    }
                    Rule::funciones => {
                        self.process_function(field)?;
                    }
                    _ => {}
                } 
            }
        }
            _ => { 
                warn!("unknown {:?} {:?}", data.as_rule(), data.as_str());
            }
        }

        if !self.function_table.contains_key("main") {
            return Err("No 'main' function declared.".to_string());
        } else {
            let main_fun: &Func = self.function_table.get("main").unwrap();
            self.quadruples.get_mut(0).unwrap().out_op.Location = main_fun.start_loc.to_string();
        }

        // TODO: verify main function exists, throw error if not

        debug!("Global vars: {:?}", self.global_vars);
        debug!("Constants: {:?}", self.constants);
        debug!("{:?}", self.function_table);
        
        for (pos, quad) in self.quadruples.iter().enumerate() {
            debug!("{} {}", pos, quad);
        }
        Ok(())
    }

    pub fn process_file(&mut self, file_name: &str) {
        let file = fs::read_to_string(file_name).expect("Cannot read file");

        let data = MMIParser::parse(Rule::file, &file).expect("unsuccessful parse").next().unwrap();

        self.process_rules(data.into_inner().next().unwrap().clone()).unwrap();
        println!("{:?}", self.function_table);
    }

    pub fn write_obj_file(&self, file_name: &str) -> std::io::Result<()> {
        let mut file = File::create("file.obj")?;

        // Write a &str in the file (ignoring the result).
        for constant in self.constants.iter() {
            writeln!(file, "C {} {} {:?}", constant.Value, constant.Location, constant.Type)?;
        }

        writeln!(file, "G {} {} {}", self.global_locs.0, self.global_locs.1, self.global_locs.2)?;
            

        for (_, func) in self.function_table.iter() {
            // name, start_loc, local_i, local_f, local_c, temp_i, temp_f, temp_c, temp_b
            write!(file, "F {} {} ", func.name, func.start_loc)?;
            write!(file, "{} {} {} ", func.local_vars.0, func.local_vars.1, func.local_vars.2)?;
            write!(file, "{} {} {} {}\n", func.temp_vars.0, func.temp_vars.1, func.temp_vars.2, func.temp_vars.3)?;
        };

        for quad in self.quadruples.iter() {
            let lh = match &quad.lh_op {
                Some(lh) => lh.Location.clone(),
                None => "-1".to_string()
            };
            let rh = match &quad.rh_op {
                Some(rh) => rh.Location.clone(),
                None => "-1".to_string()
            };
        
            writeln!(file, "A {:?} {} {} {}", quad.op, lh, rh, quad.out_op.Location)?
            // match quad.op {
            //     Actions::Sum |
            //     Actions::Sub |
            //     Actions::Mult |
            //     Actions::Div |
            //     Actions::NotEqual |
            //     Actions::Equal |
            //     Actions::MoreThan |
            //     Actions::LessThan |
            //     Actions::And |
            //     Actions::Or => {
            //         writeln!(file, "{:?} {} {} {}", quad.op, quad.lh_op.unwrap().Location, quad.rh_op.unwrap().Location, quad.out_op.Location)
            //     },
            //     // Actions with 3 elements
            //     Actions::Assign |
            //     Actions::Param => {
            //         writeln!(file, "{:?} {} {}", quad.op, quad.lh_op.unwrap().Location, quad.out_op.Location)
            //     }
            //     // Actions with 2 elements
                
            // }
        };

        Ok(())
    }
}