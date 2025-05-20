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
}

/// Memory address ranges
pub struct MemoryAddresses;
impl MemoryAddresses {
    pub const INT_START: i32 = 1000;
    pub const FLOAT_START: i32 = 2000;
    pub const CTE_INT_START: i32 = 3000;
    pub const CTE_FLOAT_START: i32 = 3500;
    pub const TEMP_INT_START: i32 = 4000;
    pub const TEMP_FLOAT_START: i32 = 5000;
    pub const TEMP_BOOL_START: i32 = 6000;
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
            _ => "UNKNOWN",
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
            _ => "UNKNOWN",
        };

        // Get variable names or values for the addresses
        let arg1_name = qg.get_name_by_address(self.arg1);
        let arg2_name = qg.get_name_by_address(self.arg2);
        let result_name = qg.get_name_by_address(self.result);

        format!("({}, {}, {}, {})", op_str, arg1_name, arg2_name, result_name)
    }
}

/// Handles the generation of quadruples for intermediate code
pub struct QuadrupleGenerator {
    // Stacks for compilation - renamed to match the image
    p_oper: Vec<i32>,            // operator stack (POper in the image)
    pila_o: Vec<i32>,           // operand stack (PilaO in the image)
    p_types: Vec<Type>,          // type stack (PTypes in the image)
    p_jumps: Vec<usize>,        // jumps stack (PSaltos in the image) - stores quadruple indices

    // Queue for generated quadruples
    quad_queue: VecDeque<Quadruple>,

    // Counters for memory addresses (only for temporaries and constants)
    temp_int_counter: i32,
    temp_float_counter: i32,
    temp_bool_counter: i32,

    // Constant pools for storing literals - use address as index
    int_constants: Vec<i32>,       // Value stored at index [address - CTE_INT_START]
    float_constants: Vec<f64>,     // Value stored at index [address - CTE_FLOAT_START]

    // Current function scope for variable lookup
    current_scope: String,

    // Reference to function directory
    function_directory: Option<FunctionDirectory>,

    // Next available temporary address by type
    avail: i32,
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
            temp_float_counter: MemoryAddresses::TEMP_FLOAT_START,
            temp_bool_counter: MemoryAddresses::TEMP_BOOL_START,
            int_constants: Vec::new(),
            float_constants: Vec::new(),
            current_scope: "global".to_string(),
            function_directory: None,
            avail: 0,
        }
    }

    /// Set the function directory for address resolution
    pub fn set_function_directory(&mut self, directory: FunctionDirectory) {
        self.function_directory = Some(directory);
    }

    /// Set the current scope for variable lookups
    pub fn set_current_scope(&mut self, scope: &str) {
        self.current_scope = scope.to_string();
    }

    /// Get memory address for an identifier
    fn get_address(&self, id: &str) -> Option<i32> {
        // Check if we have a function directory
        if let Some(ref directory) = self.function_directory {
            // Try to get from current scope
            if let Some(address) = directory.get_variable_address(&self.current_scope, id) {
                return Some(address);
            }

            // If not in current scope, check if it's a global variable
            if self.current_scope != "global" {
                if let Some(address) = directory.get_variable_address("global", id) {
                    return Some(address);
                }
            }
        }
        
        None
    }

    /// Get variable type for an identifier
    fn get_type(&self, id: &str) -> Option<Type> {
        // Check if we have a function directory
        if let Some(ref directory) = self.function_directory {
            // Try to get from current scope
            if let Some(var_type) = directory.get_variable_type(&self.current_scope, id) {
                return Some(var_type.clone());
            }

            // If not in current scope, check if it's a global variable
            if self.current_scope != "global" {
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

            (Type::Int, Type::Int, Operator::Divide) => Ok(Type::Int),
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
            (Type::Int, Type::Float, Operator::Equal) => Ok(Type::Bool),
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
                // Generate assignment quadruple
                self.quad_queue.push_back(Quadruple::new(
                    OpCode::ASSIGN,
                    result_addr,
                    -1,  // Not used for assignment
                    target_addr
                ));
            } else {
                println!("Error: Variable '{}' not found in current or global scope", assign.id);
                // Recovery: we could either abort, create a dummy address, or continue silently
            }
        }
    }

    /// Process a print statement
    fn process_print(&mut self, print_stmt: &PrintStatement) {
        match print_stmt {
            PrintStatement::Expression(expr) => {
                self.process_expression(expr);
                if let Some(value_addr) = self.pila_o.pop() {
                    self.p_types.pop();
                    self.quad_queue.push_back(Quadruple::new(OpCode::PRINT, value_addr, -1, -1));
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
                println!("Warning: Condition should evaluate to a boolean result");
            }
            
            // 3. Generate GOTOF quadruple (goto false)
            //    The target address is initially set to -1 and will be filled later
            let gotof_quad_idx = self.quad_queue.len();
            self.quad_queue.push_back(Quadruple::new(
                OpCode::GOTOF,
                result_addr,
                -1,
                -1 // Placeholder for jump destination
            ));
            
            // 4. Push jump position to jumps stack
            self.p_jumps.push(gotof_quad_idx);
            
            // 5. Process if-body statements
            self.generate_from_statements(&cond.if_body);
            
            // Check if there's an else clause
            if let Some(else_body) = &cond.else_body {
                // 6. Generate GOTO to skip over else-body once if-body completes
                let goto_quad_idx = self.quad_queue.len();
                self.quad_queue.push_back(Quadruple::new(
                    OpCode::GOTO,
                    -1,
                    -1,
                    -1 // Placeholder for jump destination after else
                ));
                
                // 7. Fill the pending GOTOF jump with the current quad position
                let jump_target = self.quad_queue.len();
                self.fill_jump(gotof_quad_idx, jump_target as i32);
                
                // 8. Push the GOTO position to jumps stack
                self.p_jumps.push(goto_quad_idx);
                
                // 9. Process else-body statements
                self.generate_from_statements(else_body);
                
                // 10. Fill the pending GOTO jump with the current quad position
                let jump_target = self.quad_queue.len();
                let jump_pos = self.p_jumps.pop().unwrap(); // GOTO jump
                self.fill_jump(jump_pos, jump_target as i32);
            } else {
                // 6b. No else clause, fill the GOTOF with the current quad position
                let jump_target = self.quad_queue.len();
                let jump_pos = self.p_jumps.pop().unwrap(); // GOTOF jump
                self.fill_jump(jump_pos, jump_target as i32);
            }
        }
    }

    /// Process a cycle statement (while)
    fn process_cycle(&mut self, cycle: &crate::ast::Cycle) {
        // 1. Save the position where we need to return for the next iteration
        let return_pos = self.quad_queue.len();
        
        // 2. Process the condition expression
        self.process_expression(&cycle.condition);
        
        // 3. Get the result from the expression evaluation
        if let Some(result_addr) = self.pila_o.pop() {
            let result_type = self.p_types.pop().unwrap_or(Type::Bool);
            
            // Check that the condition evaluates to a boolean result
            if !matches!(result_type, Type::Bool) {
                println!("Warning: Cycle condition should evaluate to a boolean result");
            }
            
            // 4. Generate GOTOF quadruple (goto false)
            //    The target address is initially set to -1 and will be filled later
            let gotof_quad_idx = self.quad_queue.len();
            self.quad_queue.push_back(Quadruple::new(
                OpCode::GOTOF,
                result_addr,
                -1,
                -1 // Placeholder for jump destination after the loop
            ));
            
            // 5. Push jump position to jumps stack
            self.p_jumps.push(gotof_quad_idx);
            
            // 6. Process loop body statements
            self.generate_from_statements(&cycle.body);
            
            // 7. Generate GOTO to jump back to the condition evaluation
            self.quad_queue.push_back(Quadruple::new(
                OpCode::GOTO,
                -1,
                -1,
                return_pos as i32 // Jump to condition evaluation
            ));
            
            // 8. Fill the pending GOTOF jump with the current quad position
            let jump_target = self.quad_queue.len();
            let jump_pos = self.p_jumps.pop().unwrap(); // GOTOF jump
            self.fill_jump(jump_pos, jump_target as i32);
        }
    }

    /// Fill a jump quadruple's target address
    fn fill_jump(&mut self, quad_idx: usize, target: i32) {
        if let Some(quad) = self.quad_queue.get_mut(quad_idx) {
            quad.result = target;
        } else {
            println!("Error: Could not fill jump at index {}", quad_idx);
        }
    }

    /// Process a function call
    fn process_function_call(&mut self, _func_call: &crate::ast::FunctionCall) {
        // For the current implementation, we'll focus only on arithmetic expressions and assignments
        // This would be implemented in a more complete compiler
    }

    /// Match the semantic actions in the image
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
        
        Err(format!("Variable '{}' not found in scope '{}'", id, self.current_scope))
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
            // For action 5 (higher precedence operations)
            op == OpCode::MULT || op == OpCode::DIV
        } else {
            // For action 4 (lower precedence operations)
            op == OpCode::ADD || op == OpCode::SUB
        };

        if should_process {
            // Pop the operator
            let operator = self.p_oper.pop().unwrap();

            // Make sure we have enough operands
            if self.pila_o.len() < 2 {
                println!("Error: Not enough operands for operation");
                return;
            }

            // Pop right operand and type
            let right_operand = self.pila_o.pop().unwrap();
            let right_type = self.p_types.pop().unwrap();

            // Pop left operand and type
            let left_operand = self.pila_o.pop().unwrap();
            let left_type = self.p_types.pop().unwrap();

            // Perform type checking (semantics)
            let op_enum = self.code_to_operator(operator);
            let result_type = self.semantics(&left_type, &right_type, &op_enum);

            match result_type {
                Ok(result_type) => {
                    // Get next available temporary
                    let result = self.avail_next(result_type.clone());

                    // Generate quadruple
                    let quad = Quadruple::new(operator, left_operand, right_operand, result);
                    self.quad_queue.push_back(quad);

                    // Push result back to stacks
                    self.pila_o.push(result);
                    self.p_types.push(result_type);
                },
                Err(msg) => {
                    println!("Type error: {}", msg);
                    // Error recovery - push placeholder
                    let result = self.avail_next(Type::Int);
                    self.pila_o.push(result);
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
                    _ => {
                        // Other operators like comparison operators
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
                        self.action_process_operation(true);

                        // Action 4: Process + and - operations
                        self.action_process_operation(false);
                    },
                    _ => {
                        // For comparison operators
                        // First handle any arithmetic operations
                        self.action_process_operation(true);
                        self.action_process_operation(false);

                        // Then handle the comparison
                        if let Some(op) = self.p_oper.pop() {
                            if self.pila_o.len() >= 2 {
                                let right = self.pila_o.pop().unwrap();
                                let right_type = self.p_types.pop().unwrap();

                                let left = self.pila_o.pop().unwrap();
                                let left_type = self.p_types.pop().unwrap();

                                let op_enum = self.code_to_operator(op);
                                let result_type = self.semantics(&left_type, &right_type, &op_enum);

                                if let Ok(typ) = result_type {
                                    let result = self.avail_next(typ.clone());

                                    let quad = Quadruple::new(op, left, right, result);
                                    self.quad_queue.push_back(quad);

                                    self.pila_o.push(result);
                                    self.p_types.push(typ);
                                }
                            }
                        }
                    }
                }
            },
            Expression::Identifier(id) => {
                // Action 1: Push identifier to operand stack
                match self.action_push_id(id) {
                    Ok(_) => {},
                    Err(err) => println!("Error: {}", err),
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
                // Storing boolean as integer constant
                let int_value = if *value { 1 } else { 0 };
                self.action_push_constant(int_value, Type::Bool);
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
            _ => panic!("Unknown operator code: {}", code),
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
    pub fn get_variables(&self) -> Vec<(String, i32)> {
        let mut vars = Vec::new();
        
        if let Some(ref directory) = self.function_directory {
            // Add variables from current scope
            if let Some(func_info) = directory.get_function(&self.current_scope) {
                for (name, var_info) in &func_info.local_variables {
                    vars.push((name.clone(), var_info.address));
                }
                
                // Add parameters
                for (name, _, addr) in &func_info.parameters {
                    vars.push((name.clone(), *addr));
                }
            }
            
            // Add global variables if not already in current scope
            if self.current_scope != "global" {
                if let Some(global_info) = directory.get_function("global") {
                    for (name, var_info) in &global_info.local_variables {
                        // Only add if not already in the list
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

    /// Get int constant value from address
    pub fn get_int_constant_value(&self, address: i32) -> Option<i32> {
        let index = (address - MemoryAddresses::CTE_INT_START) as usize;
        self.int_constants.get(index).copied()
    }

    /// Get float constant value from address
    pub fn get_float_constant_value(&self, address: i32) -> Option<f64> {
        let index = (address - MemoryAddresses::CTE_FLOAT_START) as usize;
        self.float_constants.get(index).copied()
    }

    /// Get variable or constant name by address
    pub fn get_name_by_address(&self, address: i32) -> String {
        if address == -1 {
            return "-".to_string();
        }

        // Check if it's a variable in the function directory
        if let Some(ref directory) = self.function_directory {
            // First check current scope
            if let Some(func_info) = directory.get_function(&self.current_scope) {
                // Check local variables
                for (name, var_info) in &func_info.local_variables {
                    if var_info.address == address {
                        return format!("{} ({})", name, address);
                    }
                }
                
                // Check parameters
                for (name, _, addr) in &func_info.parameters {
                    if *addr == address {
                        return format!("{} ({})", name, address);
                    }
                }
            }
            
            // Then check global scope if not already checked
            if self.current_scope != "global" {
                if let Some(global_info) = directory.get_function("global") {
                    for (name, var_info) in &global_info.local_variables {
                        if var_info.address == address {
                            return format!("{} ({})", name, address);
                        }
                    }
                }
            }
        }

        // Check if it's a temporary integer
        if address >= MemoryAddresses::TEMP_INT_START && address < MemoryAddresses::TEMP_FLOAT_START {
            return format!("t{} ({})", address - MemoryAddresses::TEMP_INT_START, address);
        }

        // Check if it's a temporary float
        if address >= MemoryAddresses::TEMP_FLOAT_START && address < MemoryAddresses::TEMP_BOOL_START {
            return format!("t{} ({})", address - MemoryAddresses::TEMP_FLOAT_START, address);
        }

        // Check if it's a temporary boolean
        if address >= MemoryAddresses::TEMP_BOOL_START {
            return format!("t{} ({})", address - MemoryAddresses::TEMP_BOOL_START, address);
        }

        // Check if it's an integer constant
        if address >= MemoryAddresses::CTE_INT_START && address < MemoryAddresses::CTE_FLOAT_START {
            if let Some(value) = self.get_int_constant_value(address) {
                return format!("{} ({})", value, address);
            }
        }

        // Check if it's a float constant
        if address >= MemoryAddresses::CTE_FLOAT_START && address < MemoryAddresses::TEMP_INT_START {
            if let Some(value) = self.get_float_constant_value(address) {
                return format!("{} ({})", value, address);
            }
        }

        // If not found, return just the address
        format!("({})", address)
    }

    /// Clear all stacks, queues, and tables
    pub fn clear(&mut self) {
        self.p_oper.clear();
        self.pila_o.clear();
        self.p_types.clear();
        self.p_jumps.clear();  // Clear jumps stack
        self.quad_queue.clear();
        self.int_constants.clear();
        self.float_constants.clear();

        // Reset counters
        self.temp_int_counter = MemoryAddresses::TEMP_INT_START;
        self.temp_float_counter = MemoryAddresses::TEMP_FLOAT_START;
        self.temp_bool_counter = MemoryAddresses::TEMP_BOOL_START;
    }
}

