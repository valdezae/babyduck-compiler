# BabyDuck Compiler Project

This project implements a compiler for the BabyDuck language using Rust and LALRPOP as the parser generator.

## Analysis of Tools

### LALRPOP vs Chumsky

During the development of this compiler, multiple parser generation tools were evaluated:

#### LALRPOP
- **Pros:**
  - Well-documented with good examples
  - Generates Rust code that integrates seamlessly with the rest of the project
  - Familiar grammar syntax similar to BNF notation
  - Good error reporting capabilities
  - Supports context-free grammars which are suitable for most programming languages
  
- **Cons:**
  - Learning curve for more complex grammar features
  - Debugging grammar conflicts can be challenging
  - Limited flexibility when handling certain ambiguous constructs

#### Chumsky
- **Pros:**
  - Modern parser combinator library
  - Flexible API for complex parsing scenarios
  - Good error recovery mechanisms
  
- **Cons:**
  - Steeper learning curve
  - More complex to understand for straightforward grammar definition
  - Less intuitive for developers familiar with traditional grammar notation
  - Documentation is still evolving

After experimentation, LALRPOP was chosen for this project due to its more approachable grammar format and better documentation for newcomers to parser generators.

## Grammar Implementation

The BabyDuck language grammar was implemented in LALRPOP using a modular approach. LALRPOP uses a format similar to BNF notation, with productions and semantic actions.

### Lexical Analysis

The lexical tokens are defined using regular expressions:

```rust
match {
    r"\s*" => {},                     // Whitespace
    r"//[^\n\r]*[\n\r]*" => {},       // Line comments
    r"/\*([^*]|\*[^/])*\*/" => {},    // Block comments

    // Keywords
    "program" => PROGRAM,
    "var" => VAR,
    "int" => INT,
    "float" => FLOAT,
    // ...

    // Operators and symbols
    "=" => ASSIGN,
    "+" => PLUS,
    "-" => MINUS,
    // ...

    // Literals
    r"[0-9]+" => CTE_INT,
    r"[0-9]+\.[0-9]+" => CTE_FLOAT,
    r#""[^"]*""# => CTE_STRING,

    // Identifiers
    r"[a-zA-Z][a-zA-Z0-9_]*" => ID,
}
```

### Syntactic Analysis

The grammar rules define the structure of the BabyDuck language:

#### Program Structure

```rust
pub Program: Program = {
    PROGRAM <id:ID> SEMICOLON <decls:ProgramDecls> => {
        let (vars, funcs, body) = decls;
        Program {
            id: id.to_string(),
            vars,
            funcs,
            main_body: body,
        }
    }
};
```

#### Variable Declarations

Multiple `var` sections are supported by recursive rules:

```rust
VarSections: Vec<VarDeclaration> = {
    <v:VarSection> => v,
    <v:VarSection> <rest:VarSections> => {
        let mut result = v;
        result.extend(rest);
        result
    }
};

VarSection: Vec<VarDeclaration> = {
    VAR <decls:DeclaracionVar> => decls
};
```

#### Expressions with Correct Precedence

Expression precedence is handled through rule nesting (higher precedence deeper):

```rust
// Comparison operators (lowest precedence)
COMPARISON: Expression = {
    <left:EXP> GT <right:EXP> => Expression::BinaryOp {
        left: Box::new(left),
        operator: Operator::GreaterThan,
        right: Box::new(right),
    },
    // other comparison operators...
    <exp:EXP> => exp,
};

// Addition/subtraction
EXP: Expression = {
    <left:EXP> PLUS <right:TERMINO> => Expression::BinaryOp {
        left: Box::new(left),
        operator: Operator::Plus,
        right: Box::new(right),
    },
    // other operators...
    <term:TERMINO> => term,
};

// Multiplication/division (higher precedence)
TERMINO: Expression = {
    <left:TERMINO> MULTIPLY <right:FACTOR> => Expression::BinaryOp {
        left: Box::new(left),
        operator: Operator::Multiply,
        right: Box::new(right),
    },
    // other operators...
    <factor:FACTOR> => factor,
};

// Highest precedence (parentheses, literals, variables)
FACTOR: Expression = {
    LPAREN <expr:EXPRESION> RPAREN => expr,
    <id:ID> => Expression::Identifier(id.to_string()),
    <cte:CTE> => cte,
};
```

#### Control Structures

The grammar handles common control structures like if-else and while loops:

```rust
CONDITION: Condition = {
    IF LPAREN <expr:EXPRESION> RPAREN <if_body:Body> ELSE <else_body:Body> => {
        Condition {
            condition: expr,
            if_body,
            else_body: Some(else_body),
        }
    },
    // No-else version...
};

CYCLE: Cycle = {
    WHILE LPAREN <expr:EXPRESION> RPAREN DO <body:Body> SEMICOLON => {
        Cycle {
            condition: expr,
            body,
        }
    },
};
```

### Abstract Syntax Tree (AST)

The AST is represented using Rust structures and enums that mirror the language constructs:

```rust
pub struct Program {
    pub id: String,
    pub vars: Vec<VarDeclaration>,
    pub funcs: Vec<FunctionDeclaration>,
    pub main_body: Vec<Statement>,
}

pub enum Expression {
    BinaryOp {
        left: Box<Expression>,
        operator: Operator,
        right: Box<Expression>,
    },
    Identifier(String),
    IntegerLiteral(i32),
    FloatLiteral(f64),
}
```

## Challenges Encountered

During the implementation, several challenges were encountered:

1. **Multiple Variable Sections**: The grammar initially couldn't handle multiple `var` sections in the program. This was resolved by adding recursive rules to accumulate variable declarations.

2. **Operator Precedence**: Ensuring proper operator precedence required careful nesting of grammar rules from lowest to highest precedence.

3. **Dangling Else Problem**: The ambiguity in if-else statements was resolved by having specific rules for both if-with-else and if-without-else.

4. **Complex Expressions**: Supporting nested expressions with different operators required careful consideration of precedence and associativity.

## Test Cases

The following test cases were developed to validate the compiler's functionality: They can be run with `cargo test`.

### Basic Program Structure

```rust
let program = r#"
    program example;
    var x: int;
    var y: float;
    main {
        x = 10;
        y = 20.5;
    }
    end
"#;
```

### Control Structures

```rust
let program = r#"
    program example;
    var x: int;
    main {
        if (x > 5) {
            print("x is greater than 5");
        } else {
            print("x is not greater than 5");
        }
    }
    end
"#;
```

### Loops and Arithmetic

```rust
let program = r#"
    program example;
    var x: int;
    main {
        x = 0;
        while (x < 10) do {
            x = x + 1;
        };
        print(x);
    }
    end
"#;
```

### Complex Program

```rust
let program = r#"
    program example;
    var x: int;
    var y: float;
    main {
        x = 10;
        y = 20.5;
        if (x > 5) {
            print("x is greater than 5");
        } else {
            print("x is not greater than 5");
        }
        while (x < 100) do {
            x = x + 1;
        };
        print(x);
    }
    end
"#;
```

## Function Directory and Variable Tables

### Semantic Considerations Table (Semantic Cube)

The semantic considerations table (or semantic cube) for the BabyDuck language defines valid operations between different data types. This ensures type compatibility for operations like addition, comparison, and assignment.

| Operation | int, int | float, float | int, float | float, int | string, string | Other combinations |
|-----------|----------|--------------|------------|------------|----------------|--------------------|
| +         | int      | float        | float      | float      | Error          | Error              |
| -         | int      | float        | float      | float      | Error          | Error              |
| *         | int      | float        | float      | float      | Error          | Error              |
| /         | int      | float        | float      | float      | Error          | Error              |
| $>$       | bool     | bool         | bool       | bool       | Error          | Error              |
| <         | bool     | bool         | bool       | bool       | Error          | Error              |
| ==        | bool     | bool         | bool       | bool       | bool           | Error              |
| !=        | bool     | bool         | bool       | bool       | bool           | Error              |
| =         | Valid    | Valid        | Valid*     | Valid*     | Valid          | Error              |

*Type coercion may be applied in these cases

### Data Structures Implementation

#### Function Directory Structure

The Function Directory is implemented using a Rust HashMap that maps function names to their associated information:

```rust
pub struct FunctionDirectory {
    functions: HashMap<String, FunctionInfo>,
}

pub struct FunctionInfo {
    pub return_type: Option<Type>,
    pub parameters: Vec<(String, Type)>,
    pub local_variables: HashMap<String, Type>,
    pub is_program: bool,  // Flag to indicate if this is the program entry
}
```

This structure was chosen for the following reasons:
- **Fast Lookup**: HashMaps provide O(1) average-case lookup time, which is essential for frequent symbol resolution
- **Hierarchical Organization**: Each function has its own table of local variables
- **Flexible Structure**: Can easily accommodate additional function metadata as needed
- **Memory Efficiency**: Only stores what's needed for each function

#### Error Handling Structure

A custom error type was implemented to handle semantic errors during compilation:

```rust
pub enum FunctionDirError {
    DuplicateVariable(String, String), // (var_name, scope_name)
    DuplicateFunction(String),
    // Can add more error types as needed
}
```

This approach was chosen because:
- It provides detailed error information (variable name and scope)
- It's extensible for adding other error types in the future
- It integrates well with Rust's Result type for error propagation

### Key Implementation Points

#### Function Directory Creation

The function directory is created during the semantic analysis phase by traversing the AST:

```rust
pub fn from_program(program: &Program) -> Result<Self, FunctionDirError> {
    let mut directory = Self::new();
    
    // Add program as a special function entry
    // Add global variables to a special "global" entry
    // Add main function
    // Add all other functions
    
    Ok(directory)
}
```

#### Variable Declaration Validation

The compiler validates variable declarations to prevent duplicate variables:

1. **Global Scope**: Checks if a variable name already exists in the global scope
2. **Local Scope**: Checks if a variable name already exists in the function's local scope
3. **Parameter Conflict**: Ensures local variables don't conflict with parameter names

For example:
```rust
pub fn add_function(&mut self, func: &FunctionDeclaration) -> Result<(), FunctionDirError> {
    // Check for duplicate function name
    // Process parameters and check for duplicates
    // Process local variables and check for duplicates and conflicts
    // Store function information
    Ok(())
}
```

#### Variable Resolution

The compiler implements variable lookup with proper scope resolution:

```rust
pub fn get_variable_type(&self, function_name: &str, variable_name: &str) -> Option<&Type> {
    // Check if variable exists in the function's local scope
    // If not found and not in global scope, check global scope
    None
}
```

This approach prioritizes local variables over global variables with the same name, implementing proper variable shadowing.

### Testing and Validation

The function directory implementation is tested with various scenarios:
- Basic function and variable declarations
- Duplicate variable detection
- Scope handling between global and local variables
- Parameter processing and validation

For example:
```rust
#[test]
fn test_duplicate_variable_error() {
  // Test with duplicate global variables
  let program_with_duplicate = r#"
    program example;
    var global_x: int;
    var global_y: float;
    
    void add(a: int, b: int) [
        var result: int; 
        var result: int; 
        {
            result = a + b;
        }
        
    ];

    main {
        global_x = 10;
        global_y = 20.5;
    }
    end
    "#;

  let parse_result = babyduck::ProgramParser::new().parse(program_with_duplicate);
  assert!(parse_result.is_ok(), "Failed to parse program: {:?}", parse_result.err());

  let ast = parse_result.unwrap();
  let function_directory_result = FunctionDirectory::from_program(&ast);
  println!("Function directory result: {:?}", &function_directory_result);
  // Should fail with duplicate variable error
  assert!(function_directory_result.is_err());

  if let Err(FunctionDirError::DuplicateVariable(var_name, scope)) = function_directory_result {
    println!("Correctly detected duplicate variable '{}' in scope '{}'", var_name, scope);
    assert_eq!(var_name, "result");
    assert_eq!(scope, "add");
  } else {
    panic!("Expected DuplicateVariable error for global variable");
  }
}
```

These tests ensure that the function directory correctly handles the symbolic information of the BabyDuck language and provides appropriate error messages for semantic issues.

## Intermediate Code Generation

The compiler generates intermediate code in the form of quadruples, which represent operations in a format that's closer to machine language while still being abstract enough to be machine-independent.

### Data Structures for Quadruple Generation

The quadruple generator uses several important data structures to manage the compilation process:

#### Stacks

1. **p_oper (Operator Stack)**
    - Stores operators during expression parsing
    - Used to manage operator precedence
    - Implemented as a Rust `Vec<i32>` with push/pop operations
    - Operators are stored as integer codes for efficiency
    - Each operator is pushed when encountered during expression parsing
    - Operators are popped and processed when their precedence is lower than or equal to the operator at the top of the stack

2. **pila_o (Operand Stack)**
    - Stores memory addresses of operands (variables, constants, temporaries)
    - Implemented as a Rust `Vec<i32>`
    - Addresses pointing to the memory location of each value
    - Used to track operands during expression evaluation
    - When an expression is processed, operands are pushed onto this stack
    - When an operation is performed, operands are popped, and the result address is pushed back

3. **p_types (Type Stack)**
    - Parallel to the operand stack, stores the type of each operand
    - Implemented as a Rust `Vec<Type>`
    - Used for type checking and semantic validation
    - Each type is pushed when an operand is pushed onto pila_o
    - Types are used for semantic checking to determine the result type of operations

#### Queues

1. **quad_queue (Quadruple Queue)**
    - Stores generated quadruples
    - Implemented as a Rust `VecDeque<Quadruple>`
    - Each quadruple represents a single operation in the intermediate code
    - Contains operation code, operand addresses, and result address
    - Quadruples are consumed by the virtual machine during execution

### Key Semantic Actions in Quadruple Generation

The compiler uses several neuralgic points in expression parsing to generate quadruples:

![Diagrama sin título.drawio (2).png](Diagrama%20sin%20t%C3%ADtulo.drawio%20%282%29.png)

1. **Action 1: Push Operand**
    - **Location**: When encountering identifiers or constants in factors
    - **Action**: Pushes the operand's memory address to pila_o and its type to p_types
    - **Implementation**: `action_push_id()` for variables and `action_push_constant()` for literals
    - **Purpose**: Prepares operands for future operations

2. **Action 2: Push Multiplication/Division Operator**
    - **Location**: When encountering * or / operators in term expressions
    - **Action**: Pushes the operator code to p_oper stack
    - **Implementation**: `action_push_mult_div_oper()`
    - **Purpose**: Records high-precedence operations for processing

3. **Action 3: Push Addition/Subtraction Operator**
    - **Location**: When encountering + or - operators in expressions
    - **Action**: Pushes the operator code to p_oper stack
    - **Implementation**: `action_push_add_sub_oper()`
    - **Purpose**: Records lower-precedence operations for processing

4. **Action 4: Process Addition/Subtraction Operations**
    - **Location**: After processing a term, checks if + or - operations should be resolved
    - **Action**:
        - Checks if top operator in p_oper is + or -
        - If yes, pops operands and types from respective stacks
        - Performs type checking via semantics function
        - Generates quadruple for the operation
        - Pushes result address and type back to stacks
    - **Implementation**: `action_process_operation(false)`
    - **Purpose**: Maintains operator precedence by processing + and - operations at appropriate times

5. **Action 5: Process Multiplication/Division Operations**
    - **Location**: After processing a factor, checks if * or / operations should be resolved
    - **Action**:
        - Similar to Action 4, but checks for * or / operators
        - Pops operands and generates quadruples with higher precedence
        - Uses temporary variables to store intermediate results
    - **Implementation**: `action_process_operation(true)`
    - **Purpose**: Ensures higher-precedence operations (* and /) are processed before lower-precedence ones (+ and -)

6. **Action 6: Process Condition Expression**
    - **Location**: In CONDITION block after the if token, at the expression evaluation point between the parentheses
    - **Action**: Evaluates the boolean expression that determines flow control
    - **Implementation**: `self.process_expression(&cond.condition)`
    - **Purpose**: Determines whether to execute the if-block or loop body

7. **Action 7: Generate GOTOF (GoTo False) Quadruple**
    - **Location**: After the right parenthesis in CONDITION block, before the Body
    - **Action**: Creates a conditional jump with a placeholder destination
    - **Implementation**: `self.quad_queue.push_back(Quadruple::new(OpCode::GOTOF, result_addr, -1, -1))`
    - **Purpose**: Bypasses the if-body when condition is false

8. **Action 8: Save Jump Position**
    - **Location**: Same point as Action 7, in the transition between the condition and Body
    - **Action**: Pushes the quadruple index to the jumps stack for later backpatching
    - **Implementation**: `self.p_jumps.push(gotof_quad_idx)`
    - **Purpose**: Remembers the position to update with correct jump destination

9. **Action 9: Process Body Statements**
    - **Location**: Inside the Body box of the CONDITION or CYCLE diagram sections
    - **Action**: Processes all statements in the if-body or loop body
    - **Implementation**: `self.generate_from_statements(&cond.if_body)` or `self.generate_from_statements(&cycle.body)`
    - **Purpose**: Generates quadruples for the statements to be executed when condition is true

10. **Action 10: Generate Unconditional GOTO (For If-Else Only)**
    - **Location**: At the arrow leading from Body to the else token in CONDITION block
    - **Action**: Creates a GOTO to skip the else block after if-body completes
    - **Implementation**: `self.quad_queue.push_back(Quadruple::new(OpCode::GOTO, -1, -1, -1))`
    - **Purpose**: Ensures if-body and else-body are mutually exclusive

11. **Action 11: Backpatch GOTOF with Current Position**
    - **Location**: At the arrow entering the else token or at the final semicolon if no else
    - **Action**: Updates the previously created GOTOF with the correct jump target
    - **Implementation**: `self.fill_jump(gotof_quad_idx, jump_target as i32)`
    - **Purpose**: Completes the conditional jump to properly bypass code blocks

12. **Action 12: Generate Loop-Back GOTO (For Cycle Only)**
    - **Location**: At the arrow returning from the Body to the while condition in CYCLE block
    - **Action**: Creates an unconditional jump back to condition evaluation
    - **Implementation**: `self.quad_queue.push_back(Quadruple::new(OpCode::GOTO, -1, -1, return_pos as i32))`
    - **Purpose**: Implements the repetitive nature of loops by returning to condition check

13. **Action 13: Fill Final Jump Destination**
    - **Location**: At the final semicolon in CONDITION or CYCLE blocks
    - **Action**: Backpatches any remaining jumps with the current position
    - **Implementation**: `self.fill_jump(jump_pos, jump_target as i32)`
    - **Purpose**:  Ensures all control flow paths converge at the correct point after the structure

### Example of Generated Quadruples

Below is an example of quadruples generated for a complex expression:

```
R = ((A + B) * C + D * E * F + K / H * J) + G * L + H + J > (A - C * D) / F;
print(R);
```

This expression is parsed and converted to the following intermediate code:

```
Generated quadruples:
===================================
No. | Operation | Arg1 | Arg2 | Result
-----------------------------------
  0 | +         | 1000 | 1001 | 4000  
  1 | *         | 4000 | 1002 | 4001  
  2 | *         | 1003 | 1004 | 4002  
  3 | *         | 4002 | 1005 | 4003  
  4 | +         | 4001 | 4003 | 4004  
  5 | /         | 1010 | 1007 | 4005  
  6 | *         | 4005 | 1009 | 4006  
  7 | +         | 4004 | 4006 | 4007  
  8 | *         | 1006 | 1011 | 4008  
  9 | +         | 4007 | 4008 | 4009  
 10 | +         | 4009 | 1007 | 4010  
 11 | +         | 4010 | 1009 | 4011  
 12 | *         | 1002 | 1003 | 4012  
 13 | -         | 1000 | 4012 | 4013  
 14 | /         | 4013 | 1005 | 4014  
 15 | >         | 4011 | 4014 | 6000  
 16 | =         | 6000 | -1   | 1012  
 17 | PRINT     | 1012 | -1   | -1    
===================================
```

#### Memory Addressing Scheme

In the quadruple system, different memory address ranges are used for different types of data:

- **1000-1999**: Global and local integer variables
- **2000-2999**: Float variables (both global and local)
- **3000-3499**: Integer constants (literals stored in the constant table)
- **3500-3999**: Float constants (literals stored in the constant table)
- **4000-4999**: Integer temporaries (intermediate results)
- **5000-5999**: Float temporaries (intermediate results)
- **6000-6999**: Boolean temporaries (results of comparison operations)

The variable addresses for this example are:

```
Variable Addresses:
===================================
Variable | Address
-----------------------------------
L        |   1011
H        |   1007
R        |   1012
K        |   1010
B        |   1001
J        |   1009
A        |   1000
E        |   1004
C        |   1002
F        |   1005
D        |   1003
G        |   1006
I        |   1008
===================================
```

#### Execution Flow Explanation

The quadruples above show the step-by-step calculation of the expression:

1. First, `A + B` is calculated and stored in temp `4000` (quad 0)
2. The result is multiplied by `C` and stored in temp `4001` (quad 1)
3. In parallel, `D * E` is calculated and stored in temp `4002` (quad 2)
4. That result is multiplied by `F` and stored in temp `4003` (quad 3)
5. The results from steps 2 and 4 are added and stored in temp `4004` (quad 4)
6. The division `K / H` is calculated and stored in temp `4005` (quad 5)
7. That result is multiplied by `J` and stored in temp `4006` (quad 6)
8. The results from steps 5 and 7 are added and stored in temp `4007` (quad 7)
9. In parallel, `G * L` is calculated and stored in temp `4008` (quad 8)
10. The results from steps 8 and 9 are added and stored in temp `4009` (quad 9)
11. Add `H` to obtain temp `4010` (quad 10)
12. Add `J` to obtain temp `4011` (quad 11), which completes the left side of the `>` operation
13. For the right side, first `C * D` is calculated and stored in temp `4012` (quad 12)
14. Then `A - 4012` is calculated and stored in temp `4013` (quad 13)
15. Finally, `4013 / F` is calculated and stored in temp `4014` (quad 14)
16. The comparison `4011 > 4014` is performed and stored in the boolean temp `6000` (quad 15)
17. The boolean result is assigned to variable `R` (quad 16)
18. The value of `R` is printed (quad 17)

This demonstrates how a complex expression is broken down into simple operations following operator precedence rules, and how intermediate values are stored and reused in temporary variables.

Okay, here's the English translation of the "Máquina Virtual de BabyDuck" and "Memoria de Ejecución en la VM" sections:

## BabyDuck Virtual Machine

The BabyDuck Virtual Machine (VM) is responsible for executing the intermediate code (quadruples) generated by the compiler. It is designed with several key data structures to manage program execution:

### Main VM Data Structures

*   **`Quad` (Quadruple in the VM):**
    ```rust
    struct Quad {
        op: i32,     // Operation code
        arg1: i32,   // Address of the first argument (or special value)
        arg2: i32,   // Address of the second argument (or special value)
        result: i32, // Address where the result is stored (or jump target)
    }
    ```
    This structure represents a single instruction for the VM. The `arg1`, `arg2`, and `result` fields generally contain memory addresses but can have special meanings for certain operations (e.g., jumps, immediate boolean values).

*   **`VMValue` (Value in the VM):**
    ```rust
    enum VMValue {
        Int(i32),
        Float(f64),
        Bool(bool),
    }
    ```
    Represents the data types that the VM can handle and store in its memory.

*   **`VMFunctionInfo` (Function Information in the VM):**
    ```rust
    struct VMFunctionInfo {
        name: String,            // Function name
        param_count: usize,      // Number of parameters
        param_addresses: Vec<i32>, // Memory addresses for parameters
    }
    ```
    Stores metadata about each function loaded from the object file, crucial for managing function calls.

*   **`VM` Structure (Core Virtual Machine Structure):**
    Contains all components necessary for execution:
    *   `quads: Vec<Quad>`: A vector storing all loaded program quadruples.
    *   `ip: usize`: The Instruction Pointer, indicating the index of the current quadruple to execute in the `quads` vector.
    *   `call_stack: Vec<usize>`: A stack storing return addresses (IPs) for function calls (`GOSUB` and `ENDFUNC` operations).
    *   `functions: HashMap<i32, VMFunctionInfo>`: A map associating a function's starting quadruple index with its `VMFunctionInfo`. Allows the VM to find function information during an `ERA` or `GOSUB` call.
    *   `staged_params: Vec<VMValue>`: A temporary vector storing parameter values (obtained via the `PARAM` operation) before a function call (`GOSUB`) is made.
    *   `int_memory: Vec<Option<i32>>`, `float_memory: Vec<Option<f64>>`, `bool_memory: Vec<Option<bool>>`: Vectors constituting the Execution Memory (see next section).
    *   `max_..._addr`: Internal variables to track the highest address used in each memory segment, used to dynamically size memory vectors during `.obj` file loading.

## Execution Memory in the VM

Execution Memory is where the VM stores all data during the execution of a BabyDuck program. This includes global and local variables, constants, and temporary values generated during expression evaluation.

### Memory Structure

The memory is organized into three main vectors, one for each base data type:

*   `int_memory: Vec<Option<i32>>`: Stores all integer values.
*   `float_memory: Vec<Option<f64>>`: Stores all floating-point values.
*   `bool_memory: Vec<Option<bool>>`: Stores all boolean values.

These vectors are initialized empty and are resized once (`resize_memory()`) after an initial scan of the object file (`.obj`) to determine the maximum required size for each data type and segment.

#### Memory Segmentation (Conceptual Version)

Within each type-specific vector, memory is conceptually segmented according to the address ranges defined by the compiler (see `MemoryAddresses` in `quadruples.rs`). The VM maps these addresses to indices within its vectors. The mapping structure is as follows:

| VM Vector      | Data Type | Memory Segment (Origin)            | Address Range (Example) | Mapping to Vector Index (Conceptual)                                        |
|----------------|-------------|------------------------------------|---------------------------|-----------------------------------------------------------------------------|
| `int_memory`   | Integer     | Variables (Global/Local)           | `INT_START` (1000+)       | `idx = address - INT_START`                                                 |
|                |             | Integer Constants                  | `CTE_INT_START` (4000+)     | `idx = (address - CTE_INT_START) + local_int_offset`                      |
|                |             | Integer Temporaries                | `TEMP_INT_START` (5000+)    | `idx = (address - TEMP_INT_START) + local_int_offset + const_int_offset`  |
| `float_memory` | Float       | Variables (Global/Local)           | `FLOAT_START` (2000+)     | `idx = address - FLOAT_START`                                               |
|                |             | Float Constants                    | `CTE_FLOAT_START` (4500+)   | `idx = (address - CTE_FLOAT_START) + local_float_offset`                  |
|                |             | Float Temporaries                  | `TEMP_FLOAT_START` (6000+)  | `idx = (address - TEMP_FLOAT_START) + local_float_offset + const_float_offset`|
| `bool_memory`  | Boolean     | Variables (Global/Local)           | `BOOL_START` (3000+)      | `idx = address - BOOL_START`                                                |
|                |             | (Boolean Constants via Temporaries)| (`TEMP_BOOL_START` range) | (Treated as temporaries by the compiler, stored in the boolean temporary segment by the VM) |
|                |             | Boolean Temporaries                | `TEMP_BOOL_START` (7000+)   | `idx = (address - TEMP_BOOL_START) + local_bool_offset (+ const_bool_offset_if_exists)` |

*Note: `..._offset` represents the size of the preceding segment within the same vector.* For example, in `int_memory`, integer constants are stored after all local/global integer variables. Memory addresses (`MemoryAddresses` like `INT_START`, `CTE_INT_START`, `TEMP_INT_START`) are global across all types, but the VM segregates them into their respective vectors (`int_memory`, `float_memory`, `bool_memory`) and then concatenates the segments (local, constant, temporary) within those vectors.

### Main Memory Access Methods

The VM uses the following methods to interact with its memory system:

*   **`update_max_address(address: i32)`**:
    During object file loading, this method is called for each address found. It tracks the highest address for each memory segment (local variables, constants, temporaries for each type).

*   **`resize_memory()`**:
    Called after all addresses have been processed by `update_max_address`. It resizes the `int_memory`, `float_memory`, and `bool_memory` vectors to be large enough to hold all necessary data, initializing positions with `None`.

*   **`get_int_idx(address: i32) -> Result<usize, String>`** (and analogues `get_float_idx`, `get_bool_idx`):
    These internal methods convert a global memory address (e.g., 1005, 4002, 5001) into a specific index for the corresponding vector (`int_memory`, `float_memory`, or `bool_memory`). They consider the different segments (local, constant, temporary) to calculate the correct index. For example, an address in `TEMP_INT_START` will map to an index in `int_memory` that comes after all spaces reserved for `INT_START` and `CTE_INT_START`.

*   **`get_value(address: i32) -> Result<VMValue, String>`**:
    Receives a memory address and returns the `VMValue` stored at that location. Internally, it determines which memory type the address belongs to (int, float, bool) and then uses the `get_..._idx` methods to find the correct vector and index. It returns an error if the address is invalid or memory has not been initialized at that position.

*   **`set_value(address: i32, value: VMValue) -> Result<(), String>`**:
    Receives a memory address and a `VMValue`, and writes the value to the corresponding location. Similar to `get_value`, it uses the `get_..._idx` methods. It can perform implicit type conversions (e.g., assigning a `VMValue::Int` to a float address, promoting the integer to float, or a boolean to an integer as 0 or 1, depending on the target address type).

These methods ensure that the VM can access and manipulate data efficiently and correctly according to the addressing scheme defined by the compiler.

### Complete BabyDuck Example: test.bd

Below is a comprehensive example showing most BabyDuck language features:

```
program ComplexTestProgram;

// Test multiple variable sections with different declaration styles
var
    x, y, z : int;
    temperature, humidity : float;

var
    flag, isReady, done : bool;

var
    counter : int;
    result : float;

// Test functions with different parameter configurations
void printTwo(first : int, second : int) [
    var
        sum : int;
    {
        sum = first + second;
        print(first);
        print(second);
        print(sum);
    }
];

void calculate(base : float, multiplier : int) [
    var
        temp1 : float;
        temp2 : float;
        check : bool;
    {
        temp1 = base * multiplier;
        temp2 = temp1 + 10.5;
        check = temp2 > 50.0;
        print(temp1);
        print(temp2);
        print(check);
    }
];

void noParams() [
    {
        print(100);
        print(3.14);
    }
];

main {
    // Test basic assignments with all types
    x = 15;
    y = 25;
    z = 35;
    temperature = 22.5;
    humidity = 65.8;
    flag = true;
    isReady = false;
    done = true;

    // Test arithmetic expressions with proper precedence
    counter = x + y * 2;        // Should be 65 (25*2=50, 15+50=65)
    result = temperature * 2.0 + 5.5;  // Should be 50.5

    // Test parentheses for precedence override
    z = (x + y) * 2;           // Should be 80 ((15+25)*2)
    humidity = (temperature + 5.0) * 1.5; // Should be 41.25

    // Test all comparison operators
    flag = x > y;              // false (15 > 25)
    isReady = y < 30;          // true (25 < 30)
    done = z == 80;            // true
    flag = counter != 65;      // false

    // Test print with variables and literals
    print(x);
    print(temperature);
    print(flag);
    print(42);
    print(3.14159);

    // Test print with expressions
    print(x + y);
    print(temperature - 2.5);
    print(x * y);
    print(humidity / 2.0);
    print(x > 10);
    print(temperature == 22.5);

    // Test nested if statements
    if (x < y) {
        print(1);
        counter = counter + 10;
        if (counter > 70) {
            print(2);
            flag = true;
        } else {
            print(3);
            flag = false;
        }
    } else {
        print(4);
        counter = counter - 5;
    }

    // Test if without else
    if (temperature > 20.0) {
        humidity = humidity + 5.0;
        print(humidity);
    }

    // Test while loops with different conditions
    counter = 1;
    while (counter < 5) do {
        print(counter);
        counter = counter + 1;
    };

    // Test while with float condition
    result = 10.0;
    while (result > 5.0) do {
        print(result);
        result = result - 1.5;
    };

    // Test nested while loops
    x = 1;
    while (x < 3) do {
        y = 1;
        while (y < 2) do {
            z = x * y;
            print(z);
            y = y + 1;
        };
        x = x + 1;
    };

    // Test function calls with different argument patterns
    printTwo(10, 20);
    printTwo(x, y);
    printTwo(x + 5, y - 3);

    calculate(25.5, 2);
    calculate(temperature, 3);
    calculate(humidity / 2.0, counter);

    noParams();

    // Test more complex expressions
    x = y + z - counter;
    temperature = humidity * 0.5 + 15.0;
    flag = x > y + 10;
    isReady = temperature < humidity - 5.0;

    // Test complex function call arguments
    printTwo(x * 2, y + z);
    calculate(temperature + 5.0, x - y);

    // Final complex conditional
    if (flag == true) {
        while (x > 0) do {
            print(x);
            x = x - 10;
            if (x < 5) {
                done = true;
                print(done);
            }
        };
    } else {
        counter = 0;
        while (counter < 3) do {
            print(counter * 2);
            counter = counter + 1;
        };
    }

    // Final outputs
    print(x);
    print(temperature);
    print(flag);
    print(done);
}
end
```

This example demonstrates:

1. Multiple variable declaration sections with different types (int, float, bool)
2. Multiple function declarations with different parameter configurations
3. Expression evaluation with proper operator precedence
4. Complex arithmetic expressions
5. Conditional statements (if-else) including nested conditionals
6. While loops including nested loops
7. Function calls with various argument patterns
8. Boolean operations and comparisons
9. Complex control flow combinations

Running this program through the BabyDuck compiler will test all phases of compilation including lexical analysis, syntax analysis, semantic analysis, and code generation.