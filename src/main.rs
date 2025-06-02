use lalrpop_util::lalrpop_mod;
use std::env;
use std::fs;
use std::io::{Write, BufWriter}; // For writing to file
use std::path::Path;

lalrpop_mod!(pub babyduck);

pub mod ast;
pub mod function_directory;
pub mod quadruples;
mod vm;

use function_directory::{FunctionDirectory, FunctionDirError};
use quadruples::{QuadrupleGenerator, OpCode};


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: babyduck_compiler <input_file.bd>");
        std::process::exit(1);
    }

    let input_filename = &args[1];
    let source_code = match fs::read_to_string(input_filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", input_filename, e);
            std::process::exit(1);
        }
    };

    println!("Compiling: {}", input_filename);

    // 1. Parse the source code
    let ast_result = babyduck::ProgramParser::new().parse(&source_code);
    let ast = match ast_result {
        Ok(program_ast) => {
            println!("Parsing successful!");
            program_ast
        }
        Err(e) => {
            eprintln!("Parsing failed: {:?}", e);
            std::process::exit(1);
        }
    };

    // 2. Create Function Directory
    let function_directory_result = FunctionDirectory::from_program(&ast);
    let function_directory = match function_directory_result {
        Ok(dir) => {
            println!("Function Directory created successfully!");
            dir
        }
        Err(e) => {
            eprintln!("Failed to create function directory: {}", e);
            std::process::exit(1);
        }
    };

    // 3. Generate Quadruples
    let mut quad_gen = QuadrupleGenerator::new();
    quad_gen.set_function_directory(function_directory.clone());

    let quad_gen_result = quad_gen.generate_for_program(&ast);
    if let Err(e) = quad_gen_result {
        eprintln!("Quadruple generation failed: {}", e);
        std::process::exit(1);
    }
    println!("Quadruple generation successful!");

    // 4. Prepare .obj file content
    let mut obj_content = String::new();
    obj_content.push_str("// BabyDuck Object File\n");
    obj_content.push_str(&format!("// Source: {}\n\n", input_filename));

    // Integer Constants
    obj_content.push_str("CONSTANTS_INT:\n");
    for (value, addr) in quad_gen.get_int_constants() {
        obj_content.push_str(&format!("{},{}\n", value, addr));
    }
    obj_content.push_str("END_CONSTANTS_INT\n\n");

    // Float Constants
    obj_content.push_str("CONSTANTS_FLOAT:\n");
    for (value, addr) in quad_gen.get_float_constants() {
        obj_content.push_str(&format!("{},{}\n", value, addr));
    }
    obj_content.push_str("END_CONSTANTS_FLOAT\n\n");
    
    // Boolean Constants are no longer saved to the obj file

    // Functions
    obj_content.push_str("FUNCTIONS:\n");
    if let Some(final_function_directory) = &quad_gen.function_directory {
        for (name, info) in final_function_directory.get_all_functions() {
            // Skip "global" scope and the program's own name entry, as they aren't callable functions.
            // "main" is the entry point and is included.
            if name == "global" || name == &ast.id { continue; }
            let start_idx = info.start_quad_idx.unwrap_or(-1);
            let param_count = info.parameters.len();
            let local_var_count = info.local_variables.len(); // This counts distinct local variable declarations.

            let mut param_addrs_str = String::new();
            for (_, _, addr) in &info.parameters { // Iterate over (param_name, param_type, param_address)
                param_addrs_str.push_str(&format!(",{}", addr));
            }
            // New format: name,start_idx,param_count,local_var_count[,param1_addr][,param2_addr]...
            obj_content.push_str(&format!("{},{},{},{}{}\n", name, start_idx, param_count, local_var_count, param_addrs_str));
        }
    }
    obj_content.push_str("END_FUNCTIONS\n\n");


    // Quadruples (Machine-readable format)
    obj_content.push_str("QUADRUPLES:\n");
    for quad in quad_gen.get_quadruples().iter() {
        // Output raw op_codes and addresses
        obj_content.push_str(&format!("{},{},{},{}\n",
                                      quad.operation,
                                      quad.arg1,
                                      quad.arg2,
                                      quad.result
        ));
    }
    obj_content.push_str("END_QUADRUPLES\n");

    // 5. Write to .obj file
    let output_path = Path::new(input_filename).with_extension("obj");
    let output_filename = output_path.to_str().unwrap_or("output.obj");

    match fs::File::create(output_filename) {
        Ok(file) => {
            let mut writer = BufWriter::new(file);
            if let Err(e) = writer.write_all(obj_content.as_bytes()) {
                eprintln!("Error writing to object file '{}': {}", output_filename, e);
                std::process::exit(1);
            }
            println!("Compilation successful! Output written to {}", output_filename);
        }
        Err(e) => {
            eprintln!("Error creating object file '{}': {}", output_filename, e);
            std::process::exit(1);
        }
    }

    // // Optional: Run the VM if a specific condition is met (e.g., a command-line flag or specific input file)
    // // This is a simple demonstration. A proper CLI would handle this better.
   
    println!("\n--- Attempting to run VM on {} ---", output_filename);
    let mut vm_instance = vm::VM::new();
    match vm_instance.load_obj_file(output_filename) {
        Ok(_) => {
            if let Err(e) = vm_instance.run() {
                eprintln!("VM runtime error: {}", e);
            } else {
                println!("VM execution finished successfully.");
            }
        }
        Err(e) => {
            eprintln!("Error loading object file ('{}') into VM: {}", output_filename, e);
        }
    }
    
}


// --- ALL YOUR EXISTING TESTS GO HERE ---
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
    // println!("Parse result: {:#?}", result.unwrap());
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
    // println!("Parse result: {:#?}", result.unwrap());
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
    // println!("Parse result: {:#?}", result.unwrap());
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
    // println!("Parse result: {:#?}", result.unwrap());
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

    // Check parameter addresses (example, actual addresses depend on counter state)
    // Assuming int_counter starts at 1000 for function 'add' parameters
    assert!(add_func.parameters[0].2 >= 1000); // address of 'a'
    assert!(add_func.parameters[1].2 > add_func.parameters[0].2); // address of 'b'

    println!("Function directory test passed");
    // println!("Function directory: {:#?}", function_directory);
}

#[test]
fn test_duplicate_variable_error() {
    // Test with duplicate local variables in a function
    let program_with_duplicate = r#"
    program example;
    var global_x: int;

    void add(a: int, b: int) [
        var result: int;
        var result: int; // Duplicate local
        {
            result = a + b;
        }
    ];

    main {
        global_x = 10;
    }
    end
    "#;

    let parse_result = babyduck::ProgramParser::new().parse(program_with_duplicate);
    assert!(parse_result.is_ok(), "Failed to parse program: {:?}", parse_result.err());

    let ast = parse_result.unwrap();
    let function_directory_result = FunctionDirectory::from_program(&ast);
    assert!(function_directory_result.is_err(), "Expected error for duplicate local variable, got: {:?}", function_directory_result.ok());

    if let Err(FunctionDirError::DuplicateVariable(var_name, scope)) = function_directory_result {
        println!("Correctly detected duplicate variable '{}' in scope '{}'", var_name, scope);
        assert_eq!(var_name, "result");
        assert_eq!(scope, "add"); // Scope should be the function name
    } else {
        panic!("Expected DuplicateVariable error for local variable, got: {:?}", function_directory_result);
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
    // println!("Parse result: {:#?}", result.unwrap());
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

    let gen_result = quad_gen.generate_for_program(&ast);
    assert!(gen_result.is_ok(), "Quadruple generation failed: {:?}", gen_result.err());

    let quadruples_raw = quad_gen.get_quadruples();

    assert!(!quadruples_raw.is_empty(), "No quadruples were generated");
    assert_eq!(quadruples_raw[0].operation, OpCode::GOTO, "First quad should be GOTO main");
    assert_eq!(quadruples_raw.back().unwrap().operation, OpCode::HALT, "Last quad should be HALT");
    assert!(quadruples_raw.len() >= 17, "Expected at least 17 quadruples for this complex expression, got {}", quadruples_raw.len());
    let has_comparison = quadruples_raw.iter().any(|q| q.operation == OpCode::GT);
    assert!(has_comparison, "No '>' comparison quadruple found");
    println!("\nQuadruple generation test passed");
}

#[test]
fn test_control_flow_quadruple_generation() {
    let program = r#"
    program control_flow_test;
    var x, y, result, counter: int;
    var flag: bool;
    main {
        if (x > y) { result = x; } else { result = y; }
        if (x == 5) { if (y == 10) { result = 15; } else { result = 5; } } else { result = 0; }
        counter = 0; while (counter < 5) do { counter = counter + 1; };
        counter = 0; while (counter < 10) do { counter = counter + 1; if (counter == 5) { result = 100; } };
        flag = x > y; if (flag) { result = 1; } else { result = 0; }
    }
    end
    "#;

    let parse_result = babyduck::ProgramParser::new().parse(program);
    assert!(parse_result.is_ok(), "Failed to parse program: {:?}", parse_result.err());
    let ast = parse_result.unwrap();
    let function_directory_result = FunctionDirectory::from_program(&ast);
    assert!(function_directory_result.is_ok(), "Failed to create function directory: {:?}", function_directory_result.err());
    let function_directory = function_directory_result.unwrap();

    let mut quad_gen = QuadrupleGenerator::new();
    quad_gen.set_function_directory(function_directory);
    let gen_result = quad_gen.generate_for_program(&ast);
    assert!(gen_result.is_ok(), "Quadruple generation failed: {:?}", gen_result.err());

    let quadruples_raw = quad_gen.get_quadruples();
    assert_eq!(quadruples_raw[0].operation, OpCode::GOTO);
    assert_eq!(quadruples_raw.back().unwrap().operation, OpCode::HALT);
    let goto_count = quadruples_raw.iter().filter(|q| q.operation == OpCode::GOTO).count();
    let gotof_count = quadruples_raw.iter().filter(|q| q.operation == OpCode::GOTOF).count();
    assert!(goto_count >= 4 + 1); // 1 initial GOTO main + at least 4 for if/while constructs in test
    assert!(gotof_count >= 5);   // For 5 conditional jumps (if, if, while, while, if)
    let comparison_ops = quadruples_raw.iter().filter(|q| q.operation == OpCode::GT || q.operation == OpCode::LT || q.operation == OpCode::EQ).count();
    assert!(comparison_ops >= 5);
    let assign_ops = quadruples_raw.iter().filter(|q| q.operation == OpCode::ASSIGN).count();
    assert!(assign_ops >= 8);
    let add_ops = quadruples_raw.iter().filter(|q| q.operation == OpCode::ADD).count();
    assert!(add_ops >= 2);
    println!("\nControl flow quadruple generation test passed successfully!");
}


#[test]
fn test_function_call_quadruple_generation() {
    let program = r#"
    program function_test;
    var global_res: int;

    void multiply(param_a: int, param_b: int) [
        var local_prod: int;
        {
            local_prod = param_a * param_b;
            global_res = local_prod;
            print(global_res);
        }
    ];

    main {
        global_res = 0;
        multiply(5, 10); // Call 1
        multiply(global_res, 2); // Call 2
    }
    end
    "#;

    let parse_result = babyduck::ProgramParser::new().parse(program);
    assert!(parse_result.is_ok(), "Failed to parse program: {:?}", parse_result.err());
    let ast = parse_result.unwrap();

    let function_directory_result = FunctionDirectory::from_program(&ast);
    assert!(function_directory_result.is_ok(), "Failed to create function directory: {:?}", function_directory_result.err());
    let function_directory = function_directory_result.unwrap();


    let mut quad_gen = QuadrupleGenerator::new();
    quad_gen.set_function_directory(function_directory);

    let gen_result = quad_gen.generate_for_program(&ast);
    assert!(gen_result.is_ok(), "Quadruple generation failed: {:?}", gen_result.err());

    let quads_raw = quad_gen.get_quadruples();

    assert_eq!(quads_raw[0].operation, OpCode::GOTO, "Quad 0 should be GOTO main");
    let main_code_start_idx = quads_raw[0].result as usize;

    let multiply_func_info = quad_gen.function_directory.as_ref().unwrap().get_function("multiply").unwrap();
    let multiply_start_idx = multiply_func_info.start_quad_idx.unwrap() as usize;

    // Validate multiply function body quads
    let mut func_body_quad_idx = multiply_start_idx;
    assert_eq!(quads_raw[func_body_quad_idx].operation, OpCode::MULT, "Multiply op in function");
    func_body_quad_idx += 1;
    assert_eq!(quads_raw[func_body_quad_idx].operation, OpCode::ASSIGN, "Assign to local_prod");
    func_body_quad_idx += 1;
    assert_eq!(quads_raw[func_body_quad_idx].operation, OpCode::ASSIGN, "Assign to global_res");
    func_body_quad_idx += 1;
    assert_eq!(quads_raw[func_body_quad_idx].operation, OpCode::PRINT, "Print global_res");
    func_body_quad_idx += 1;
    assert_eq!(quads_raw[func_body_quad_idx].operation, OpCode::ENDFUNC, "ENDFUNC for multiply");
    let end_of_multiply_body_idx = func_body_quad_idx;

    assert!(main_code_start_idx > end_of_multiply_body_idx, "Main code should start after function definitions");

    // Validate main body quads related to function calls
    let mut current_quad_idx = main_code_start_idx;
    assert_eq!(quads_raw[current_quad_idx].operation, OpCode::ASSIGN, "Main: assign global_res = 0"); current_quad_idx +=1;

    // Call 1: multiply(5, 10)
    assert_eq!(quads_raw[current_quad_idx].operation, OpCode::ERA, "Main: ERA for multiply (call 1)");
    assert_eq!(quads_raw[current_quad_idx].arg1, multiply_start_idx as i32, "ERA arg1 should be func start index");
    current_quad_idx +=1;
    assert_eq!(quads_raw[current_quad_idx].operation, OpCode::PARAM, "Main: PARAM 1 (5)");
    assert_eq!(quads_raw[current_quad_idx].result, 0, "PARAM result should be param index 0"); // param_a
    current_quad_idx +=1;
    assert_eq!(quads_raw[current_quad_idx].operation, OpCode::PARAM, "Main: PARAM 2 (10)");
    assert_eq!(quads_raw[current_quad_idx].result, 1, "PARAM result should be param index 1"); // param_b
    current_quad_idx +=1;
    assert_eq!(quads_raw[current_quad_idx].operation, OpCode::GOSUB, "Main: GOSUB multiply (call 1)");
    assert_eq!(quads_raw[current_quad_idx].arg1, multiply_start_idx as i32, "GOSUB arg1 should be func start index");
    current_quad_idx +=1;

    // Call 2: multiply(global_res, 2)
    assert_eq!(quads_raw[current_quad_idx].operation, OpCode::ERA, "Main: ERA for multiply (call 2)");
    assert_eq!(quads_raw[current_quad_idx].arg1, multiply_start_idx as i32);
    current_quad_idx +=1;
    assert_eq!(quads_raw[current_quad_idx].operation, OpCode::PARAM, "Main: PARAM 1 (global_res)");
    assert_eq!(quads_raw[current_quad_idx].result, 0);
    current_quad_idx +=1;
    assert_eq!(quads_raw[current_quad_idx].operation, OpCode::PARAM, "Main: PARAM 2 (2)");
    assert_eq!(quads_raw[current_quad_idx].result, 1);
    current_quad_idx +=1;
    assert_eq!(quads_raw[current_quad_idx].operation, OpCode::GOSUB, "Main: GOSUB multiply (call 2)");
    assert_eq!(quads_raw[current_quad_idx].arg1, multiply_start_idx as i32);
    current_quad_idx +=1;

    assert_eq!(quads_raw[current_quad_idx].operation, OpCode::HALT, "Main: HALT at end of program");
    assert_eq!(current_quad_idx, quads_raw.len() - 1, "HALT should be the last quadruple");

    println!("\nFunction call quadruple generation test passed successfully!");
}
