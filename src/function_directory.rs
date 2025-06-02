use std::collections::HashMap;
use crate::ast::{Program, FunctionDeclaration, Type};
use std::fmt;
use crate::quadruples::MemoryAddresses;

/// Custom error type for function directory operations
#[derive(Debug)]
pub enum FunctionDirError {
    DuplicateVariable(String, String), // (var_name, scope_name)
    DuplicateFunction(String),
    // Can add more error types as needed
}

impl fmt::Display for FunctionDirError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FunctionDirError::DuplicateVariable(var, scope) =>
                write!(f, "Duplicate variable '{}' in scope '{}'", var, scope),
            FunctionDirError::DuplicateFunction(func) =>
                write!(f, "Duplicate function name '{}'", func),
        }
    }
}

/// Represents a variable with its type and memory address
#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub var_type: Type,
    pub address: i32,  // Memory address where the variable is stored
}

/// Represents a function in the directory
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub return_type: Option<Type>,
    pub parameters: Vec<(String, Type, i32)>,  // (name, type, address)
    pub local_variables: HashMap<String, VariableInfo>,
    pub is_program: bool,  // Flag to indicate if this is the program entry
    pub start_quad_idx: Option<i32>, // Starting quadruple index for the function
}

/// Function directory that stores information about all functions in a program
#[derive(Debug, Clone)]
pub struct FunctionDirectory {
    functions: HashMap<String, FunctionInfo>,
    // Memory address counters
    int_counter: i32,
    float_counter: i32,
    bool_counter: i32,
}

impl FunctionDirectory {
    /// Create a new empty function directory
    pub fn new() -> Self {
        FunctionDirectory {
            functions: HashMap::new(),
            int_counter: MemoryAddresses::INT_START,    // Starting at base addresses defined in quadruples.rs
            float_counter: MemoryAddresses::FLOAT_START,
            bool_counter: MemoryAddresses::BOOL_START,   // Using dedicated bool addresses
        }
    }

    /// Get a new memory address for a variable based on its type
    fn get_next_address(&mut self, var_type: &Type) -> i32 {
        match var_type {
            Type::Int => {
                let addr = self.int_counter;
                self.int_counter += 1;
                addr
            },
            Type::Float => {
                let addr = self.float_counter;
                self.float_counter += 1;
                addr
            },
            Type::Bool => {
                let addr = self.bool_counter;
                self.bool_counter += 1;
                addr
            },
        }
    }

    /// Create a function directory from an AST Program
    pub fn from_program(program: &Program) -> Result<Self, FunctionDirError> {
        let mut directory = Self::new();

        // Add program as a special function entry
        directory.functions.insert(program.id.clone(), FunctionInfo {
            return_type: None,
            parameters: Vec::new(),
            local_variables: HashMap::new(),
            is_program: true,
            start_quad_idx: None,
        });

        // Add global variables to a special "global" entry
        let mut global_vars = HashMap::new();
        for var in &program.vars {
            // Check for duplicate global variables
            if global_vars.contains_key(&var.id) {
                return Err(FunctionDirError::DuplicateVariable(
                    var.id.clone(),
                    "global".to_string()
                ));
            }

            // Assign a memory address based on the variable type
            let address = directory.get_next_address(&var.var_type);

            global_vars.insert(var.id.clone(), VariableInfo {
                var_type: var.var_type.clone(),
                address,
            });
        }

        directory.functions.insert("global".to_string(), FunctionInfo {
            return_type: None,
            parameters: Vec::new(),
            local_variables: global_vars,
            is_program: false,
            start_quad_idx: None,
        });

        // Add main function
        let main_vars = HashMap::new();

        directory.functions.insert("main".to_string(), FunctionInfo {
            return_type: None,
            parameters: Vec::new(),
            local_variables: main_vars,
            is_program: false,
            start_quad_idx: None, // Will be set during quad generation
        });

        // Add all other functions
        for func in &program.funcs {
            directory.add_function(func)?;
        }

        Ok(directory)
    }

    /// Add a function to the directory
    pub fn add_function(&mut self, func: &FunctionDeclaration) -> Result<(), FunctionDirError> {
        // Check for duplicate function name
        if self.functions.contains_key(&func.id) {
            return Err(FunctionDirError::DuplicateFunction(func.id.clone()));
        }

        let mut params = Vec::new();
        let mut param_names = HashMap::new();

        // Check for duplicate parameters and assign addresses
        for param in &func.parameters {
            // Check if parameter name already exists
            if param_names.contains_key(&param.id) {
                return Err(FunctionDirError::DuplicateVariable(
                    param.id.clone(),
                    format!("function {} parameters", func.id)
                ));
            }

            // Assign a memory address based on the parameter type
            let address = self.get_next_address(&param.param_type);

            param_names.insert(param.id.clone(), ());
            params.push((param.id.clone(), param.param_type.clone(), address));
        }

        let mut local_vars = HashMap::new();

        // Check for duplicate local variables and assign addresses
        for var in &func.vars {
            // Check if variable name already exists as a local variable
            if local_vars.contains_key(&var.id) {
                return Err(FunctionDirError::DuplicateVariable(
                    var.id.clone(),
                    format!("{}", func.id)
                ));
            }

            // Check if variable name already exists as a parameter
            if param_names.contains_key(&var.id) {
                return Err(FunctionDirError::DuplicateVariable(
                    var.id.clone(),
                    format!("function {} (parameter conflict)", func.id)
                ));
            }

            // Assign a memory address based on the variable type
            let address = self.get_next_address(&var.var_type);

            local_vars.insert(var.id.clone(), VariableInfo {
                var_type: var.var_type.clone(),
                address,
            });
        }

        self.functions.insert(func.id.clone(), FunctionInfo {
            return_type: None, // BabyDuck doesn't specify return types in the grammar
            parameters: params,
            local_variables: local_vars,
            is_program: false,
            start_quad_idx: None, // Will be set during quad generation
        });

        Ok(())
    }

    // Debug functions

    /// Get information about a function by name
    pub fn get_function(&self, name: &str) -> Option<&FunctionInfo> {
        self.functions.get(name)
    }

    /// Check if a function exists
    pub fn function_exists(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// Get a variable's type from a function (or from global if not found)
    pub fn get_variable_type(&self, function_name: &str, variable_name: &str) -> Option<&Type> {
        // Check if variable exists in the function's local scope
        if let Some(func_info) = self.functions.get(function_name) {
            if let Some(var_info) = func_info.local_variables.get(variable_name) {
                return Some(&var_info.var_type);
            }

            // Check if it's a parameter
            for (param_name, param_type, _) in &func_info.parameters {
                if param_name == variable_name {
                    return Some(param_type);
                }
            }
        }

        // If not found and not in global scope, check global scope
        if function_name != "global" {
            if let Some(global_info) = self.functions.get("global") {
                if let Some(var_info) = global_info.local_variables.get(variable_name) {
                    return Some(&var_info.var_type);
                }
            }
        }

        None
    }

    /// Get a variable's address from a function (or from global if not found)
    pub fn get_variable_address(&self, function_name: &str, variable_name: &str) -> Option<i32> {
        // Check if variable exists in the function's local scope
        if let Some(func_info) = self.functions.get(function_name) {
            if let Some(var_info) = func_info.local_variables.get(variable_name) {
                return Some(var_info.address);
            }

            // Check if it's a parameter
            for (param_name, _, param_addr) in &func_info.parameters {
                if param_name == variable_name {
                    return Some(*param_addr);
                }
            }
        }

        // If not found and not in global scope, check global scope
        if function_name != "global" {
            if let Some(global_info) = self.functions.get("global") {
                if let Some(var_info) = global_info.local_variables.get(variable_name) {
                    return Some(var_info.address);
                }
            }
        }

        None
    }

    /// Get all functions as a reference to the internal HashMap
    pub fn get_all_functions(&self) -> &HashMap<String, FunctionInfo> {
        &self.functions
    }

    /// Get all global variables
    pub fn get_global_variables(&self) -> Option<&HashMap<String, VariableInfo>> {
        self.functions.get("global").map(|info| &info.local_variables)
    }

    /// Check if a function is the program entry
    pub fn is_program_entry(&self, name: &str) -> bool {
        match self.functions.get(name) {
            Some(info) => info.is_program,
            None => false,
        }
    }

    /// Check if a type assignment is valid based on the semantic cube rules
    pub fn is_valid_assignment(&self, target_type: &Type, value_type: &Type) -> bool {
        match (target_type, value_type) {
            // Same types are always valid
            (Type::Int, Type::Int) => true,
            (Type::Float, Type::Float) => true,
            (Type::Bool, Type::Bool) => true,

            // Int can be assigned to float (but with possible precision loss)
            (Type::Float, Type::Int) => true,

            // A boolean result can be assigned to a boolean variable
            (Type::Bool, _) => true,  // Allowing any type to bool for comparison results

            // All other combinations are invalid
            _ => false,
        }
    }

    /// Set the starting quadruple index for a function
    pub fn set_function_start_quad(&mut self, func_name: &str, start_idx: i32) {
        if let Some(info) = self.functions.get_mut(func_name) {
            info.start_quad_idx = Some(start_idx);
        }
    }
}
