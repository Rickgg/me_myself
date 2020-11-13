use std::collections::HashMap;

pub enum BaseDirs {
    // Base directions
    GlobalInt = 5000,
    GlobalFloat = 6000,
    GlobalChar = 7000,
    GlobalUpperLim = 8000,

    // Local directions
    LocalInt = 10000,
    LocalFloat = 11000,
    LocalChar = 12000,
    LocalUpperLim = 13000,

    // Temporal directions
    TempInt = 20000,
    TempFloat = 21000,
    TempChar = 22000,
    TempBool = 23000,
    TempUpperLim = 24000,

    // Constant directions
    CteInt = 30000,
    CteFloat = 31000,
    CteChar = 32000,
    CteString = 33000,
    CteUpperLim = 34000
}

#[derive(Clone, Debug)]
pub enum VarValue {
    Int(i32),
    Float(f64),
    Char(String),
    Bool(bool)
}

type VarMap = HashMap<i32, VarValue>;

#[derive(Default, Debug, Clone)]
pub struct Memory {
    variables: VarMap
}

// TODO: verify upper limits (memory overflow)
impl Memory {
    pub fn set_globals(&mut self, g_i: i32, g_f: i32, g_c: i32) {
        for i in 0..g_i {
            self.variables.insert(i + BaseDirs::GlobalInt as i32, VarValue::Int(0));
        }
        for i in 0..g_f {
            self.variables.insert(i + BaseDirs::GlobalFloat as i32, VarValue::Float(0.0));
        }
        for i in 0..g_c {
            self.variables.insert(i + BaseDirs::GlobalChar as i32, VarValue::Char(String::from("")));
        }
    }

    pub fn set_new_func(&mut self, locals: (i32, i32, i32), temp: (i32, i32, i32, i32)) {
        // insert local ints
        for i in 0..locals.0 {
            self.variables.insert(i + BaseDirs::LocalInt as i32, VarValue::Int(0));
        }
        for i in 0..locals.1 {
            self.variables.insert(i + BaseDirs::LocalFloat as i32, VarValue::Float(0.0));
        }
        for i in 0..locals.2 {
            self.variables.insert(i + BaseDirs::LocalChar as i32, VarValue::Char(String::from("")));
        }

        for i in 0..temp.0 {
            self.variables.insert(i + BaseDirs::TempInt as i32, VarValue::Int(0));
        }
        for i in 0..temp.1 {
            self.variables.insert(i + BaseDirs::TempFloat as i32, VarValue::Float(0.0));
        }
        for i in 0..temp.2 {
            self.variables.insert(i + BaseDirs::TempChar as i32, VarValue::Char(String::from("")));
        }
        for i in 0..temp.3 {
            self.variables.insert(i + BaseDirs::TempBool as i32, VarValue::Bool(false));
        }
    }

    pub fn get_val(&self, location: i32) -> Result<VarValue, String> {
        // Global variable
        let val = if location >= BaseDirs::GlobalInt as i32 && location < BaseDirs::TempUpperLim as i32 {
            match self.variables.get(&location) {
                Some(val) => val,
                None => return Err(format!("Memory location {} not initialized", location))
            }
        } else {
            return Err(format!("Memory location {} not initialized", location))
        };
        Ok(val.clone())
    }

    pub fn set_val(&mut self, location: i32, new_val: VarValue) -> Result<(), String> {
        if location >= BaseDirs::GlobalInt as i32 && location < BaseDirs::TempUpperLim as i32 {
            if let Some(val) = self.variables.get_mut(&location) {
                *val = new_val;
            } else {
                return Err(format!("Memory location {} not initialized", location))
            }
        } else {
            return Err(format!("Memory location {} not initialized", location))
        };
        Ok(())
    }

    pub fn clear(&mut self) {
        self.variables.clear();
    }
}