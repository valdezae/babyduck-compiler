use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub babyduck);

pub mod ast;

#[test]
fn babyduck() {
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

    let result = babyduck::ProgramParser::new()
        .parse(program);
    
    match result {
        Ok(parsed) => {
            println!("Successfully parsed program: {:?}", parsed);
        },
        Err(e) => {
            panic!("Parse error: {:?}", e);
        }
    }
}

fn main() {}
