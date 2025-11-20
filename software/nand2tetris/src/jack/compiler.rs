use crate::jack::ast::{Expression, Operator, Term};

pub struct Compiler {
    vm_lines: Vec<String>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            vm_lines: Vec::new(),
        }
    }

    pub fn get_vm_code(&self) -> String {
        self.vm_lines.join("\n")
    }

    pub fn compile(&mut self) {
        self.vm_lines.clear();
    }

    fn compile_expression(&mut self, expr: &Expression) {
        self.compile_term(expr.term.as_ref());
        for (op, term) in &expr.rest {
            self.compile_term(term.as_ref());
            // Here you would add VM code for the operator
            match op {
                // Example for addition
                Operator::Plus => {
                    self.vm_lines.push("add".to_string());
                }
                Operator::Minus => {
                    self.vm_lines.push("sub".to_string());
                }
                Operator::Multiply => {
                    self.vm_lines.push("call Math.multiply 2".to_string());
                }
                Operator::Divide => {
                    self.vm_lines.push("call Math.divide 2".to_string());
                }
                Operator::And => {
                    self.vm_lines.push("and".to_string());
                }
                Operator::Or => {
                    self.vm_lines.push("or".to_string());
                }
                _ => {
                    todo!("Operator {:?} not implemented", op);
                }
            }
        }
    }

    fn compile_term(&mut self, term: &Term) {
        match term {
            Term::IntegerConstant(value) => {
                self.vm_lines.push(format!("push constant {}", value));
            }
            Term::ExpressionInParens(expr) => {
                self.compile_expression(expr);
            }
            _ => {
                todo!("not implemented")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_product_expression() {
        let expr = Expression {
            term: Box::new(Term::IntegerConstant(6)),
            rest: vec![(Operator::Multiply, Box::new(Term::IntegerConstant(7)))],
        };

        let mut compiler = Compiler::new();
        compiler.compile_expression(&expr);
        let vm_code = compiler.get_vm_code();

        let expected_vm_code = "push constant 6\npush constant 7\ncall Math.multiply 2";
        assert_eq!(vm_code, expected_vm_code);
    }

    #[test]
    fn test_compile_expr_in_parens() {
        let expr = Expression {
            term: Box::new(Term::ExpressionInParens(Box::new(Expression {
                term: Box::new(Term::IntegerConstant(3)),
                rest: vec![(Operator::Plus, Box::new(Term::IntegerConstant(4)))]
            }))),
            rest: vec![(Operator::Multiply, Box::new(Term::IntegerConstant(6)))],
        };

        let mut compiler = Compiler::new();
        compiler.compile_expression(&expr);
        let vm_code = compiler.get_vm_code();

        let expected_vm_code = "push constant 3\npush constant 4\nadd\npush constant 6\ncall Math.multiply 2";
        assert_eq!(vm_code, expected_vm_code);
    }
}