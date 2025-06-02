use std::collections::{HashMap};
use std::fs;
use std::io::{BufRead, BufReader};

// Define OpCodes (consistent with quadruples.rs)
struct OpCode;
impl OpCode {
    pub const ASSIGN: i32 = 1;
    pub const ADD: i32 = 4;
    pub const SUB: i32 = 5;
    pub const MULT: i32 = 6;
    pub const DIV: i32 = 7;
    pub const GT: i32 = 8;
    pub const LT: i32 = 9;
    pub const EQ: i32 = 10;
    pub const NEQ: i32 = 11;
    pub const PRINT: i32 = 20;
    pub const GOTO: i32 = 30;
    pub const GOTOF: i32 = 31;
    pub const ERA: i32 = 40;
    pub const PARAM: i32 = 41;
    pub const GOSUB: i32 = 42;
    pub const ENDFUNC: i32 = 43;
    pub const HALT: i32 = 50;
}

// Define Memory Address Constants (consistent with quadruples.rs)
const INT_START: i32 = 1000;
const FLOAT_START: i32 = 2000;
const BOOL_START: i32 = 3000;
const CTE_INT_START: i32 = 4000;
const CTE_FLOAT_START: i32 = 4500;
const CTE_BOOL_START: i32 = 4800;
const TEMP_INT_START: i32 = 5000;
const TEMP_FLOAT_START: i32 = 6000;
const TEMP_BOOL_START: i32 = 7000;

#[derive(Debug, Clone, Copy)]
struct Quad {
    op: i32,
    arg1: i32,
    arg2: i32,
    result: i32,
}

#[derive(Debug, Clone)]
enum VMValue {
    Int(i32),
    Float(f64),
    Bool(bool),  // Add dedicated boolean type
}

#[derive(Debug, Clone)]
struct VMFunctionInfo {
    name: String,
    param_count: usize,
    param_addresses: Vec<i32>, // Loaded from the .obj file
}

pub struct VM {
    quads: Vec<Quad>,
    ip: usize,
    int_memory: Vec<Option<i32>>,
    float_memory: Vec<Option<f64>>,
    bool_memory: Vec<Option<bool>>,
    call_stack: Vec<usize>, // Stores return IPs
    functions: HashMap<i32, VMFunctionInfo>, // Map start_quad_idx to info

    // Track the highest address used in each segment for dynamic sizing
    max_int_addr: i32,
    max_float_addr: i32,
    max_bool_addr: i32,
    max_cte_int_addr: i32,
    max_cte_float_addr: i32,
    max_cte_bool_addr: i32,
    max_temp_int_addr: i32,
    max_temp_float_addr: i32,
    max_temp_bool_addr: i32,

    // For function calls
    staged_params: Vec<VMValue>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            quads: Vec::new(),
            ip: 0,
            int_memory: Vec::new(),
            float_memory: Vec::new(),
            bool_memory: Vec::new(),  // Initialize bool memory
            call_stack: Vec::new(),
            functions: HashMap::new(),
            staged_params: Vec::new(),

            // Initialize max addresses to their respective starts (no addresses used yet)
            max_int_addr: INT_START - 1,
            max_float_addr: FLOAT_START - 1,
            max_bool_addr: BOOL_START - 1,
            max_cte_int_addr: CTE_INT_START - 1,
            max_cte_float_addr: CTE_FLOAT_START - 1,
            max_cte_bool_addr: CTE_BOOL_START - 1,
            max_temp_int_addr: TEMP_INT_START - 1,
            max_temp_float_addr: TEMP_FLOAT_START - 1,
            max_temp_bool_addr: TEMP_BOOL_START - 1,
        }
    }

    fn update_max_address(&mut self, address: i32) {
        match address {
            addr if addr >= INT_START && addr < FLOAT_START => {
                self.max_int_addr = self.max_int_addr.max(addr);
            }
            addr if addr >= FLOAT_START && addr < BOOL_START => {
                self.max_float_addr = self.max_float_addr.max(addr);
            }
            addr if addr >= BOOL_START && addr < CTE_INT_START => {
                self.max_bool_addr = self.max_bool_addr.max(addr);
            }
            addr if addr >= CTE_INT_START && addr < CTE_FLOAT_START => {
                self.max_cte_int_addr = self.max_cte_int_addr.max(addr);
            }
            addr if addr >= CTE_FLOAT_START && addr < CTE_BOOL_START => {
                self.max_cte_float_addr = self.max_cte_float_addr.max(addr);
            }
            addr if addr >= CTE_BOOL_START && addr < TEMP_INT_START => {
                self.max_cte_bool_addr = self.max_cte_bool_addr.max(addr);
            }
            addr if addr >= TEMP_INT_START && addr < TEMP_FLOAT_START => {
                self.max_temp_int_addr = self.max_temp_int_addr.max(addr);
            }
            addr if addr >= TEMP_FLOAT_START && addr < TEMP_BOOL_START => {
                self.max_temp_float_addr = self.max_temp_float_addr.max(addr);
            }
            addr if addr >= TEMP_BOOL_START => {
                self.max_temp_bool_addr = self.max_temp_bool_addr.max(addr);
            }
            _ => {} // Unknown address range
        }
    }

    fn resize_memory(&mut self) {
        // Calculate required sizes for each segment
        let int_local_size = if self.max_int_addr >= INT_START {
            (self.max_int_addr - INT_START + 1) as usize
        } else { 0 };

        let float_local_size = if self.max_float_addr >= FLOAT_START {
            (self.max_float_addr - FLOAT_START + 1) as usize
        } else { 0 };
        
        let bool_local_size = if self.max_bool_addr >= BOOL_START {
            (self.max_bool_addr - BOOL_START + 1) as usize
        } else { 0 };

        let cte_int_size = if self.max_cte_int_addr >= CTE_INT_START {
            (self.max_cte_int_addr - CTE_INT_START + 1) as usize
        } else { 0 };

        let cte_float_size = if self.max_cte_float_addr >= CTE_FLOAT_START {
            (self.max_cte_float_addr - CTE_FLOAT_START + 1) as usize
        } else { 0 };
        
        let cte_bool_size = if self.max_cte_bool_addr >= CTE_BOOL_START {
            (self.max_cte_bool_addr - CTE_BOOL_START + 1) as usize
        } else { 0 };

        let temp_int_size = if self.max_temp_int_addr >= TEMP_INT_START {
            (self.max_temp_int_addr - TEMP_INT_START + 1) as usize
        } else { 0 };
        
        let temp_float_size = if self.max_temp_float_addr >= TEMP_FLOAT_START {
            (self.max_temp_float_addr - TEMP_FLOAT_START + 1) as usize
        } else { 0 };

        let temp_bool_size = if self.max_temp_bool_addr >= TEMP_BOOL_START {
            (self.max_temp_bool_addr - TEMP_BOOL_START + 1) as usize
        } else { 0 };

        // Resize int_memory
        let total_int_size = int_local_size + cte_int_size + temp_int_size;
        if total_int_size > 0 {
            self.int_memory.resize(total_int_size, None);
        }

        // Resize float_memory
        let total_float_size = float_local_size + cte_float_size + temp_float_size;
        if total_float_size > 0 {
            self.float_memory.resize(total_float_size, None);
        }
        
        // Resize bool_memory
        let total_bool_size = bool_local_size + cte_bool_size + temp_bool_size;
        if total_bool_size > 0 {
            self.bool_memory.resize(total_bool_size, None);
        }
    }

    fn get_int_idx(&self, address: i32) -> Result<usize, String> {
        let int_local_size = if self.max_int_addr >= INT_START {
            (self.max_int_addr - INT_START + 1) as usize
        } else { 0 };

        let cte_int_size = if self.max_cte_int_addr >= CTE_INT_START {
            (self.max_cte_int_addr - CTE_INT_START + 1) as usize
        } else { 0 };

        match address {
            addr if addr >= INT_START && addr <= self.max_int_addr => {
                Ok((addr - INT_START) as usize)
            }
            addr if addr >= CTE_INT_START && addr <= self.max_cte_int_addr => {
                Ok((addr - CTE_INT_START) as usize + int_local_size)
            }
            addr if addr >= TEMP_INT_START && addr <= self.max_temp_int_addr => {
                Ok((addr - TEMP_INT_START) as usize + int_local_size + cte_int_size)
            }
            _ => Err(format!("Invalid or unmapped integer address: {}", address)),
        }
    }

    fn get_float_idx(&self, address: i32) -> Result<usize, String> {
        let float_local_size = if self.max_float_addr >= FLOAT_START {
            (self.max_float_addr - FLOAT_START + 1) as usize
        } else { 0 };

        let cte_float_size = if self.max_cte_float_addr >= CTE_FLOAT_START {
            (self.max_cte_float_addr - CTE_FLOAT_START + 1) as usize
        } else { 0 };

        match address {
            addr if addr >= FLOAT_START && addr <= self.max_float_addr => {
                Ok((addr - FLOAT_START) as usize)
            }
            addr if addr >= CTE_FLOAT_START && addr <= self.max_cte_float_addr => {
                Ok((addr - CTE_FLOAT_START) as usize + float_local_size)
            }
            addr if addr >= TEMP_FLOAT_START && addr <= self.max_temp_float_addr => {
                Ok((addr - TEMP_FLOAT_START) as usize + float_local_size + cte_float_size)
            }
            _ => Err(format!("Invalid or unmapped float address: {}", address)),
        }
    }
    
    fn get_bool_idx(&self, address: i32) -> Result<usize, String> {
        let bool_local_size = if self.max_bool_addr >= BOOL_START {
            (self.max_bool_addr - BOOL_START + 1) as usize
        } else { 0 };

        let cte_bool_size = if self.max_cte_bool_addr >= CTE_BOOL_START {
            (self.max_cte_bool_addr - CTE_BOOL_START + 1) as usize
        } else { 0 };

        match address {
            addr if addr >= BOOL_START && addr <= self.max_bool_addr => {
                Ok((addr - BOOL_START) as usize)
            }
            addr if addr >= CTE_BOOL_START && addr <= self.max_cte_bool_addr => {
                Ok((addr - CTE_BOOL_START) as usize + bool_local_size)
            }
            addr if addr >= TEMP_BOOL_START && addr <= self.max_temp_bool_addr => {
                Ok((addr - TEMP_BOOL_START) as usize + bool_local_size + cte_bool_size)
            }
            _ => Err(format!("Invalid or unmapped bool address: {}", address)),
        }
    }

    fn get_value(&self, address: i32) -> Result<VMValue, String> {
        if address == -1 { return Err("Attempted to read from -1 address".to_string());}
        
        // First try int memory
        if let Ok(idx) = self.get_int_idx(address) {
            if idx < self.int_memory.len() {
                if let Some(val) = self.int_memory[idx] {
                    return Ok(VMValue::Int(val));
                } else {
                    return Err(format!("Read from uninitialized integer memory at address {}, mapped to idx {}", address, idx));
                }
            } else {
                return Err(format!("Index {} out of bounds for int_memory (size {})", idx, self.int_memory.len()));
            }
        }
        
        // Then try float memory
        if let Ok(idx) = self.get_float_idx(address) {
            if idx < self.float_memory.len() {
                if let Some(val) = self.float_memory[idx] {
                    return Ok(VMValue::Float(val));
                } else {
                    return Err(format!("Read from uninitialized float memory at address {}, mapped to idx {}", address, idx));
                }
            } else {
                return Err(format!("Index {} out of bounds for float_memory (size {})", idx, self.float_memory.len()));
            }
        }
        
        // Finally try bool memory
        if let Ok(idx) = self.get_bool_idx(address) {
            if idx < self.bool_memory.len() {
                if let Some(val) = self.bool_memory[idx] {
                    return Ok(VMValue::Bool(val));
                } else {
                    return Err(format!("Read from uninitialized bool memory at address {}, mapped to idx {}", address, idx));
                }
            } else {
                return Err(format!("Index {} out of bounds for bool_memory (size {})", idx, self.bool_memory.len()));
            }
        }
        
        Err(format!("Address {} does not map to any known memory segment for get_value", address))
    }

    fn set_value(&mut self, address: i32, value: VMValue) -> Result<(), String> {
        if address == -1 { return Err("Attempted to write to -1 address".to_string());}
        
        // First try int memory
        if let Ok(idx) = self.get_int_idx(address) {
            if idx >= self.int_memory.len() {
                return Err(format!("Index {} out of bounds for int_memory (size {})", idx, self.int_memory.len()));
            }
            match value {
                VMValue::Int(i) => self.int_memory[idx] = Some(i),
                VMValue::Float(_) => return Err(format!("Type mismatch: cannot assign Float to Int address {}", address)),
                VMValue::Bool(b) => self.int_memory[idx] = Some(if b { 1 } else { 0 }), // Convert bool to int
            }
            return Ok(());
        }
        
        // Then try float memory
        if let Ok(idx) = self.get_float_idx(address) {
            if idx >= self.float_memory.len() {
                return Err(format!("Index {} out of bounds for float_memory (size {})", idx, self.float_memory.len()));
            }
            match value {
                VMValue::Float(f) => self.float_memory[idx] = Some(f),
                VMValue::Int(i) => self.float_memory[idx] = Some(i as f64), // Allow int to float assignment (promotion)
                VMValue::Bool(_) => return Err(format!("Type mismatch: cannot assign Bool to Float address {}", address)),
            }
            return Ok(());
        }
        
        // Finally try bool memory
        if let Ok(idx) = self.get_bool_idx(address) {
            if idx >= self.bool_memory.len() {
                return Err(format!("Index {} out of bounds for bool_memory (size {})", idx, self.bool_memory.len()));
            }
            match value {
                VMValue::Bool(b) => self.bool_memory[idx] = Some(b),
                VMValue::Int(i) => self.bool_memory[idx] = Some(i != 0), // Convert int to bool (0 = false, non-zero = true)
                VMValue::Float(_) => return Err(format!("Type mismatch: cannot assign Float to Bool address {}", address)),
            }
            return Ok(());
        }
        
        Err(format!("Address {} does not map to any known memory segment for set_value", address))
    }

    pub fn load_obj_file(&mut self, filepath: &str) -> Result<(), String> {
        let file = fs::File::open(filepath).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let mut current_section = "".to_string();

        // First pass: collect all addresses to determine memory requirements
        let mut addresses_to_track = Vec::new();

        for line_result in reader.lines() {
            let line = line_result.map_err(|e| e.to_string())?.trim().to_string();
            if line.starts_with("//") || line.is_empty() {
                continue;
            }

            if line.ends_with(':') {
                current_section = line.trim_end_matches(':').to_string();
                continue;
            }
            if line.starts_with("END_") {
                current_section = "".to_string();
                continue;
            }

            match current_section.as_str() {
                "CONSTANTS_INT" => {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() == 2 {
                        let addr = parts[1].parse::<i32>().map_err(|e| format!("{}", e))?;
                        addresses_to_track.push(addr);
                    }
                }
                "CONSTANTS_FLOAT" => {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() == 2 {
                        let addr = parts[1].parse::<i32>().map_err(|e| format!("{}", e))?;
                        addresses_to_track.push(addr);
                    }
                }
                "FUNCTIONS" => {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 4 {
                        let param_count = parts[2].parse::<usize>().map_err(|e| format!("{}", e))?;
                        // Collect parameter addresses
                        for i in 0..param_count {
                            if 4 + i < parts.len() {
                                let param_addr = parts[4 + i].parse::<i32>().map_err(|e| format!("{}", e))?;
                                addresses_to_track.push(param_addr);
                            }
                        }
                    }
                }
                "QUADRUPLES" => {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() == 4 {
                        // Collect addresses from quadruples (arg1, arg2, result)
                        for i in 1..4 {
                            if let Ok(addr) = parts[i].parse::<i32>() {
                                if addr != -1 { // Skip invalid addresses
                                    addresses_to_track.push(addr);
                                }
                            }
                        }
                    }
                }
                _ => {} // Unknown section
            }
        }

        // Update max addresses based on collected addresses
        for addr in addresses_to_track {
            self.update_max_address(addr);
        }

        // Resize memory based on discovered addresses
        self.resize_memory();

        // Second pass: actually load the data
        let file = fs::File::open(filepath).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let mut current_section = "".to_string();

        for line_result in reader.lines() {
            let line = line_result.map_err(|e| e.to_string())?.trim().to_string();
            if line.starts_with("//") || line.is_empty() {
                continue;
            }

            if line.ends_with(':') {
                current_section = line.trim_end_matches(':').to_string();
                continue;
            }
            if line.starts_with("END_") {
                current_section = "".to_string();
                continue;
            }

            match current_section.as_str() {
                "CONSTANTS_INT" => {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() == 2 {
                        let val = parts[0].parse::<i32>().map_err(|e| format!("{}", e))?;
                        let addr = parts[1].parse::<i32>().map_err(|e| format!("{}", e))?;
                        let idx = self.get_int_idx(addr)?;
                        self.int_memory[idx] = Some(val);
                    }
                }
                "CONSTANTS_FLOAT" => {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() == 2 {
                        let val = parts[0].parse::<f64>().map_err(|e| format!("{}", e))?;
                        let addr = parts[1].parse::<i32>().map_err(|e| format!("{}", e))?;
                        let idx = self.get_float_idx(addr)?;
                        self.float_memory[idx] = Some(val);
                    }
                }
                "FUNCTIONS" => {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 4 {
                        let name = parts[0].to_string();
                        let start_idx = parts[1].parse::<i32>().map_err(|e| format!("{}", e))?;
                        let param_count = parts[2].parse::<usize>().map_err(|e| format!("{}", e))?;

                        let mut param_addresses = Vec::new();
                        if parts.len() != 4 + param_count {
                            return Err(format!(
                                "Function '{}': Mismatch between param_count ({}) and number of address parts provided ({}). Expected {} parts in total for function definition line.",
                                name, param_count, parts.len() - 4, 4 + param_count
                            ));
                        }

                        for i in 0..param_count {
                            let param_addr_str = parts[4 + i];
                            param_addresses.push(param_addr_str.parse::<i32>().map_err(|e| format!("Error parsing param address '{}' for function {}: {}", param_addr_str, name, e))?);
                        }

                        self.functions.insert(start_idx, VMFunctionInfo {
                            name,
                            param_count,
                            param_addresses,
                        });
                    } else {
                        return Err(format!("Invalid line in FUNCTIONS section: '{}'. Expected at least 4 comma-separated values.", line));
                    }
                }
                "QUADRUPLES" => {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() == 4 {
                        let op = parts[0].parse().map_err(|e| format!("{}", e))?;
                        let arg1 = parts[1].parse().map_err(|e| format!("{}", e))?;
                        let arg2 = parts[2].parse().map_err(|e| format!("{}", e))?;
                        let result = parts[3].parse().map_err(|e| format!("{}", e))?;
                        
                        self.quads.push(Quad {
                            op,
                            arg1,
                            arg2,
                            result,
                        });
                    }
                }
                _ => {} // Unknown section or content within a section
            }
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), String> {
        if self.quads.is_empty() {
            return Err("No quadruples loaded to run.".to_string());
        }

        while self.ip < self.quads.len() {
            let quad = self.quads[self.ip];
            // println!("Executing IP: {}, Quad: {:?}", self.ip, quad); // Debug print

            match quad.op {
                OpCode::ASSIGN => {
                    // Handle special case for boolean assignment where arg2 indicates true/false
                    if quad.arg1 == -1 && (quad.arg2 == 0 || quad.arg2 == 1) {
                        // This is our special case for boolean literals (arg1 = -1, arg2 = 0 or 1)
                        if let Ok(idx) = self.get_bool_idx(quad.result) {
                            self.bool_memory[idx] = Some(quad.arg2 == 1);
                            self.ip += 1;
                            continue;
                        } else if let Ok(idx) = self.get_int_idx(quad.result) {
                            // Handle case where bool is assigned to int variable
                            self.int_memory[idx] = Some(quad.arg2);
                            self.ip += 1;
                            continue;
                        }
                    }
                    
                    // Normal assignment for all other types
                    let val = self.get_value(quad.arg1)?;
                    self.set_value(quad.result, val)?;
                    self.ip += 1;
                }
                OpCode::ADD | OpCode::SUB | OpCode::MULT | OpCode::DIV => {
                    let v1 = self.get_value(quad.arg1)?;
                    let v2 = self.get_value(quad.arg2)?;
                    let result_val = match (v1, v2) {
                        (VMValue::Int(i1), VMValue::Int(i2)) => match quad.op {
                            OpCode::ADD => VMValue::Int(i1 + i2),
                            OpCode::SUB => VMValue::Int(i1 - i2),
                            OpCode::MULT => VMValue::Int(i1 * i2),
                            OpCode::DIV => if i2 == 0 { return Err(format!("Division by zero: {} / {}", i1, i2))} else {VMValue::Int(i1 / i2)},
                            _ => unreachable!(),
                        },
                        (VMValue::Float(f1), VMValue::Float(f2)) => match quad.op {
                            OpCode::ADD => VMValue::Float(f1 + f2),
                            OpCode::SUB => VMValue::Float(f1 - f2),
                            OpCode::MULT => VMValue::Float(f1 * f2),
                            OpCode::DIV => if f2 == 0.0 { return Err(format!("Division by zero: {} / {}", f1, f2))} else {VMValue::Float(f1 / f2)},
                            _ => unreachable!(),
                        },
                        (VMValue::Int(i1), VMValue::Float(f2)) => {
                            let f1 = i1 as f64;
                            match quad.op {
                                OpCode::ADD => VMValue::Float(f1 + f2),
                                OpCode::SUB => VMValue::Float(f1 - f2),
                                OpCode::MULT => VMValue::Float(f1 * f2),
                                OpCode::DIV => if f2 == 0.0 { return Err(format!("Division by zero: {} / {}", f1, f2))} else {VMValue::Float(f1 / f2)},
                                _ => unreachable!(),
                            }
                        },
                        (VMValue::Float(f1), VMValue::Int(i2)) => {
                            let f2 = i2 as f64;
                            match quad.op {
                                OpCode::ADD => VMValue::Float(f1 + f2),
                                OpCode::SUB => VMValue::Float(f1 - f2),
                                OpCode::MULT => VMValue::Float(f1 * f2),
                                OpCode::DIV => if f2 == 0.0 { return Err(format!("Division by zero: {} / {}", f1, f2))} else {VMValue::Float(f1 / f2)},
                                _ => unreachable!(),
                            }
                        },
                        (VMValue::Bool(b1), VMValue::Bool(b2)) => match quad.op {
                            OpCode::ADD => VMValue::Int((b1 as i32) + (b2 as i32)),
                            OpCode::SUB => VMValue::Int((b1 as i32) - (b2 as i32)),
                            OpCode::MULT => VMValue::Int((b1 as i32) * (b2 as i32)),
                            OpCode::DIV => {
                                if !b2 { return Err("Division by zero (false)".to_string()); }
                                VMValue::Int((b1 as i32) / (b2 as i32))
                            },
                            _ => unreachable!(),
                        },
                        (VMValue::Bool(b1), VMValue::Int(i2)) => {
                            let i1 = b1 as i32;
                            match quad.op {
                                OpCode::ADD => VMValue::Int(i1 + i2),
                                OpCode::SUB => VMValue::Int(i1 - i2),
                                OpCode::MULT => VMValue::Int(i1 * i2),
                                OpCode::DIV => if i2 == 0 { return Err(format!("Division by zero: {} / {}", i1, i2))} else {VMValue::Int(i1 / i2)},
                                _ => unreachable!(),
                            }
                        },
                        (VMValue::Int(i1), VMValue::Bool(b2)) => {
                            let i2 = b2 as i32;
                            match quad.op {
                                OpCode::ADD => VMValue::Int(i1 + i2),
                                OpCode::SUB => VMValue::Int(i1 - i2),
                                OpCode::MULT => VMValue::Int(i1 * i2),
                                OpCode::DIV => if !b2 { return Err(format!("Division by zero: {} / false", i1))} else {VMValue::Int(i1 / i2)},
                                _ => unreachable!(),
                            }
                        },
                        (VMValue::Bool(b1), VMValue::Float(f2)) => {
                            let f1 = (b1 as i32) as f64;
                            match quad.op {
                                OpCode::ADD => VMValue::Float(f1 + f2),
                                OpCode::SUB => VMValue::Float(f1 - f2),
                                OpCode::MULT => VMValue::Float(f1 * f2),
                                OpCode::DIV => if f2 == 0.0 { return Err(format!("Division by zero: {} / {}", f1, f2))} else {VMValue::Float(f1 / f2)},
                                _ => unreachable!(),
                            }
                        },
                        (VMValue::Float(f1), VMValue::Bool(b2)) => {
                            let f2 = (b2 as i32) as f64;
                            match quad.op {
                                OpCode::ADD => VMValue::Float(f1 + f2),
                                OpCode::SUB => VMValue::Float(f1 - f2),
                                OpCode::MULT => VMValue::Float(f1 * f2),
                                OpCode::DIV => if !b2 { return Err(format!("Division by zero: {} / false", f1))} else {VMValue::Float(f1 / f2)},
                                _ => unreachable!(),
                            }
                        }
                    };
                    self.set_value(quad.result, result_val)?;
                    self.ip += 1;
                }
                OpCode::GT | OpCode::LT | OpCode::EQ | OpCode::NEQ => {
                    // Handle special case for direct boolean value in comparison
                    if quad.arg1 == -2 {
                        // Direct boolean literal in left operand, value in result field
                        let left_bool_value = quad.result == 1; // true if 1, false if 0
                        let v2 = self.get_value(quad.arg2)?;
                        
                        let bool_result = match v2 {
                            VMValue::Bool(b2) => match quad.op {
                                OpCode::EQ => left_bool_value == b2,
                                OpCode::NEQ => left_bool_value != b2,
                                _ => return Err(format!("Invalid comparison operator {} for boolean values", quad.op)),
                            },
                            VMValue::Int(i2) => match quad.op {
                                OpCode::EQ => (left_bool_value as i32) == i2,
                                OpCode::NEQ => (left_bool_value as i32) != i2,
                                _ => return Err(format!("Invalid comparison operator {} between Bool and Int", quad.op)),
                            },
                            VMValue::Float(f2) => match quad.op {
                                OpCode::EQ => (left_bool_value as i32) as f64 == f2,
                                OpCode::NEQ => (left_bool_value as i32) as f64 != f2,
                                _ => return Err(format!("Invalid comparison operator {} between Bool and Float", quad.op)),
                            }
                        };
                        
                        // Find the next instruction - need to figure out the result address since we're using result field for the boolean value
                        // The result address is typically stored in pila_o before this operation, so we need to extract it from a different place
                        let next_quad = if self.ip + 1 < self.quads.len() { Some(&self.quads[self.ip + 1]) } else { None };
                        if let Some(next_q) = next_quad {
                            if next_q.op == OpCode::GOTOF && next_q.arg1 >= TEMP_BOOL_START {
                                // Likely a conditional jump that uses our comparison result
                                self.set_value(next_q.arg1, VMValue::Bool(bool_result))?;
                            } else {
                                // Create a temporary address and store the result
                                let temp_addr = TEMP_BOOL_START + self.bool_memory.len() as i32;
                                let idx = self.get_bool_idx(temp_addr)?;
                                if idx >= self.bool_memory.len() {
                                    self.bool_memory.resize(idx + 1, None);
                                }
                                self.bool_memory[idx] = Some(bool_result);
                            }
                        }
                        self.ip += 1;
                        continue;
                    }
                    else if quad.arg2 == -2 {
                        // Direct boolean literal in right operand, value in result field
                        let v1 = self.get_value(quad.arg1)?;
                        let right_bool_value = quad.result == 1; // true if 1, false if 0
                        
                        let bool_result = match v1 {
                            VMValue::Bool(b1) => match quad.op {
                                OpCode::EQ => b1 == right_bool_value,
                                OpCode::NEQ => b1 != right_bool_value,
                                _ => return Err(format!("Invalid comparison operator {} for boolean values", quad.op)),
                            },
                            VMValue::Int(i1) => match quad.op {
                                OpCode::EQ => i1 == (right_bool_value as i32),
                                OpCode::NEQ => i1 != (right_bool_value as i32),
                                _ => return Err(format!("Invalid comparison operator {} between Int and Bool", quad.op)),
                            },
                            VMValue::Float(f1) => match quad.op {
                                OpCode::EQ => f1 == (right_bool_value as i32) as f64,
                                OpCode::NEQ => f1 != (right_bool_value as i32) as f64,
                                _ => return Err(format!("Invalid comparison operator {} between Float and Bool", quad.op)),
                            }
                        };
                        
                        // Find the next instruction (same approach as above)
                        let next_quad = if self.ip + 1 < self.quads.len() { Some(&self.quads[self.ip + 1]) } else { None };
                        if let Some(next_q) = next_quad {
                            if next_q.op == OpCode::GOTOF && next_q.arg1 >= TEMP_BOOL_START {
                                // Likely a conditional jump that uses our comparison result
                                self.set_value(next_q.arg1, VMValue::Bool(bool_result))?;
                            } else {
                                // Create a temporary address and store the result
                                let temp_addr = TEMP_BOOL_START + self.bool_memory.len() as i32;
                                let idx = self.get_bool_idx(temp_addr)?;
                                if idx >= self.bool_memory.len() {
                                    self.bool_memory.resize(idx + 1, None);
                                }
                                self.bool_memory[idx] = Some(bool_result);
                            }
                        }
                        self.ip += 1;
                        continue;
                    }

                    // Regular comparison handling
                    let v1 = self.get_value(quad.arg1)?;
                    let v2 = self.get_value(quad.arg2)?;
                    let bool_result = match (v1, v2) {
                        (VMValue::Int(i1), VMValue::Int(i2)) => match quad.op {
                            OpCode::GT => i1 > i2, 
                            OpCode::LT => i1 < i2, 
                            OpCode::EQ => i1 == i2, 
                            OpCode::NEQ => i1 != i2, 
                            _ => unreachable!(),
                        },
                        (VMValue::Float(f1), VMValue::Float(f2)) => match quad.op {
                            OpCode::GT => f1 > f2, 
                            OpCode::LT => f1 < f2, 
                            OpCode::EQ => f1 == f2, 
                            OpCode::NEQ => f1 != f2, 
                            _ => unreachable!(),
                        },
                        (VMValue::Int(i1), VMValue::Float(f2)) => {
                            let f1 = i1 as f64; 
                            match quad.op {
                                OpCode::GT => f1 > f2, 
                                OpCode::LT => f1 < f2, 
                                OpCode::EQ => f1 == f2, 
                                OpCode::NEQ => f1 != f2, 
                                _ => unreachable!(),
                            }
                        },
                        (VMValue::Float(f1), VMValue::Int(i2)) => {
                            let f2 = i2 as f64; 
                            match quad.op {
                                OpCode::GT => f1 > f2, 
                                OpCode::LT => f1 < f2, 
                                OpCode::EQ => f1 == f2, 
                                OpCode::NEQ => f1 != f2, 
                                _ => unreachable!(),
                            }
                        },
                        (VMValue::Bool(b1), VMValue::Bool(b2)) => match quad.op {
                            OpCode::EQ => b1 == b2, 
                            OpCode::NEQ => b1 != b2,
                            OpCode::GT | OpCode::LT => return Err(format!("Invalid comparison operator {} for boolean values", quad.op)),
                            _ => unreachable!(),
                        },
                        (VMValue::Bool(b1), VMValue::Int(i2)) => match quad.op {
                            OpCode::EQ => (b1 as i32) == i2, 
                            OpCode::NEQ => (b1 as i32) != i2,
                            _ => return Err(format!("Invalid comparison operator {} between Bool and Int", quad.op)),
                        },
                        (VMValue::Int(i1), VMValue::Bool(b2)) => match quad.op {
                            OpCode::EQ => i1 == (b2 as i32), 
                            OpCode::NEQ => i1 != (b2 as i32),
                            _ => return Err(format!("Invalid comparison operator {} between Int and Bool", quad.op)),
                        },
                        (VMValue::Bool(b1), VMValue::Float(f2)) => match quad.op {
                            OpCode::EQ => (b1 as i32) as f64 == f2,
                            OpCode::NEQ => (b1 as i32) as f64 != f2,
                            _ => return Err(format!("Invalid comparison operator {} between Bool and Float", quad.op)),
                        },
                        (VMValue::Float(f1), VMValue::Bool(b2)) => match quad.op {
                            OpCode::EQ => f1 == (b2 as i32) as f64,
                            OpCode::NEQ => f1 != (b2 as i32) as f64,
                            _ => return Err(format!("Invalid comparison operator {} between Float and Bool", quad.op)),
                        }
                    };
                    self.set_value(quad.result, VMValue::Bool(bool_result))?;
                    self.ip += 1;
                }
                OpCode::PRINT => {
                    let val = self.get_value(quad.arg1)?;
                    match val {
                        VMValue::Int(i) => {
                            // Heuristic: If the value came from a TEMP_BOOL address, print true/false
                            if quad.arg1 >= TEMP_BOOL_START && quad.arg1 <= self.max_temp_bool_addr {
                                println!("{}", if i == 0 { "false" } else { "true" });
                            } else {
                                println!("{}", i);
                            }
                        }
                        VMValue::Float(f) => println!("{}", f),
                        VMValue::Bool(b) => println!("{}", b),
                    }
                    self.ip += 1;
                }
                OpCode::GOTO => {
                    // quad.result contains the target IP
                    if quad.result < 0 || quad.result as usize >= self.quads.len() {
                        return Err(format!("GOTO: Invalid jump target {}", quad.result));
                    }
                    self.ip = quad.result as usize;
                }
                OpCode::GOTOF => {
                    let cond_val = self.get_value(quad.arg1)?;
                    let is_false = match cond_val {
                        VMValue::Bool(b) => !b,
                        VMValue::Int(i) => i == 0,
                        VMValue::Float(_) => return Err("GOTOF condition cannot be a float".to_string()),
                    };
                    
                    if is_false {
                        // Condition is false, jump to target
                        if quad.result < 0 || quad.result as usize >= self.quads.len() {
                            return Err(format!("GOTOF: Invalid jump target {}", quad.result));
                        }
                        self.ip = quad.result as usize;
                    } else {
                        // Condition is true, continue to next instruction
                        self.ip += 1;
                    }
                }
                OpCode::ERA => {
                    // quad.arg1 is the start_quad_idx of the function being called
                    if let Some(func_info) = self.functions.get(&quad.arg1) {
                        self.staged_params = Vec::with_capacity(func_info.param_count);
                    } else {
                        return Err(format!("ERA: Function with start_idx {} not found.", quad.arg1));
                    }
                    self.ip += 1;
                }
                OpCode::PARAM => {
                    let arg_val = self.get_value(quad.arg1)?; // Value to be passed
                    let param_k_idx = quad.result as usize;    // k-th parameter (0-indexed)

                    // Ensure staged_params is large enough. This handles cases where PARAMs might not be strictly sequential.
                    if param_k_idx >= self.staged_params.len() {
                        self.staged_params.resize_with(param_k_idx + 1, || VMValue::Int(-999)); // Dummy placeholder
                    }
                    self.staged_params[param_k_idx] = arg_val;
                    self.ip += 1;
                }
                OpCode::GOSUB => {
                    let target_func_start_idx = quad.arg1; // This is the func's start quad index

                    // First, validate the function exists and get the required info
                    let (func_name, param_count, param_addresses) = if let Some(func_info) = self.functions.get(&target_func_start_idx) {
                        (func_info.name.clone(), func_info.param_count, func_info.param_addresses.clone())
                    } else {
                        return Err(format!("GOSUB: Function with start_idx {} not found.", target_func_start_idx));
                    };

                    // Validate parameter count
                    if param_count != self.staged_params.len() {
                        return Err(format!("GOSUB: Mismatched param count for function '{}' (start_idx {}). Expected {}, got {} staged params.", func_name, target_func_start_idx, param_count, self.staged_params.len()));
                    }

                    // Clone staged_params to avoid borrowing issues
                    let staged_params_copy = self.staged_params.clone();

                    // Copy parameters to their destination addresses
                    for (k_idx, staged_val) in staged_params_copy.iter().enumerate() {
                        if k_idx < param_addresses.len() {
                            let param_dest_addr = param_addresses[k_idx];
                            self.set_value(param_dest_addr, staged_val.clone())?;
                        } else {
                            return Err(format!("GOSUB: Not enough destination addresses provided for function '{}' for param index {}.", func_name, k_idx));
                        }
                    }

                    // Push return address (next instruction after GOSUB)
                    self.call_stack.push(self.ip + 1);

                    // Jump to function start
                    if target_func_start_idx < 0 || target_func_start_idx as usize >= self.quads.len() {
                        return Err(format!("GOSUB: Invalid function start index {}", target_func_start_idx));
                    }
                    self.ip = target_func_start_idx as usize;

                    self.staged_params.clear(); // Clear after use
                }
                OpCode::ENDFUNC => {
                    if let Some(ret_ip) = self.call_stack.pop() {
                        if ret_ip >= self.quads.len() {
                            return Err(format!("ENDFUNC: Invalid return address {}", ret_ip));
                        }
                        self.ip = ret_ip;
                    } else {
                        // If call stack is empty and we hit ENDFUNC, this means we're returning from main
                        // In this case, we should treat it as program termination
                        return Ok(());
                    }
                }
                OpCode::HALT => {
                    // println!("Program halted at IP: {}.", self.ip);
                    return Ok(()); // End execution
                }
                _ => return Err(format!("Unknown OpCode: {} at IP: {}", quad.op, self.ip)),
            }
        }

        // If we exit the loop without hitting HALT or ENDFUNC, check if this is normal termination
        if self.call_stack.is_empty() {
            Ok(()) // Normal program termination
        } else {
            Err("Program ended without proper return from function calls".to_string())
        }
    }
}
