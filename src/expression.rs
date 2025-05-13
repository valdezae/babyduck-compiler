use ast::Statement;
use crate::ast;

pub struct Quadruple {
    pub operation_code: String,
    pub operand1: String,
    pub operand2: String,
    pub result: String,
}

impl Quadruple {
    pub fn new(operation_code: String, operand1: String, operand2: String, result: String) -> Self {
        Quadruple {
            operation_code,
            operand1,
            operand2,
            result,
        }
    }
}

pub fn generate_quadruples(statements: &[Statement]) -> Vec<Quadruple> {
    let mut quadruples = Vec::new();
    let mut operand_stack: Vec<String> = Vec::new();
    

    

    quadruples
}