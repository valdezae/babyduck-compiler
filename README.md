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