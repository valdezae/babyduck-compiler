use std::collections::{VecDeque};
use crate::ast::{Statement, Expression, Operator, Type, PrintStatement};
use crate::function_directory::FunctionDirectory;

/// Represents a quadruple instruction in the intermediate code with memory addresses
#[derive(Debug, Clone)]
pub struct Quadruple {
    pub operation: i32,  // Operation code instead of string
    pub arg1: i32,      // Memory address instead of string
    pub arg2: i32,      // Memory address instead of string
    pub result: i32,    // Memory address instead of string
}

/// Operation codes for quadruples
pub struct OpCode;
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
    pub const GOTOT: i32 = 32; 
    
    // Function call opcodes
    pub const ERA: i32 = 40;    // Activate Record / Function Call Setup
    pub const PARAM: i32 = 41;  // Parameter passing
    pub const GOSUB: i32 = 42;  // Go to Subroutine / Function Call
    pub const ENDFUNC: i32 = 43; // End of Function / Return
    pub const HALT: i32 = 50; // End of Program
}

/// Memory address ranges
pub struct MemoryAddresses;
impl MemoryAddresses {
    pub const INT_START: i32 = 1000;
    pub const FLOAT_START: i32 = 2000;
    pub const BOOL_START: i32 = 3000;  // Memory segment for boolean variables
    pub const CTE_INT_START: i32 = 4000;
    pub const CTE_FLOAT_START: i32 = 4500;
    pub const TEMP_INT_START: i32 = 5000;
    pub const TEMP_FLOAT_START: i32 = 6000;
    pub const TEMP_BOOL_START: i32 = 7000; // Temporary boolean variables
}

impl Quadruple {
    pub fn new(operation: i32, arg1: i32, arg2: i32, result: i32) -> Self {
        Quadruple {
            operation,
            arg1,
            arg2,
            result,
        }
    }

    pub fn to_string(&self) -> String {
        // Map operation code back to readable string for debugging
        let op_str = match self.operation {
            OpCode::ASSIGN => "=",
            OpCode::ADD => "+",
            OpCode::SUB => "-",
            OpCode::MULT => "*",
            OpCode::DIV => "/",
            OpCode::GT => ">",
            OpCode::LT => "<",
            OpCode::EQ => "==",
            OpCode::NEQ => "!=",
            OpCode::PRINT => "PRINT",
            OpCode::GOTO => "GOTO",
            OpCode::GOTOF => "GOTOF",
            OpCode::GOTOT => "GOTOT",
            OpCode::ERA => "ERA",
            OpCode::PARAM => "PARAM",
            OpCode::GOSUB => "GOSUB",
            OpCode::ENDFUNC => "ENDFUNC",
            OpCode::HALT => "HALT",
            _ => "UNKNOWN_OP",
        };

        format!("({}, {}, {}, {})", op_str, self.arg1, self.arg2, self.result)
    }

    pub fn to_string_with_names(&self, qg: &QuadrupleGenerator) -> String {
        // Map operation code back to readable string for debugging
        let op_str = match self.operation {
            OpCode::ASSIGN => "=",
            OpCode::ADD => "+",
            OpCode::SUB => "-",
            OpCode::MULT => "*",
            OpCode::DIV => "/",
            OpCode::GT => ">",
            OpCode::LT => "<",
            OpCode::EQ => "==",
            OpCode::NEQ => "!=",
            OpCode::PRINT => "PRINT",
            OpCode::GOTO => "GOTO",
            OpCode::GOTOF => "GOTOF",
            OpCode::GOTOT => "GOTOT",
            OpCode::ERA => "ERA",
            OpCode::PARAM => "PARAM",
            OpCode::GOSUB => "GOSUB",
            OpCode::ENDFUNC => "ENDFUNC",
            OpCode::HALT => "HALT",
            _ => "UNKNOWN_OP",
        };

        // Get variable names or values for the addresses
        let arg1_name = if self.operation == OpCode::ERA || self.operation == OpCode::GOSUB {
            qg.get_function_name_by_start_idx(self.arg1).unwrap_or_else(|| qg.get_name_by_address(self.arg1))
        } else {
            qg.get_name_by_address(self.arg1)
        };
        // For PARAM, result is param_index. For GOTO/GOTOF/GOTOT, result is jump target.
        let result_name = if self.operation == OpCode::PARAM || self.operation == OpCode::GOTO || self.operation == OpCode::GOTOF || self.operation == OpCode::GOTOT {
            self.result.to_string() // Show raw number for index/target
        } else {
            qg.get_name_by_address(self.result)
        };
        let arg2_name = qg.get_name_by_address(self.arg2); // Usually -1 for these ops


        format!("({}, {}, {}, {})", op_str, arg1_name, arg2_name, result_name)
    }
}

/// Handles the generation of quadruples for intermediate code
pub struct QuadrupleGenerator {
    // Stacks for compilation - renamed to match the image
    p_oper: Vec<i32>,            // operator stack 
    pila_o: Vec<i32>,           // operand stack 
    p_types: Vec<Type>,          // type stack 
    p_jumps: Vec<usize>,         // jumps stack - stores quadruple indices

    // Queue for generated quadruples
    quad_queue: VecDeque<Quadruple>,

    // Counters for memory addresses (only for temporaries and constants)
    temp_int_counter: i32, 
    temp_float_counter: i32,
    temp_bool_counter: i32,

    // Constant pools for storing literals - use address as index
    int_constants: Vec<i32>,       // Value stored at index [address - CTE_INT_START]
    float_constants: Vec<f64>,     // Value stored at index [address - CTE_FLOAT_START]
    bool_constants: Vec<bool>,     // New constant pool for booleans

    // Current function scope for variable lookup
    scope_stack: Vec<String>,

    // Reference to function directory
    pub(crate) function_directory: Option<FunctionDirectory>,
}

impl QuadrupleGenerator {
    pub fn new() -> Self {
        QuadrupleGenerator {
            p_oper: Vec::new(),
            pila_o: Vec::new(),
            p_types: Vec::new(),
            p_jumps: Vec::new(),    // Initialize jump stack
            quad_queue: VecDeque::new(),
            temp_int_counter: MemoryAddresses::TEMP_INT_START,
            temp_float_counter: MemoryAddresses::TEMP_FLOAT_START, // Base for float temporaries
            temp_bool_counter: MemoryAddresses::TEMP_BOOL_START,
            int_constants: Vec::new(),
            float_constants: Vec::new(),
            bool_constants: Vec::new(),  // Initialize bool constants vector
            scope_stack: vec!["global".to_string()], // Initialize with global scope
            function_directory: None
            
        }
    }

    /// Set the function directory for address resolution
    pub fn set_function_directory(&mut self, directory: FunctionDirectory) {
        self.function_directory = Some(directory);
    }

    /// Get the current scope from the top of the stack
    pub fn current_scope(&self) -> String {
        self.scope_stack.last().cloned().unwrap_or_else(|| {
            // This case should ideally not be reached if scope_stack is managed correctly
            eprintln!("Warning: Scope stack is empty, defaulting to 'unknown_scope'. This should not happen.");
            "unknown_scope".to_string()
        })
    }

    /// Get memory address for an identifier
    fn get_address(&self, id: &str) -> Option<i32> {
        let current_scope_val = self.current_scope();
        // Check if we have a function directory
        if let Some(ref directory) = self.function_directory {
            // Try to get from current scope
            if let Some(address) = directory.get_variable_address(&current_scope_val, id) {
                return Some(address);
            }

            // If not in current scope, check if it's a global variable
            if current_scope_val != "global" {
                if let Some(address) = directory.get_variable_address("global", id) {
                    return Some(address);
                }
            }
        }

        None
    }

    /// Get variable type for an identifier
    fn get_type(&self, id: &str) -> Option<Type> {
        let current_scope_val = self.current_scope();
        // Check if we have a function directory
        if let Some(ref directory) = self.function_directory {
            // Try to get from current scope
            if let Some(var_type) = directory.get_variable_type(&current_scope_val, id) {
                return Some(var_type.clone());
            }

            // If not in current scope, check if it's a global variable
            if current_scope_val != "global" {
                if let Some(var_type) = directory.get_variable_type("global", id) {
                    return Some(var_type.clone());
                }
            }
        }

        None
    }

    /// Get or create memory address for integer constant
    fn get_or_create_int_constant(&mut self, value: i32) -> i32 {
        // Search for existing constant
        for (index, &val) in self.int_constants.iter().enumerate() {
            if val == value {
                return MemoryAddresses::CTE_INT_START + index as i32;
            }
        }

        // Create new constant address
        let addr = MemoryAddresses::CTE_INT_START + self.int_constants.len() as i32;
        self.int_constants.push(value);
        addr
    }

    /// Get or create memory address for float constant
    fn get_or_create_float_constant(&mut self, value: f64) -> i32 {
        // Search for existing constant
        for (index, &val) in self.float_constants.iter().enumerate() {
            if val == value {
                return MemoryAddresses::CTE_FLOAT_START + index as i32;
            }
        }

        // Create new constant address
        let addr = MemoryAddresses::CTE_FLOAT_START + self.float_constants.len() as i32;
        self.float_constants.push(value);
        addr
    }

    /// Get or create memory address for boolean constant
    fn get_or_create_bool_constant(&mut self, value: bool) -> i32 {
        // Use temporary boolean memory segment instead of constant segment
        let addr = self.temp_bool_counter;
        self.temp_bool_counter += 1;
        
        // Store the value for later reference
        let index = (addr - MemoryAddresses::TEMP_BOOL_START) as usize;
        while self.bool_constants.len() <= index {
            self.bool_constants.push(false); // Pad with defaults if needed
        }
        self.bool_constants[index] = value;
        
        addr
    }

    /// Generate a new temporary variable address based on type
    fn new_temp(&mut self, typ: Type) -> i32 {
        match typ {
            Type::Int => {
                let temp = self.temp_int_counter;
                self.temp_int_counter += 1;
                temp
            },
            Type::Float => {
                let temp = self.temp_float_counter;
                self.temp_float_counter += 1;
                temp
            },
            Type::Bool => {
                let temp = self.temp_bool_counter;
                self.temp_bool_counter += 1;
                temp
            },
        }
    }

    /// Get next available temporary - this implements the AVAIL functionality
    fn avail_next(&mut self, typ: Type) -> i32 {
        self.new_temp(typ)
    }

    /// Get the resulting type from an operation between two types
    fn semantics(&self, left_type: &Type, right_type: &Type, operator: &Operator) -> Result<Type, String> {
        match (left_type, right_type, operator) {
            // Arithmetic operations
            (Type::Int, Type::Int, Operator::Plus) => Ok(Type::Int),
            (Type::Float, Type::Float, Operator::Plus) => Ok(Type::Float),
            (Type::Int, Type::Float, Operator::Plus) => Ok(Type::Float),
            (Type::Float, Type::Int, Operator::Plus) => Ok(Type::Float),

            (Type::Int, Type::Int, Operator::Minus) => Ok(Type::Int),
            (Type::Float, Type::Float, Operator::Minus) => Ok(Type::Float),
            (Type::Int, Type::Float, Operator::Minus) => Ok(Type::Float),
            (Type::Float, Type::Int, Operator::Minus) => Ok(Type::Float),

            (Type::Int, Type::Int, Operator::Multiply) => Ok(Type::Int),
            (Type::Float, Type::Float, Operator::Multiply) => Ok(Type::Float),
            (Type::Int, Type::Float, Operator::Multiply) => Ok(Type::Float),
            (Type::Float, Type::Int, Operator::Multiply) => Ok(Type::Float),

            (Type::Int, Type::Int, Operator::Divide) => Ok(Type::Int), // BabyDuck allows int/int -> int
            (Type::Float, Type::Float, Operator::Divide) => Ok(Type::Float),
            (Type::Int, Type::Float, Operator::Divide) => Ok(Type::Float),
            (Type::Float, Type::Int, Operator::Divide) => Ok(Type::Float),

            // Comparison operations - always produce boolean results
            (Type::Int, Type::Int, Operator::GreaterThan) => Ok(Type::Bool),
            (Type::Float, Type::Float, Operator::GreaterThan) => Ok(Type::Bool),
            (Type::Int, Type::Float, Operator::GreaterThan) => Ok(Type::Bool),
            (Type::Float, Type::Int, Operator::GreaterThan) => Ok(Type::Bool),

            (Type::Int, Type::Int, Operator::LessThan) => Ok(Type::Bool),
            (Type::Float, Type::Float, Operator::LessThan) => Ok(Type::Bool),
            (Type::Int, Type::Float, Operator::LessThan) => Ok(Type::Bool),
            (Type::Float, Type::Int, Operator::LessThan) => Ok(Type::Bool),

            (Type::Int, Type::Int, Operator::Equal) => Ok(Type::Bool),
            (Type::Float, Type::Float, Operator::Equal) => Ok(Type::Bool),
            (Type::Int, Type::Float, Operator::Equal) => Ok(Type::Bool), // Allow comparison between int and float
            (Type::Float, Type::Int, Operator::Equal) => Ok(Type::Bool),
            (Type::Bool, Type::Bool, Operator::Equal) => Ok(Type::Bool),

            (Type::Int, Type::Int, Operator::NotEqual) => Ok(Type::Bool),
            (Type::Float, Type::Float, Operator::NotEqual) => Ok(Type::Bool),
            (Type::Int, Type::Float, Operator::NotEqual) => Ok(Type::Bool),
            (Type::Float, Type::Int, Operator::NotEqual) => Ok(Type::Bool),
            (Type::Bool, Type::Bool, Operator::NotEqual) => Ok(Type::Bool),

            // Invalid operations
            _ => Err(format!("Type mismatch: {:?} and {:?} cannot be used with {:?}", left_type, right_type, operator))
        }
    }

    /// Process a list of statements and generate quadruples
    pub fn generate_from_statements(&mut self, statements: &[Statement]) {
        for statement in statements {
            self.process_statement(statement);
        }
    }

    /// Process a single statement
    fn process_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Assignment(assign) => self.process_assignment(assign),
            Statement::Print(print_stmt) => self.process_print(print_stmt),
            Statement::Condition(cond) => self.process_condition(cond),
            Statement::Cycle(cycle) => self.process_cycle(cycle),
            Statement::FunctionCall(func_call) => self.process_function_call(func_call),
        }
    }

    /// Process an assignment statement
    fn process_assignment(&mut self, assign: &crate::ast::Assignment) {
        // Process the expression on the right side
        self.process_expression(&assign.expression);

        // Pop result from stacks
        if let Some(result_addr) = self.pila_o.pop() {
            let result_type = self.p_types.pop().unwrap_or(Type::Int);

            // Get the target variable address
            if let Some(target_addr) = self.get_address(&assign.id) {
                // Special case for boolean literals
                if matches!(result_type, Type::Bool) {
                    // Check if we can directly determine the boolean value from the address
                    if let Ok(VMValue::Bool(bool_value)) = self.get_direct_bool_value(result_addr) {
                        // Generate assignment quadruple with direct 0/1 value instead of address
                        self.quad_queue.push_back(Quadruple::new(
                            OpCode::ASSIGN,
                            -1,  // Special marker for direct boolean value
                            if bool_value { 1 } else { 0 },  // 0 or 1 directly in arg2
                            target_addr
                        ));
                        return; // Skip the regular assignment path
                    }
                }

                // Regular assignment quadruple
                self.quad_queue.push_back(Quadruple::new(
                    OpCode::ASSIGN,
                    result_addr,
                    -1,  // Not used for assignment
                    target_addr
                ));
            } else {
                // This should ideally be caught by semantic analysis before quad generation
                eprintln!("Error: Variable '{}' not found in current or global scope during assignment.", assign.id);
            }
        } else {
            eprintln!("Error: No result on operand stack for assignment to '{}'.", assign.id);
        }
    }

    // Helper method to get a direct boolean value if the address is a constant
    fn get_direct_bool_value(&self, address: i32) -> Result<VMValue, &'static str> {
        // Check regular BOOL_START segment
        if address >= MemoryAddresses::BOOL_START && address < MemoryAddresses::CTE_INT_START {
            // Would need to access actual variable value at runtime
            return Err("Not a direct boolean constant");
        }
        // Check temporary TEMP_BOOL_START segment
        else if address >= MemoryAddresses::TEMP_BOOL_START {
            let index = (address - MemoryAddresses::TEMP_BOOL_START) as usize;
            if index < self.bool_constants.len() {
                return Ok(VMValue::Bool(self.bool_constants[index]));
            }
        }
        Err("Not a direct boolean constant")
    }

    /// Process a print statement
    fn process_print(&mut self, print_stmt: &PrintStatement) {
        match print_stmt {
            PrintStatement::Expression(expr) => {
                self.process_expression(expr);
                if let Some(value_addr) = self.pila_o.pop() {
                    self.p_types.pop(); // Remove type from stack
                    self.quad_queue.push_back(Quadruple::new(OpCode::PRINT, value_addr, -1, -1));
                } else {
                    eprintln!("Error: No result on operand stack for PRINT statement.");
                }
            }
        }
    }

    /// Process a conditional statement (if/else)
    fn process_condition(&mut self, cond: &crate::ast::Condition) {
        // 1. Process the condition expression
        self.process_expression(&cond.condition);

        // 2. Get the result from the expression evaluation
        if let Some(result_addr) = self.pila_o.pop() {
            let result_type = self.p_types.pop().unwrap_or(Type::Bool);

            // Check that the condition evaluates to a boolean result
            if !matches!(result_type, Type::Bool) {
                eprintln!("Warning: Condition for IF statement did not evaluate to a boolean result. Actual type: {:?}", result_type);
                // Potentially push a dummy bool to recover, or rely on runtime type checks
            }

            // 3. Generate GOTOF quadruple (goto false)
            let gotof_quad_idx = self.quad_queue.len();
            self.quad_queue.push_back(Quadruple::new(
                OpCode::GOTOF,
                result_addr,
                -1, // Not used
                -1 // Placeholder for jump destination
            ));

            // 4. Push jump position to jumps stack for GOTOF
            self.p_jumps.push(gotof_quad_idx);

            // 5. Process if-body statements
            self.generate_from_statements(&cond.if_body);

            // Check if there's an else clause
            if let Some(else_body) = &cond.else_body {
                // 6. Generate GOTO to skip over else-body once if-body completes
                let goto_quad_idx = self.quad_queue.len();
                self.quad_queue.push_back(Quadruple::new(
                    OpCode::GOTO,
                    -1, // Not used
                    -1, // Not used
                    -1 // Placeholder for jump destination after else
                ));

                // 7. Fill the pending GOTOF jump (from step 4) with the current quad position (start of else)
                let jump_target_for_gotof = self.quad_queue.len();
                let gotof_jump_pos_to_fill = self.p_jumps.pop().unwrap(); // Pop GOTOF jump
                self.fill_jump(gotof_jump_pos_to_fill, jump_target_for_gotof as i32);

                // 8. Push the GOTO position (from step 6) to jumps stack
                self.p_jumps.push(goto_quad_idx);

                // 9. Process else-body statements
                self.generate_from_statements(else_body);

                // 10. Fill the pending GOTO jump (from step 8) with the current quad position
                let jump_target_for_goto = self.quad_queue.len();
                let goto_jump_pos_to_fill = self.p_jumps.pop().unwrap(); // Pop GOTO jump
                self.fill_jump(goto_jump_pos_to_fill, jump_target_for_goto as i32);
            } else {
                // No else clause, fill the GOTOF (from step 4) with the current quad position
                let jump_target_for_gotof = self.quad_queue.len();
                let gotof_jump_pos_to_fill = self.p_jumps.pop().unwrap(); // Pop GOTOF jump
                self.fill_jump(gotof_jump_pos_to_fill, jump_target_for_gotof as i32);
            }
        } else {
            eprintln!("Error: No result on operand stack for IF condition.");
        }
    }

    /// Process a cycle statement (while)
    fn process_cycle(&mut self, cycle: &crate::ast::Cycle) {
        // 1. Save the position where we need to return for the next iteration (start of condition)
        let return_pos = self.quad_queue.len();
        self.p_jumps.push(return_pos); // Push return point for GOTO at end of loop body

        // 2. Process the condition expression
        self.process_expression(&cycle.condition);

        // 3. Get the result from the expression evaluation
        if let Some(result_addr) = self.pila_o.pop() {
            let result_type = self.p_types.pop().unwrap_or(Type::Bool);

            if !matches!(result_type, Type::Bool) {
                eprintln!("Warning: Cycle condition for WHILE did not evaluate to a boolean. Actual type: {:?}", result_type);
            }

            // 4. Generate GOTOF quadruple (goto false, exit loop)
            let gotof_quad_idx = self.quad_queue.len();
            self.quad_queue.push_back(Quadruple::new(
                OpCode::GOTOF,
                result_addr,
                -1, // Not used
                -1 // Placeholder for jump destination after the loop
            ));

            // 5. Push GOTOF jump position to jumps stack
            self.p_jumps.push(gotof_quad_idx);

            // 6. Process loop body statements
            self.generate_from_statements(&cycle.body);

            // 7. Generate GOTO to jump back to the condition evaluation
            let loop_return_target = self.p_jumps.pop().unwrap(); // This should be the GOTOF index
            let condition_start_target = self.p_jumps.pop().unwrap(); // This should be return_pos

            self.quad_queue.push_back(Quadruple::new(
                OpCode::GOTO,
                -1, // Not used
                -1, // Not used
                condition_start_target as i32 // Jump to condition evaluation
            ));

            // 8. Fill the pending GOTOF jump (from step 5) with the current quad position (after loop)
            let jump_target_after_loop = self.quad_queue.len();
            // let gotof_jump_pos_to_fill = self.p_jumps.pop().unwrap(); // GOTOF jump was popped above
            self.fill_jump(loop_return_target, jump_target_after_loop as i32);
        } else {
            eprintln!("Error: No result on operand stack for WHILE condition.");
        }
    }

    /// Fill a jump quadruple's target address
    fn fill_jump(&mut self, quad_idx: usize, target: i32) {
        if let Some(quad) = self.quad_queue.get_mut(quad_idx) {
            quad.result = target;
        } else {
            eprintln!("Error: Could not fill jump. Invalid quadruple index {}", quad_idx);
        }
    }

    /// Process a function call
    fn process_function_call(&mut self, func_call: &crate::ast::FunctionCall) {
        // Extract function info first to avoid borrowing conflicts
        let func_info = match self.function_directory.as_ref() {
            Some(dir) => match dir.get_function(&func_call.id) {
                Some(info) => info.clone(), // Clone the function info
                None => {
                    eprintln!("Error: Function '{}' not found in directory.", func_call.id);
                    return;
                }
            },
            None => {
                eprintln!("Error: Function directory not available for function call processing.");
                return;
            }
        };

        // 1. Arity check
        if func_call.arguments.len() != func_info.parameters.len() {
            eprintln!("Error: Function '{}' called with {} arguments, but expected {}.",
                      func_call.id, func_call.arguments.len(), func_info.parameters.len());
            return; // Or handle error appropriately
        }

        // 2. Generate ERA quad
        // The first argument to ERA will be the function's start_quad_idx, acting as an ID.
        let func_id_for_era = func_info.start_quad_idx.unwrap_or(-1); // Should be set by now
        if func_id_for_era == -1 {
            eprintln!("Error: start_quad_idx not set for function '{}' before call.", func_call.id);
        }
        self.quad_queue.push_back(Quadruple::new(OpCode::ERA, func_id_for_era, -1, -1));

        // 3. Process arguments and generate PARAM quads
        for (k, arg_expr) in func_call.arguments.iter().enumerate() {
            self.process_expression(arg_expr); // Evaluates expression, pushes result addr to PilaO, type to PTypes

            if let (Some(arg_addr), Some(arg_type)) = (self.pila_o.pop(), self.p_types.pop()) {
                let param_info = &func_info.parameters[k];
                let expected_param_type = &param_info.1;

                // Type check - need to access function_directory again for is_valid_assignment
                let is_valid = if let Some(dir) = self.function_directory.as_ref() {
                    dir.is_valid_assignment(expected_param_type, &arg_type)
                } else {
                    false // If no directory, assume invalid
                };

                if !is_valid {
                    eprintln!("Error: Type mismatch for argument {} of function '{}'. Expected {:?}, got {:?}.",
                              k + 1, func_call.id, expected_param_type, arg_type);
                    // Error handling
                }
                self.quad_queue.push_back(Quadruple::new(OpCode::PARAM, arg_addr, -1, k as i32));
            } else {
                eprintln!("Error: Missing operand/type for argument {} of function '{}'.", k + 1, func_call.id);
                // Error handling
                return;
            }
        }

        // 4. Generate GOSUB quad
        let func_target_quad = func_info.start_quad_idx.unwrap_or(-1); // Should be set
        self.quad_queue.push_back(Quadruple::new(OpCode::GOSUB, func_target_quad, -1, -1));
    }


    /// Internal method to enter a new scope
    fn enter_scope_internal(&mut self, scope_name: String) {
        self.scope_stack.push(scope_name);
    }

    /// Internal method to exit the current scope
    fn exit_scope_internal(&mut self) {
        if self.scope_stack.len() > 1 { // Ensure "global" or a base scope always remains
            self.scope_stack.pop();
        } else {
            // This should not happen if scopes are managed correctly starting from "global"
            eprintln!("Warning: Attempted to pop the base scope from the stack.");
        }
    }

    /// Generate quadruples for the entire program AST
    pub fn generate_for_program(&mut self, program_ast: &crate::ast::Program) -> Result<(), String> {
        if self.function_directory.is_none() {
            return Err("Function directory not set in QuadrupleGenerator.".to_string());
        }
        self.clear(); // Resets counters, stacks, and scope_stack to ["global"]

        // 1. Add a GOTO main quadruple (quad 0). Target will be filled later.
        let goto_main_quad_idx = self.quad_queue.len();
        assert_eq!(goto_main_quad_idx, 0, "GOTO main should be the first quadruple.");
        self.quad_queue.push_back(Quadruple::new(
            OpCode::GOTO,
            -1,
            -1,
            -1  // Placeholder for main's starting quadruple index
        ));

        // 2. Process non-main function declarations
        for func_decl in &program_ast.funcs {
            let func_start_idx = self.quad_queue.len() as i32;
            // Update FunctionDirectory with the start index
            if let Some(ref mut dir) = self.function_directory {
                dir.set_function_start_quad(&func_decl.id, func_start_idx);
            } else { return Err("Function directory lost during generation".to_string()); }

            self.enter_scope_internal(func_decl.id.clone());
            self.generate_from_statements(&func_decl.body);
            self.quad_queue.push_back(Quadruple::new(OpCode::ENDFUNC, -1, -1, -1));
            self.exit_scope_internal();
        }

        // 3. Determine main's start index and patch GOTO main
        let main_start_index = self.quad_queue.len() as i32;
        if let Some(ref mut dir) = self.function_directory {
            dir.set_function_start_quad("main", main_start_index);
        } else { return Err("Function directory lost before main generation".to_string()); }

        if let Some(quad) = self.quad_queue.get_mut(goto_main_quad_idx) {
            quad.result = main_start_index;
        } else {
            return Err("Internal error: Failed to access GOTO main quadruple for patching.".to_string());
        }

        // 4. Generate quadruples for the main block
        self.enter_scope_internal("main".to_string());
        self.generate_from_statements(&program_ast.main_body);
        self.quad_queue.push_back(Quadruple::new(OpCode::HALT, -1, -1, -1));
        self.exit_scope_internal(); // Return to "global" scope conceptually (though stack is empty except global)

        Ok(())
    }


    
    /// Action 1: PilaO.Push(id.name) and PTypes.Push(id.type)
    fn action_push_id(&mut self, id: &str) -> Result<i32, String> {
        // Look up the variable address and type from function directory
        if let Some(addr) = self.get_address(id) {
            if let Some(var_type) = self.get_type(id) {
                // Push to stacks (Action 1)
                self.pila_o.push(addr);
                self.p_types.push(var_type);
                return Ok(addr);
            }
        }

        Err(format!("Variable '{}' not found in scope '{}'", id, self.current_scope()))
    }

    /// Action 1 for constant literals
    fn action_push_constant(&mut self, value: i32, typ: Type) -> i32 {
        let addr = self.get_or_create_int_constant(value);
        self.pila_o.push(addr);
        self.p_types.push(typ);
        addr
    }

    fn action_push_float_constant(&mut self, value: f64) -> i32 {
        let addr = self.get_or_create_float_constant(value);
        self.pila_o.push(addr);
        self.p_types.push(Type::Float);
        addr
    }
    
    fn action_push_bool_constant(&mut self, value: bool) -> i32 {
        let addr = self.get_or_create_bool_constant(value);
        self.pila_o.push(addr);
        self.p_types.push(Type::Bool);
        addr
    }

    /// Action 2: POper.Push(* or /)
    fn action_push_mult_div_oper(&mut self, op: Operator) {
        let op_code = self.operator_to_code(&op);
        self.p_oper.push(op_code);
    }

    /// Action 3: POper.Push(+ or -)
    fn action_push_add_sub_oper(&mut self, op: Operator) {
        let op_code = self.operator_to_code(&op);
        self.p_oper.push(op_code);
    }

    /// Action 4 and 5: Process operations based on operator precedence
    fn action_process_operation(&mut self, is_mult_div: bool) {
        if self.p_oper.is_empty() {
            return;
        }

        let op = self.p_oper.last().cloned().unwrap();

        // Check if we should process the operation based on precedence
        let should_process = if is_mult_div {
            // For action 5 (higher precedence operations * /)
            op == OpCode::MULT || op == OpCode::DIV
        } else {
            // For action 4 (lower precedence operations + -)
            op == OpCode::ADD || op == OpCode::SUB
        };

        if should_process {
            // Pop the operator
            let operator = self.p_oper.pop().unwrap();

            // Make sure we have enough operands
            if self.pila_o.len() < 2 {
                eprintln!("Error: Not enough operands for operation code {}", operator);
                // Attempt to recover by pushing dummy values, or this could panic
                return;
            }

            // Pop right operand and type
            let right_operand = self.pila_o.pop().unwrap();
            let right_type = self.p_types.pop().unwrap();

            // Pop left operand and type
            let left_operand = self.pila_o.pop().unwrap();
            let left_type = self.p_types.pop().unwrap();

            // Perform type checking (semantics)
            let op_enum = self.code_to_operator(operator); // Convert code back to Operator enum
            let result_type_result = self.semantics(&left_type, &right_type, &op_enum);

            match result_type_result {
                Ok(result_type) => {
                    // Get next available temporary
                    let result_addr = self.avail_next(result_type.clone());

                    // Generate quadruple
                    let quad = Quadruple::new(operator, left_operand, right_operand, result_addr);
                    self.quad_queue.push_back(quad);

                    // Push result back to stacks
                    self.pila_o.push(result_addr);
                    self.p_types.push(result_type);
                },
                Err(msg) => {
                    eprintln!("Type error: {}", msg);
                    // Error recovery: push a placeholder (e.g., int) or handle more gracefully
                    let result_addr = self.avail_next(Type::Int); // Default to Int on error
                    self.pila_o.push(result_addr);
                    self.p_types.push(Type::Int);
                }
            }
        }
    }

    /// Process an expression and generate appropriate quadruples
    fn process_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::BinaryOp { left, operator, right } => {
                // Process left operand first
                self.process_expression(left);

                // Push operator to stack based on precedence
                match operator {
                    Operator::Multiply | Operator::Divide => {
                        // Action 2: Push * or / to operator stack
                        self.action_push_mult_div_oper(operator.clone());
                    },
                    Operator::Plus | Operator::Minus => {
                        // Action 3: Push + or - to operator stack
                        self.action_push_add_sub_oper(operator.clone());
                    },
                    // Comparison operators are pushed directly
                    Operator::GreaterThan | Operator::LessThan | Operator::Equal | Operator::NotEqual => {
                        let op_code = self.operator_to_code(operator);
                        self.p_oper.push(op_code);
                    }
                }

                // Process right operand
                self.process_expression(right);

                // Apply semantic actions based on operator
                match operator {
                    Operator::Multiply | Operator::Divide => {
                        // Action 5: Process * and / operations
                        self.action_process_operation(true);
                    },
                    Operator::Plus | Operator::Minus => {
                        // First check if there are any pending * or / operations (higher precedence)
                        self.action_process_operation(true); // Process any pending MULT/DIV

                        // Action 4: Process + and - operations
                        self.action_process_operation(false); // Process current ADD/SUB
                    },
                    // Comparison operators: >, <, ==, !=
                    Operator::GreaterThan | Operator::LessThan | Operator::Equal | Operator::NotEqual => {
                        // First handle any pending arithmetic operations (MULT/DIV, then ADD/SUB)
                        self.action_process_operation(true);  // Process pending MULT/DIV
                        self.action_process_operation(false); // Process pending ADD/SUB

                        // Then handle the comparison itself
                        if let Some(op_code) = self.p_oper.pop() {
                            // Ensure it's a comparison operator code
                            if op_code >= OpCode::GT && op_code <= OpCode::NEQ {
                                if self.pila_o.len() >= 2 && self.p_types.len() >= 2 {
                                    let right_addr = self.pila_o.pop().unwrap();
                                    let right_type = self.p_types.pop().unwrap();

                                    let left_addr = self.pila_o.pop().unwrap();
                                    let left_type = self.p_types.pop().unwrap();

                                    let op_enum = self.code_to_operator(op_code);
                                    match self.semantics(&left_type, &right_type, &op_enum) {
                                        Ok(result_type) => {
                                            // Special case for boolean literals in comparisons
                                            let mut special_case = false;
                                            
                                            // Check if we're comparing with a boolean literal
                                            if matches!(op_enum, Operator::Equal | Operator::NotEqual) {
                                                // Check if right operand is a boolean literal
                                                if let Ok(vm_value) = self.get_direct_bool_value(right_addr) {
                                                    if let VMValue::Bool(right_bool_value) = vm_value {
                                                        // Create a quadruple with direct 0/1 for the boolean literal
                                                        let result_temp_addr = self.avail_next(result_type.clone());
                                                        self.quad_queue.push_back(Quadruple::new(
                                                            op_code,
                                                            left_addr,
                                                            -2,  // Special marker for direct boolean value in comparison
                                                            if right_bool_value { 1 } else { 0 }  // Store bool value directly in result
                                                        ));
                                                        // Push the result address to the operand stack
                                                        self.pila_o.push(result_temp_addr);
                                                        self.p_types.push(result_type.clone());
                                                        special_case = true;
                                                    }
                                                }
                                                // Check if left operand is a boolean literal
                                                else if let Ok(vm_value) = self.get_direct_bool_value(left_addr) {
                                                    if let VMValue::Bool(left_bool_value) = vm_value {
                                                        // Create a quadruple with direct 0/1 for the boolean literal
                                                        let result_temp_addr = self.avail_next(result_type.clone());
                                                        self.quad_queue.push_back(Quadruple::new(
                                                            op_code,
                                                            -2,  // Special marker for direct boolean value in comparison
                                                            right_addr,
                                                            if left_bool_value { 1 } else { 0 }  // Store bool value directly in result
                                                        ));
                                                        // Push the result address to the operand stack
                                                        self.pila_o.push(result_temp_addr);
                                                        self.p_types.push(result_type.clone());
                                                        special_case = true;
                                                    }
                                                }
                                            }
                                            
                                            // Generate normal quadruple if not a special case
                                            if !special_case {
                                                let result_temp_addr = self.avail_next(result_type.clone());
                                                let quad = Quadruple::new(op_code, left_addr, right_addr, result_temp_addr);
                                                self.quad_queue.push_back(quad);
                                                self.pila_o.push(result_temp_addr);
                                                self.p_types.push(result_type);
                                            }
                                        },
                                        Err(e) => eprintln!("Type error during comparison: {}", e),
                                    }
                                } else {
                                    eprintln!("Error: Not enough operands/types for comparison op code {}", op_code);
                                }
                            } else {
                                // Should not happen if logic is correct, means a non-comparison op was popped
                                eprintln!("Error: Unexpected operator {} on stack when expecting comparison.", op_code);
                                self.p_oper.push(op_code); // Push it back
                            }
                        } else {
                            eprintln!("Error: Operator stack empty when expecting comparison operator.");
                        }
                    }
                }
            },
            Expression::Identifier(id) => {
                // Action 1: Push identifier to operand stack
                match self.action_push_id(id) {
                    Ok(_) => {},
                    Err(err) => eprintln!("Error during expression processing: {}", err),
                }
            },
            Expression::IntegerLiteral(value) => {
                // Action 1: Push constant to operand stack
                self.action_push_constant(*value, Type::Int);
            },
            Expression::FloatLiteral(value) => {
                // Action 1: Push float constant
                self.action_push_float_constant(*value);
            },
            Expression::BooleanLiteral(value) => {
                // Use the constant boolean memory segment
                self.action_push_bool_constant(*value);
            }
        }
    }

    /// Convert operator enum to operation code
    fn operator_to_code(&self, op: &Operator) -> i32 {
        match op {
            Operator::Plus => OpCode::ADD,
            Operator::Minus => OpCode::SUB,
            Operator::Multiply => OpCode::MULT,
            Operator::Divide => OpCode::DIV,
            Operator::GreaterThan => OpCode::GT,
            Operator::LessThan => OpCode::LT,
            Operator::Equal => OpCode::EQ,
            Operator::NotEqual => OpCode::NEQ,
        }
    }

    /// Convert operation code to operator enum for type checking
    fn code_to_operator(&self, code: i32) -> Operator {
        match code {
            OpCode::ADD => Operator::Plus,
            OpCode::SUB => Operator::Minus,
            OpCode::MULT => Operator::Multiply,
            OpCode::DIV => Operator::Divide,
            OpCode::GT => Operator::GreaterThan,
            OpCode::LT => Operator::LessThan,
            OpCode::EQ => Operator::Equal,
            OpCode::NEQ => Operator::NotEqual,
            _ => panic!("Unknown operator code: {} cannot be converted to Operator enum.", code),
        }
    }

    /// Get the generated quadruples
    pub fn get_quadruples(&self) -> &VecDeque<Quadruple> {
        &self.quad_queue
    }

    /// Get the generated quadruples and convert to string for display
    pub fn get_quadruples_as_strings(&self) -> Vec<String> {
        self.quad_queue.iter().map(|q| q.to_string()).collect()
    }

    /// Get the generated quadruples and convert to string with variable names for display
    pub fn get_quadruples_as_strings_with_names(&self) -> Vec<String> {
        self.quad_queue.iter().map(|q| q.to_string_with_names(self)).collect()
    }

    /// Get the variables from the function directory for debugging
    /// This shows variables for the *current* scope, or global if current is not "global".
    pub fn get_variables(&self) -> Vec<(String, i32)> {
        let mut vars = Vec::new();

        let current_scope_val = self.current_scope();
        if let Some(ref directory) = self.function_directory {
            // Add variables from current scope
            if let Some(func_info) = directory.get_function(&current_scope_val) {
                for (name, var_info) in &func_info.local_variables {
                    vars.push((name.clone(), var_info.address));
                }

                // Add parameters if the current scope is a function (not "global" or "main" without params)
                for (name, _, addr) in &func_info.parameters {
                    vars.push((name.clone(), *addr));
                }
            }

            // Add global variables if current scope is not "global" and they are not already shadowed
            // This part might be redundant if get_address always resolves to global if not local.
            // However, for display purposes, explicitly listing globals can be useful.
            if current_scope_val != "global" {
                if let Some(global_info) = directory.get_function("global") {
                    for (name, var_info) in &global_info.local_variables {
                        // Only add if not already in the list (shadowed by local/param)
                        if !vars.iter().any(|(var_name, _)| var_name == name) {
                            vars.push((name.clone(), var_info.address));
                        }
                    }
                }
            }
        }

        vars
    }

    /// Get the constant tables for debugging
    pub fn get_int_constants(&self) -> Vec<(i32, i32)> {
        self.int_constants.iter().enumerate()
            .map(|(index, &value)| (value, MemoryAddresses::CTE_INT_START + index as i32))
            .collect()
    }

    pub fn get_float_constants(&self) -> Vec<(f64, i32)> {
        self.float_constants.iter().enumerate()
            .map(|(index, &value)| (value, MemoryAddresses::CTE_FLOAT_START + index as i32))
            .collect()
    }
    
    pub fn get_bool_constants(&self) -> Vec<(bool, i32)> {
        self.bool_constants.iter().enumerate()
            .map(|(index, &value)| (value, MemoryAddresses::TEMP_BOOL_START + index as i32))
            .collect()
    }

    /// Get int constant value from address
    pub fn get_int_constant_value(&self, address: i32) -> Option<i32> {
        if address >= MemoryAddresses::CTE_INT_START && address < MemoryAddresses::CTE_FLOAT_START {
            let index = (address - MemoryAddresses::CTE_INT_START) as usize;
            return self.int_constants.get(index).copied();
        }
        None
    }

    /// Get float constant value from address
    pub fn get_float_constant_value(&self, address: i32) -> Option<f64> {
        if address >= MemoryAddresses::CTE_FLOAT_START && address < MemoryAddresses::TEMP_INT_START {
            let index = (address - MemoryAddresses::CTE_FLOAT_START) as usize;
            return self.float_constants.get(index).copied();
        }
        None
    }
    
    /// Get bool constant value from address
    pub fn get_bool_constant_value(&self, address: i32) -> Option<bool> {
        if address >= MemoryAddresses::TEMP_BOOL_START {
            let index = (address - MemoryAddresses::TEMP_BOOL_START) as usize;
            return self.bool_constants.get(index).copied();
        }
        None
    }

    /// Get variable or constant name by address
    pub fn get_name_by_address(&self, address: i32) -> String {
        if address == -1 {
            return "-".to_string(); // Placeholder for unused arguments
        }

        // Iterate through all known scopes in the function directory
        if let Some(ref directory) = self.function_directory {
            for (scope_name, func_info) in directory.get_all_functions() {
                // Check local variables in this scope
                for (var_name, var_info) in &func_info.local_variables {
                    if var_info.address == address {
                        return format!("{}.{} ({})", scope_name, var_name, address);
                    }
                }
                // Check parameters in this scope
                for (param_name, _, param_addr) in &func_info.parameters {
                    if *param_addr == address {
                        return format!("{}.{} (param) ({})", scope_name, param_name, address);
                    }
                }
            }
        }

        // Check if it's a temporary integer
        if address >= MemoryAddresses::TEMP_INT_START && address < MemoryAddresses::TEMP_FLOAT_START {
            return format!("t_int{} ({})", address - MemoryAddresses::TEMP_INT_START, address);
        }

        // Check if it's a temporary float
        if address >= MemoryAddresses::TEMP_FLOAT_START && address < MemoryAddresses::TEMP_BOOL_START {
            return format!("t_float{} ({})", address - MemoryAddresses::TEMP_FLOAT_START, address);
        }

        // Check if it's a temporary boolean
        if address >= MemoryAddresses::TEMP_BOOL_START {
            return format!("t_bool{} ({})", address - MemoryAddresses::TEMP_BOOL_START, address);
        }

        // Check if it's an integer constant
        if let Some(value) = self.get_int_constant_value(address) {
            return format!("{} (cte_int) ({})", value, address);
        }

        // Check if it's a float constant
        if let Some(value) = self.get_float_constant_value(address) {
            return format!("{:.1} (cte_float) ({})", value, address); // Format float
        }
        
        // Check if it's a boolean value
        if let Some(value) = self.get_bool_constant_value(address) {
            return format!("{} (temp_bool) ({})", value, address);
        }

        // If not found, return just the address as a string
        format!("addr:{}", address)
    }

    /// Get function name by its starting quadruple index
    pub fn get_function_name_by_start_idx(&self, start_idx: i32) -> Option<String> {
        if let Some(ref directory) = self.function_directory {
            for (func_name, func_info) in directory.get_all_functions() {
                if func_info.start_quad_idx == Some(start_idx) {
                    return Some(func_name.clone());
                }
            }
        }
        None
    }

    /// Clear all stacks, queues, and tables for reuse or a new compilation pass
    pub fn clear(&mut self) {
        self.p_oper.clear();
        self.pila_o.clear();
        self.p_types.clear();
        self.p_jumps.clear();  // Clear jumps stack
        self.quad_queue.clear();
        self.int_constants.clear();
        self.float_constants.clear();
        self.bool_constants.clear();  // Clear bool constants

        // Reset counters
        self.temp_int_counter = MemoryAddresses::TEMP_INT_START;
        self.temp_float_counter = MemoryAddresses::TEMP_FLOAT_START;
        self.temp_bool_counter = MemoryAddresses::TEMP_BOOL_START;

        // Reset scope stack to its initial state
        self.scope_stack = vec!["global".to_string()];
    }
}

// Add VMValue enum for internal use in the quadruple generator
#[derive(Debug)]
enum VMValue {
    Int(i32),
    Float(f64),
    Bool(bool),
}
