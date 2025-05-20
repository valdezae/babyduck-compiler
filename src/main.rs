use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub babyduck);

pub mod ast;
pub mod function_directory;
pub mod quadruples;

use function_directory::{FunctionDirectory, FunctionDirError};
use quadruples::QuadrupleGenerator;

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

#[test]
fn babyduck_boolean_operations() {
    let program = r#"
    program example;
    var x: bool;
    var y: int;
    main {
        y = 10;
        x = true;
        x = y > 5;
        if (x == true) {
            print("Condition is true");
        } else {
            print("Condition is false");
        }
    }
    end
    "#;

    let result = babyduck::ProgramParser::new().parse(program);
    assert!(result.is_ok(), "Failed to parse boolean operations: {:?}", result.err());
    println!("Boolean operations test passed");
    println!("Parse result: {:#?}", result.unwrap());
}

#[test]
fn test_quadruple_generation() {
    // Create a simple program with arithmetic operations
    let program = r#"
    program example;
    var A, B, C, D, E, F, G, H, I, J, K, L, R: int;
    main {
       R = ((A + B) * C + D * E * F + K / H * J) + G * L + H + J > (A - C * D) / F;
       print(R);
    }
    end
    "#;

    let parse_result = babyduck::ProgramParser::new().parse(program);
    assert!(parse_result.is_ok(), "Failed to parse program: {:?}", parse_result.err());
    
    let ast = parse_result.unwrap();
    
    // Create function directory for variable lookups
    let function_directory_result = FunctionDirectory::from_program(&ast);
    assert!(function_directory_result.is_ok(), "Failed to create function directory: {:?}", function_directory_result.err());
    let function_directory = function_directory_result.unwrap();
    
    // Create quadruple generator
    let mut quad_gen = QuadrupleGenerator::new();
    quad_gen.set_function_directory(function_directory);
    quad_gen.set_current_scope("main");
    
    // Generate quadruples from the AST's main statements
    quad_gen.generate_from_statements(&ast.main_body);
    
    println!("\n=== QUADRUPLES GENERATED FOR TEST PROGRAM ===");
    println!("Source code:");
    println!("```");
    println!("{}", program);
    println!("```\n");
    
    // Get the generated quadruples and print them
    let quadruples = quad_gen.get_quadruples_as_strings();
    println!("Generated quadruples:");
    println!("===================================");
    println!("No. | Operation | Arg1 | Arg2 | Result");
    println!("-----------------------------------");
    
    for (i, quad) in quadruples.iter().enumerate() {
        // Strip the outer parentheses if they exist
        let quad_str = if quad.starts_with('(') && quad.ends_with(')') {
            &quad[1..quad.len()-1]
        } else {
            quad
        };
        
        // Split by commas and print in a tabular format
        let parts: Vec<&str> = quad_str.split(',').map(|s| s.trim()).collect();
        if parts.len() >= 4 {
            println!("{:3} | {:9} | {:4} | {:4} | {:6}", i, parts[0], parts[1], parts[2], parts[3]);
        } else {
            println!("{:3} | {}", i, quad);
        }
    }
    println!("===================================");
    
    // Print variable addresses for better understanding
    println!("\nVariable Addresses:");
    println!("===================================");
    println!("Variable | Address");
    println!("-----------------------------------");
    for (name, addr) in quad_gen.get_variables() {
        println!("{:8} | {:6}", name, addr);
    }
    println!("===================================");
    
    // Print constant tables for debugging
    println!("\nInteger Constants:");
    println!("===================================");
    println!("Value | Address");
    println!("-----------------------------------");
    for (value, addr) in quad_gen.get_int_constants() {
        println!("{:5} | {:6}", value, addr);
    }
    println!("===================================");
    
    println!("\nFloat Constants:");
    println!("===================================");
    println!("Value | Address");
    println!("-----------------------------------");
    for (value, addr) in quad_gen.get_float_constants() {
        println!("{:5.1} | {:6}", value, addr);
    }
    println!("===================================");
    
    // Assert that quadruples were generated
    assert!(!quadruples.is_empty(), "No quadruples were generated");
    
    // Check that we have a sufficient number of quadruples for this complex expression
    assert!(quadruples.len() >= 15, "Expected at least 15 quadruples for this complex expression, got {}", quadruples.len());
    
    // Validate that we have comparison operation (>)
    let has_comparison = quadruples.iter().any(|q| q.contains(">"));
    assert!(has_comparison, "No '>' comparison quadruple found");
    
    // Check that all variables are properly registered
    let vars = quad_gen.get_variables();
    let expected_vars = ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "R"];
    for var_name in expected_vars.iter() {
        let var_exists = vars.iter().any(|(name, _)| name == var_name);
        assert!(var_exists, "Variable '{}' not found in variables table", var_name);
    }
    
    println!("\nQuadruple generation test passed");
}

#[test]
fn test_control_flow_quadruple_generation() {
    // Create a program with conditional statements and loops
    let program = r#"
    program control_flow_test;
    var x, y, result, counter: int;
    var flag: bool;
    main {
        // Simple if-else statement
        if (x > y) {
            result = x;
        } else {
            result = y;
        }
        
        // Nested if-else
        if (x == 5) {
            if (y == 10) {
                result = 15;
            } else {
                result = 5;
            }
        } else {
            result = 0;
        }
        
        // Simple while loop
        counter = 0;
        while (counter < 5) do {
            counter = counter + 1;
        };
        
        // While loop with conditional inside
        counter = 0;
        while (counter < 10) do {
            counter = counter + 1;
            if (counter == 5) {
                result = 100;
            }
        };
        
        // Bool variable in condition
        flag = x > y;
        if (flag) {
            result = 1;
        } else {
            result = 0;
        }
    }
    end
    "#;

    let parse_result = babyduck::ProgramParser::new().parse(program);
    assert!(parse_result.is_ok(), "Failed to parse program: {:?}", parse_result.err());
    
    let ast = parse_result.unwrap();
    
    // Create function directory for variable lookups
    let function_directory_result = FunctionDirectory::from_program(&ast);
    assert!(function_directory_result.is_ok(), "Failed to create function directory: {:?}", function_directory_result.err());
    let function_directory = function_directory_result.unwrap();
    println!("{:#?}", function_directory);
    
    // Create quadruple generator
    let mut quad_gen = QuadrupleGenerator::new();
    quad_gen.set_function_directory(function_directory);
    quad_gen.set_current_scope("main");
    
    // Generate quadruples from the AST's main statements
    quad_gen.generate_from_statements(&ast.main_body);
    
    println!("\n=== QUADRUPLES VECTOR FOR CONTROL FLOW TEST ===");

    println!("\n=== QUADRUPLES GENERATED FOR CONTROL FLOW TEST ===");
    // Get the generated quadruples and print them in tabular format
    let quadruples = quad_gen.get_quadruples_as_strings();
    
    println!("Generated quadruples:");
    println!("===================================");
    println!("No. | Operation | Arg1 | Arg2 | Result");
    println!("-----------------------------------");

    for (i, quad) in quadruples.iter().enumerate() {
        // Strip the outer parentheses if they exist
        let quad_str = if quad.starts_with('(') && quad.ends_with(')') {
            &quad[1..quad.len()-1]
        } else {
            quad
        };

        // Split by commas and print in a tabular format
        let parts: Vec<&str> = quad_str.split(',').map(|s| s.trim()).collect();
        if parts.len() >= 4 {
            println!("{:3} | {:9} | {:4} | {:4} | {:6}", i, parts[0], parts[1], parts[2], parts[3]);
        } else {
            println!("{:3} | {}", i, quad);
        }
    }
    println!("===================================");
    
    // Verify the quadruples
    
    // 1. Check for GOTO and GOTOF operations which are essential for control flow
    let goto_count = quadruples.iter()
                               .filter(|q| q.contains("GOTO"))
                               .count();
    let gotof_count = quadruples.iter()
                                .filter(|q| q.contains("GOTOF"))
                                .count();
                                
    println!("\nFound {} GOTO and {} GOTOF operations", goto_count, gotof_count);
    assert!(goto_count >= 4, "Expected at least 4 GOTO operations for control structures");
    assert!(gotof_count >= 5, "Expected at least 5 GOTOF operations for control structures");
    
    // 2. Check for comparison operations
    let comparison_ops = quadruples.iter()
                                   .filter(|q| q.contains(">") || 
                                              q.contains("<") ||
                                              q.contains("=="))
                                   .count();
    println!("Found {} comparison operations", comparison_ops);
    assert!(comparison_ops >= 5, "Expected at least 5 comparison operations for control structures");
    
    // 3. Check for assignments in both if and else branches
    let assign_ops = quadruples.iter()
                               .filter(|q| q.contains("="))
                               .count();
    println!("Found {} assignment operations", assign_ops);
    assert!(assign_ops >= 8, "Expected at least 8 assignment operations");
    
    // 4. Check for addition operations (counter increments)
    let add_ops = quadruples.iter()
                            .filter(|q| q.contains("+"))
                            .count();
    println!("Found {} addition operations", add_ops);
    assert!(add_ops >= 2, "Expected at least 2 addition operations for counter increments");
    
    // Print variable addresses for reference
    println!("\nVariable Addresses:");
    println!("===================================");
    println!("Variable | Address");
    println!("-----------------------------------");
    for (name, addr) in quad_gen.get_variables() {
        println!("{:8} | {:6}", name, addr);
    }
    println!("===================================");
    
    println!("\nControl flow quadruple generation test passed successfully!");
}

fn main() {
    println!("BabyDuck Compiler");
    println!("Run the tests using 'cargo test' to verify the parser's functionality");
}

