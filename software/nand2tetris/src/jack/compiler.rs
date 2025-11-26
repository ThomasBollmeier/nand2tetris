use crate::jack::ast::*;
use crate::jack::symbol_table::{SymbolTable, SymbolTableRef};
use std::collections::HashMap;
use crate::vmtrans::ast::Segment;

pub struct Compiler {
    vm_lines: Vec<String>,
    char_map: HashMap<char, u8>,
    curr_class_name: String,
    curr_symbols: Option<SymbolTableRef>,
    class_symbols: HashMap<String, SymbolTableRef>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            vm_lines: Vec::new(),
            char_map: Self::initialize_char_map(),
            curr_class_name: String::new(),
            curr_symbols: None,
            class_symbols: HashMap::new(),
        }
    }

    pub fn get_vm_code(&self) -> String {
        self.vm_lines.join("\n")
    }

    pub fn compile_class(&mut self, class: &Class) {
        self.vm_lines.clear();
        self.curr_class_name = class.name.clone();
        self.curr_symbols = Some(SymbolTable::new_ref(None));
        self.class_symbols.insert(
            class.name.clone(),
            self.curr_symbols.as_ref().unwrap().clone(),
        );

        for class_var_decl in &class.class_var_declarations {
            self.compile_class_var_declaration(class_var_decl);
        }

        let num_fields = class.class_var_declarations
            .iter()
            .map(|class_var_decl| { class_var_decl.names.len() })
            .sum();

        for subroutine_decl in &class.subroutine_declarations {
            self.compile_subroutine(subroutine_decl, num_fields);
        }

        self.curr_symbols = None;
    }

    fn compile_subroutine(&mut self, subroutine_decl: &SubroutineDec, num_fields: usize) {
        let subroutine_symbols = SymbolTable::new_ref(Some(self.get_current_symbols()));
        self.curr_symbols = Some(subroutine_symbols.clone());

        if subroutine_decl.category == SubroutineCategory::Method {
            subroutine_symbols.borrow_mut().add_parameter(
                "this".to_string(),
                Type::Class(self.curr_class_name.clone()),
            );
        }

        for (param_type, param_name) in &subroutine_decl.parameters {
            subroutine_symbols
                .borrow_mut()
                .add_parameter(param_name.clone(), param_type.clone());
        }

        let num_locals = subroutine_decl
            .body
            .var_declarations
            .iter()
            .map(|var_dec| var_dec.names.len())
            .sum::<usize>();

        self.vm_write(format!(
            "function {}.{} {}",
            self.curr_class_name, subroutine_decl.name, num_locals
        ));

        if subroutine_decl.category == SubroutineCategory::Constructor {
            subroutine_symbols.borrow_mut().add_var(
                Segment::Pointer,
                "this".to_string(),
                Type::Class(self.curr_class_name.clone()),
            );

            self.vm_write(format!("push constant {}", num_fields));
            self.vm_write_str("call Memory.alloc 1");
            self.vm_write_str("pop pointer 0");
        }

        self.compile_subroutine_body(&subroutine_decl.body);

        let parent = subroutine_symbols
            .borrow_mut()
            .get_parent()
            .expect("subroutine parent not found");
        self.curr_symbols = Some(parent);
    }

    fn compile_subroutine_body(&mut self, body: &Body) {
        let symbols = self.get_current_symbols();

        for var_decl in &body.var_declarations {
            for name in &var_decl.names {
                symbols
                    .borrow_mut()
                    .add_local(name.clone(), var_decl.var_type.clone());
            }
        }

        for _statement in &body.statements {

        }
    }

    fn get_class_symbols(&self, class_name: &str) -> SymbolTableRef {
        self.class_symbols
            .get(class_name)
            .expect(&format!(
                "No symbol table for class {}",
                self.curr_class_name
            ))
            .clone()
    }

    fn get_current_symbols(&self) -> SymbolTableRef {
        self.curr_symbols.clone().expect(&format!(
            "No symbol table for class {}",
            self.curr_class_name
        ))
    }

    fn compile_class_var_declaration(&mut self, class_var_decl: &ClassVarDec) {
        let symbol_table = self.get_current_symbols();

        for var_name in &class_var_decl.names {
            symbol_table.borrow_mut().add_class_var(
                var_name.clone(),
                class_var_decl.category.clone(),
                class_var_decl.var_type.clone(),
            );
        }
    }

    fn compile_expression(&mut self, expr: &Expression) {
        self.compile_term(expr.term.as_ref());
        for (op, term) in &expr.rest {
            self.compile_term(term.as_ref());
            match op {
                Operator::Plus => {
                    self.vm_write_str("add");
                }
                Operator::Minus => {
                    self.vm_write_str("sub");
                }
                Operator::Multiply => {
                    self.vm_write_str("call Math.multiply 2");
                }
                Operator::Divide => {
                    self.vm_write_str("call Math.divide 2");
                }
                Operator::And => {
                    self.vm_write_str("and");
                }
                Operator::Or => {
                    self.vm_write_str("or");
                }
                Operator::GreaterThan => {
                    self.vm_write_str("gt");
                }
                Operator::LessThan => {
                    self.vm_write_str("lt");
                }
                Operator::Equal => {
                    self.vm_write_str("eq");
                }
            }
        }
    }

    fn compile_term(&mut self, term: &Term) {
        match term {
            Term::IntegerConstant(value) => {
                self.vm_write(format!("push constant {}", value));
            }
            Term::StringConstant(value) => {
                self.vm_write(format!("push constant {}", value.len()));
                self.vm_write_str("call String.new 1");
                for ch in value.chars() {
                    self.vm_write(format!("push constant {}", self.char_to_ascii(ch)));
                    self.vm_write_str("call String.appendChar 2");
                }
            }
            Term::KeywordConstant(kw) => match kw {
                KeywordConstant::True => {
                    self.vm_write_str("push constant 1");
                    self.vm_write_str("neg");
                }
                KeywordConstant::False | KeywordConstant::Null => {
                    self.vm_write_str("push constant 0");
                }
                KeywordConstant::This => {
                    self.vm_write_str("push pointer 0");
                }
            },
            Term::ExpressionInParens(expr) => {
                self.compile_expression(expr);
            }
            Term::UnaryOp { operator, term } => {
                self.compile_term(term.as_ref());
                match operator {
                    UnaryOperator::Negate => {
                        self.vm_write_str("neg");
                    }
                    UnaryOperator::Not => {
                        self.vm_write_str("not");
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

    fn vm_write_str(&mut self, line: &str) {
        self.vm_lines.push(line.to_string());
    }
    fn vm_write(&mut self, line: String) {
        self.vm_lines.push(line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammarous::StringCharStream;
    use crate::jack::{Lexer, Parser};

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
                rest: vec![(Operator::Plus, Box::new(Term::IntegerConstant(4)))],
            }))),
            rest: vec![(Operator::Multiply, Box::new(Term::IntegerConstant(6)))],
        };

        let mut compiler = Compiler::new();
        compiler.compile_expression(&expr);
        let vm_code = compiler.get_vm_code();

        let expected_vm_code =
            "push constant 3\npush constant 4\nadd\npush constant 6\ncall Math.multiply 2";
        assert_eq!(vm_code, expected_vm_code);
    }

    #[test]
    fn test_compile_class_var_declaration() {
        let code = r#"class TestClass {
            static int count, total;
            field boolean flag;
        }"#;
        let class = parse_class(code);

        let mut compiler = Compiler::new();
        compiler.compile_class(&class);

        let symbol_table = compiler.get_class_symbols("TestClass");
        let symbol_table = symbol_table.borrow();
        let count_entry = symbol_table.get_entry("count").unwrap();
        let total_entry = symbol_table.get_entry("total").unwrap();

        assert_eq!(count_entry.index, 0);
        assert_eq!(total_entry.index, 1);
    }

    #[test]
    fn test_compile_class() {
        let code = r#"
        class Person {
            field String name;
            field String first_name;

            constructor Person new(String aName, String aFirstName) {
              let name = aName;
              let firstName = aFirstName;
              return this;
            }

            method String getName() {
              return name;
            }

            function void main() {
              var Person p;
              let p = Person.new("Doe", "John");
              let name = p.getName();
              do Output.printString(name);
              return;
            }

        }
        "#;
        let class = parse_class(code);
        let mut compiler = Compiler::new();
        compiler.compile_class(&class);

        let vm_code = compiler.get_vm_code();

        print!("{vm_code}");

        let expected_start = "function Person.new 0\npush constant 2\ncall Memory.alloc 1\npop pointer 0";
        assert!(vm_code.starts_with(expected_start));
    }

    fn parse_class(code: &str) -> Class {
        let mut stream = StringCharStream::new(code);
        let mut lexer = Lexer::new(&mut stream);
        let mut parser = Parser::new(&mut lexer);
        parser.create_class_ast().unwrap()
    }
}
