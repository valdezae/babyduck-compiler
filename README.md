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

The following test cases were developed to validate the compiler's functionality:

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
