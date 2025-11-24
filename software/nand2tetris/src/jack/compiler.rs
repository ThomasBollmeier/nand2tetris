use std::collections::HashMap;
use crate::jack::ast::{Class, Expression, Operator, Term, KeywordConstant, ClassVarDec, ClassVarCategory};
use crate::jack::symbol_table::{SymbolTable, SymbolTableRef};

pub struct Compiler {
    vm_lines: Vec<String>,
    char_map: HashMap<char, u8>,
    curr_class_name: String,
    static_symbol_table: SymbolTableRef,
    curr_symbol_table: SymbolTableRef,
}

impl Compiler {
    pub fn new() -> Self {
        let symbol_table = SymbolTable::new_ref(None);
        Compiler {
            vm_lines: Vec::new(),
            char_map: Self::initialize_char_map(),
            curr_class_name: String::new(),
            static_symbol_table: symbol_table.clone(),
            curr_symbol_table: symbol_table,
        }
    }

    pub fn get_vm_code(&self) -> String {
        self.vm_lines.join("\n")
    }

    pub fn compile_class(&mut self, class: &Class) {
        self.vm_lines.clear();
        self.curr_class_name = class.name.clone();
        self.curr_symbol_table = SymbolTable::new_ref(Some(self.static_symbol_table.clone()));

        for class_var_decl in &class.class_var_declarations {
            self.compile_class_var_declaration(class_var_decl);
        }

        let parent_symbol_table = self.curr_symbol_table.borrow_mut().get_parent();
        if let Some(parent) = parent_symbol_table {
            self.curr_symbol_table = parent;
        }
    }

    fn compile_class_var_declaration(&mut self, class_var_decl: &ClassVarDec) {
        for var_name in &class_var_decl.names {
            match class_var_decl.category {
                ClassVarCategory::Static => {
                    self.static_symbol_table.borrow_mut().add_class_var(
                        var_name.clone(),
                        class_var_decl.category.clone(),
                        class_var_decl.var_type.clone(),
                    );
                }
                ClassVarCategory::Field => {
                    self.curr_symbol_table.borrow_mut().add_class_var(
                        var_name.clone(),
                        class_var_decl.category.clone(),
                        class_var_decl.var_type.clone(),
                    );
                }
            }
        }
    }

    fn compile_expression(&mut self, expr: &Expression) {
        self.compile_term(expr.term.as_ref());
        for (op, term) in &expr.rest {
            self.compile_term(term.as_ref());
            match op {
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
                Operator::GreaterThan => {
                    self.vm_lines.push("gt".to_string());
                }
                Operator::LessThan => {
                    self.vm_lines.push("lt".to_string());
                }
                Operator::Equal => {
                    self.vm_lines.push("eq".to_string());
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
            Term::StringConstant(value) => {
                self.vm_lines.push(format!("push constant {}", value.len()));
                self.vm_lines.push("call String.new 1".to_string());
                for ch in value.chars() {
                    self.vm_lines.push(format!("push constant {}", self.char_to_ascii(ch)));
                    self.vm_lines.push("call String.appendChar 2".to_string());
                }
            }
            Term::KeywordConstant(kw) => {
                match kw {
                    KeywordConstant::True => {
                        self.vm_lines.push("push constant 1".to_string());
                        self.vm_lines.push("neg".to_string());
                    }
                    KeywordConstant::False
                    | KeywordConstant::Null => {
                        self.vm_lines.push("push constant 0".to_string());
                    }
                    KeywordConstant::This => {
                        self.vm_lines.push("push pointer 0".to_string());
                    }
                }
            }
            Term::ExpressionInParens(expr) => {
                self.compile_expression(expr);
            }
            Term::UnaryOp { operator, term } => {
                self.compile_term(term.as_ref());
                match operator {
                    crate::jack::ast::UnaryOperator::Negate => {
                        self.vm_lines.push("neg".to_string());
                    }
                    crate::jack::ast::UnaryOperator::Not => {
                        self.vm_lines.push("not".to_string());
                    }
                }
            }
            _ => {
                todo!("not implemented")
            }
        }
    }

    fn initialize_char_map() -> HashMap<char, u8> {
        let mut map = HashMap::new();
        for i in 0_u8..=127 {
            map.insert(i as char, i);
        }
        map
    }

    fn char_to_ascii(&self, ch: char) -> u8 {
        *self.char_map.get(&ch).unwrap_or(&0)
    }
}

#[cfg(test)]
mod tests {
    use crate::jack::ast::Type;
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

    #[test]
    fn test_compile_class_var_declaration() {
        let class_var_declarations = vec![
            ClassVarDec {
                category: ClassVarCategory::Static,
                var_type: Type::Int,
                names: vec!["count".to_string(), "total".to_string()],
            },
            ClassVarDec {
                category: ClassVarCategory::Field,
                var_type: Type::Boolean,
                names: vec!["flag".to_string()],
            },
        ];

        let class = Class {
            name: "TestClass".to_string(),
            class_var_declarations,
            subroutine_declarations: vec![],
        };

        let mut compiler = Compiler::new();
        compiler.compile_class(&class);

        let static_table = compiler.static_symbol_table.borrow();
        let count_entry = static_table.get_entry("count").unwrap();
        let total_entry = static_table.get_entry("total").unwrap();

        assert_eq!(count_entry.index, 0);
        assert_eq!(total_entry.index, 1);
    }
}