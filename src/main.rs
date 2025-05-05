use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub babyduck);

pub mod ast;
pub mod function_directory;

use function_directory::{FunctionDirectory, FunctionDirError};

#[test]
fn babyduck_basic_structure() {
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

    let result = babyduck::ProgramParser::new().parse(program);
    assert!(result.is_ok(), "Failed to parse basic program: {:?}", result.err());
    println!("Basic program structure test passed");
    println!("Parse result: {:#?}", result.unwrap());
}

#[test]
fn babyduck_control_structures() {
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

    let result = babyduck::ProgramParser::new().parse(program);
    assert!(result.is_ok(), "Failed to parse control structures: {:?}", result.err());
    println!("Control structures test passed");
    println!("Parse result: {:#?}", result.unwrap());
}

#[test]
fn babyduck_loops_and_arithmetic() {
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

    let result = babyduck::ProgramParser::new().parse(program);
    assert!(result.is_ok(), "Failed to parse loops and arithmetic: {:?}", result.err());
    println!("Loops and arithmetic test passed");
    println!("Parse result: {:#?}", result.unwrap());
}

#[test]
fn babyduck_complex_program() {
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

    let result = babyduck::ProgramParser::new().parse(program);
    assert!(result.is_ok(), "Failed to parse complex program: {:?}", result.err());
    println!("Complex program test passed");
    println!("Parse result: {:#?}", result.unwrap());
}

#[test]
fn create_function_directory() {
    // Updated to match the grammar definition - removing var declarations inside main body
    let program = r#"
    program example;
    var global_x: int;
    var global_y: float;
    
    void add(a: int, b: int) [
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

    let parse_result = babyduck::ProgramParser::new().parse(program);
    assert!(parse_result.is_ok(), "Failed to parse program: {:?}", parse_result.err());
    
    let ast = parse_result.unwrap();
    let function_directory_result = FunctionDirectory::from_program(&ast);
    assert!(function_directory_result.is_ok(), "Failed to create function directory: {:?}", function_directory_result.err());
    
    let function_directory = function_directory_result.unwrap();
    
    // Verify function directory contents
    assert!(function_directory.function_exists("global"));
    assert!(function_directory.function_exists("main"));
    assert!(function_directory.function_exists("add"));
    
    // Check variable types
    use ast::Type;
    match function_directory.get_variable_type("global", "global_x") {
        Some(Type::Int) => println!("global_x is an int as expected"),
        _ => panic!("global_x should be an int"),
    }
    
    match function_directory.get_variable_type("global", "global_y") {
        Some(Type::Float) => println!("global_y is a float as expected"),
        _ => panic!("global_y should be a float"),
    }
    
    // Check function parameters
    let add_func = function_directory.get_function("add").unwrap();
    assert_eq!(add_func.parameters.len(), 2);
    assert_eq!(add_func.parameters[0].0, "a");
    assert_eq!(add_func.parameters[1].0, "b");
    
    println!("Function directory test passed");
    println!("Function directory: {:#?}", function_directory);
}

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

fn main() {
    println!("BabyDuck Compiler");
    println!("Run the tests using 'cargo test' to verify the parser's functionality");
}

