use std::collections::HashMap;
use crate::ast::{Program, FunctionDeclaration, Type};
use std::fmt;

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

/// Represents a function's metadata in the directory
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub return_type: Option<Type>,
    pub parameters: Vec<(String, Type)>,
    pub local_variables: HashMap<String, Type>,
    pub is_program: bool,  // Flag to indicate if this is the program entry
}

/// Function directory that stores information about all functions in a program
#[derive(Debug, Clone)]
pub struct FunctionDirectory {
    functions: HashMap<String, FunctionInfo>,
}

impl FunctionDirectory {
    /// Create a new empty function directory
    pub fn new() -> Self {
        FunctionDirectory {
            functions: HashMap::new(),
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
            global_vars.insert(var.id.clone(), var.var_type.clone());
        }
        
        directory.functions.insert("global".to_string(), FunctionInfo {
            return_type: None,
            parameters: Vec::new(),
            local_variables: global_vars,
            is_program: false,
        });
        
        // Add main function
        let main_vars = HashMap::new();
        
        directory.functions.insert("main".to_string(), FunctionInfo {
            return_type: None,
            parameters: Vec::new(),
            local_variables: main_vars,
            is_program: false,
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
        
        // Check for duplicate parameters
        for param in &func.parameters {
            // Check if parameter name already exists
            if param_names.contains_key(&param.id) {
                return Err(FunctionDirError::DuplicateVariable(
                    param.id.clone(),
                    format!("function {} parameters", func.id)
                ));
            }
            
            param_names.insert(param.id.clone(), ());
            params.push((param.id.clone(), param.param_type.clone()));
        }
        
        let mut local_vars = HashMap::new();
        
        // Check for duplicate local variables
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
            
            local_vars.insert(var.id.clone(), var.var_type.clone());
        }
        
        self.functions.insert(func.id.clone(), FunctionInfo {
            return_type: None, // BabyDuck doesn't specify return types in the grammar
            parameters: params,
            local_variables: local_vars,
            is_program: false,
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
            if let Some(var_type) = func_info.local_variables.get(variable_name) {
                return Some(var_type);
            }
        }
        
        // If not found and not in global scope, check global scope
        if function_name != "global" {
            if let Some(global_info) = self.functions.get("global") {
                return global_info.local_variables.get(variable_name);
            }
        }
        
        None
    }
    
    /// Get all functions as a reference to the internal HashMap
    pub fn get_all_functions(&self) -> &HashMap<String, FunctionInfo> {
        &self.functions
    }
    
    /// Get all global variables
    pub fn get_global_variables(&self) -> Option<&HashMap<String, Type>> {
        self.functions.get("global").map(|info| &info.local_variables)
    }
    
    /// Check if a function is the program entry
    pub fn is_program_entry(&self, name: &str) -> bool {
        match self.functions.get(name) {
            Some(info) => info.is_program,
            None => false,
        }
    }
}
