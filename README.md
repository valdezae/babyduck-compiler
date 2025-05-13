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

![Diagrama sin titulo](Diagrama%20sin%20título.drawio.png)

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

Each of these actions is triggered at specific points in the parsing process, as indicated in the semantic action diagram. The coordination between these actions ensures that expressions are evaluated with the correct operator precedence and type checking.

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