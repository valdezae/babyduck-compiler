use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub babyduck);

pub mod ast;

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

fn main() {
    println!("BabyDuck Compiler");
    println!("Run the tests using 'cargo test' to verify the parser's functionality");
}
