use crate::jack::ast::*;
use crate::jack::symbol_table::{SymbolTable, SymbolTableRef};
use crate::vmtrans::ast::Segment;
use std::collections::HashMap;

pub struct Compiler {
    vm_lines: Vec<String>,
    char_map: HashMap<char, u8>,
    curr_class_name: String,
    curr_symbols: Option<SymbolTableRef>,
    class_symbols: HashMap<String, SymbolTableRef>,
    next_label_num: usize,
    curr_subroutine_category: Option<SubroutineCategory>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            vm_lines: Vec::new(),
            char_map: Self::initialize_char_map(),
            curr_class_name: String::new(),
            curr_symbols: None,
            class_symbols: HashMap::new(),
            next_label_num: 0,
            curr_subroutine_category: None,
        }
    }

    pub fn get_vm_code(&self) -> String {
        self.vm_lines.join("\n")
    }

    pub fn compile_class(&mut self, class: &Class) {
        self.vm_lines.clear();
        self.curr_class_name = class.name.clone();
        self.next_label_num = 1;

        self.curr_symbols = Some(SymbolTable::new_ref(None));
        self.class_symbols.insert(
            class.name.clone(),
            self.curr_symbols.as_ref().unwrap().clone(),
        );

        for class_var_decl in &class.class_var_declarations {
            self.compile_class_var_declaration(class_var_decl);
        }

        let num_fields = class
            .class_var_declarations
            .iter()
            .map(|class_var_decl| class_var_decl.names.len())
            .sum();

        for subroutine_decl in &class.subroutine_declarations {
            self.compile_subroutine(subroutine_decl, num_fields);
        }

        self.curr_symbols = None;
    }

    fn compile_subroutine(&mut self, subroutine_decl: &SubroutineDec, num_fields: usize) {
        let subroutine_symbols = SymbolTable::new_ref(Some(self.get_current_symbols()));
        self.curr_symbols = Some(subroutine_symbols.clone());
        self.curr_subroutine_category = Some(subroutine_decl.category.clone());

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
        self.curr_subroutine_category = None;
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

        for statement in &body.statements {
            self.compile_statement(statement);
        }
    }

    fn compile_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Let {
                var_name,
                index_expression,
                value_expression,
            } => {
                self.compile_let_statement(var_name, index_expression, value_expression);
            }
            Statement::If {
                condition,
                if_statements,
                else_statements,
            } => {
                self.compile_if_statement(condition, if_statements, else_statements);
            }
            Statement::While {
                condition,
                body_statements,
            } => {
                self.compile_while_statement(condition, body_statements);
            }
            Statement::Do { subroutine_call } => {
                self.compile_call(subroutine_call);
                self.vm_write_str("pop temp 0");
            }
            Statement::Return { value } => {
                if let Some(expr) = value {
                    self.compile_expression(expr);
                } else {
                    self.vm_write_str("push constant 0");
                }
                self.vm_write_str("return");
            }
            _ => {
                // Placeholder for other statement types
            }
        }
    }

    fn compile_while_statement(&mut self, condition: &Expression, body_statements: &[Statement]) {
        let start_label = self.create_label();
        let end_label = self.create_label();

        self.vm_write(format!("label {}", start_label));
        self.compile_expression(condition);
        self.vm_write_str("not");
        self.vm_write(format!("if-goto {}", end_label));
        for stmt in body_statements {
            self.compile_statement(stmt);
        }
        self.vm_write(format!("goto {}", start_label));
        self.vm_write(format!("label {}", end_label));
    }

    fn compile_let_statement(
        &mut self,
        var_name: &str,
        index_expression: &Option<Expression>,
        value_expression: &Expression,
    ) {
        match index_expression {
            Some(index_expr) => {
                self.compile_var_name(var_name);
                self.compile_expression(index_expr);
                self.vm_write_str("add");
                self.compile_expression(value_expression);
                self.vm_write_str("pop temp 0");
                self.vm_write_str("pop pointer 1");
                self.vm_write_str("push temp 0");
                self.vm_write_str("pop that 0");
            }
            None => {
                self.compile_expression(value_expression);
                let (segment_str, index) = self.get_segment_and_index(var_name);
                self.vm_write(format!("pop {} {}", segment_str, index));
            }
        }
    }

    fn compile_if_statement(
        &mut self,
        condition: &Expression,
        if_statements: &Vec<Statement>,
        else_statements: &Option<Vec<Statement>>,
    ) {
        self.compile_expression(condition);
        self.vm_write_str("not");

        match else_statements {
            Some(else_statements) => {
                let else_label = self.create_label();
                let end_label = self.create_label();
                self.vm_write(format!("if-goto {}", else_label));
                for stmt in if_statements {
                    self.compile_statement(stmt);
                }
                self.vm_write(format!("goto {}", end_label));
                self.vm_write(format!("label {}", else_label));
                for stmt in else_statements {
                    self.compile_statement(stmt);
                }
                self.vm_write(format!("label {}", end_label));
            }
            None => {
                let end_label = self.create_label();
                self.vm_write(format!("if-goto {}", end_label));
                for stmt in if_statements {
                    self.compile_statement(stmt);
                }
                self.vm_write(format!("label {}", end_label));
            }
        }
    }

    fn create_label(&mut self) -> String {
        let label = format!("l{}", self.next_label_num);
        self.next_label_num += 1;
        label
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
            Term::VarName(name) => {
                self.compile_var_name(name);
            }
            Term::VarNameWithIndex {
                var_name,
                index_expression,
            } => {
                self.compile_var_name_with_index(var_name, index_expression);
            }
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
            Term::SubroutineCall(call) => {
                self.compile_call(call);
            }
            _ => {
                todo!("not implemented")
            }
        }
    }

    fn compile_call(&mut self, call: &SubroutineCall) {
        let mut is_method_call = false;

        match &call.class_or_instance_name {
            Some(name) => {
                let symbols = self.get_current_symbols();
                if let Some(_) = symbols.borrow().get_entry(&name) {
                    // It's an instance method call
                    is_method_call = true;
                    self.compile_var_name(&name);
                }
            }
            None => {
                // Method call on 'this'
                is_method_call = true;
                self.vm_write_str("push pointer 0");
            }
        }

        for arg in &call.arguments {
            self.compile_expression(arg);
        }

        let full_name = match &call.class_or_instance_name {
            Some(name) => {
                if !is_method_call {
                    // It's a class method call
                    format!("{}.{}", name, call.subroutine_name)
                } else {
                    // It's an instance method call
                    let symbols = self.get_current_symbols();
                    let entry = symbols
                        .borrow()
                        .get_entry(name)
                        .expect(&format!("Variable {} not found", name));
                    match &entry.var_type {
                        Type::Class(class_name) => {
                            format!("{}.{}", class_name, call.subroutine_name)
                        }
                        _ => panic!("Expected class type for instance method call"),
                    }
                }
            }
            None => format!("{}.{}", self.curr_class_name, call.subroutine_name),
        };
        let num_args = call.arguments.len() + if is_method_call { 1 } else { 0 };

        self.vm_write(format!("call {} {}", full_name, num_args));
    }

    fn compile_var_name(&mut self, name: &str) {
        let (segment_str, index) = self.get_segment_and_index(name);
        self.vm_write(format!("push {} {}", segment_str, index));
    }

    fn get_segment_and_index(&mut self, name: &str) -> (String, u16) {
        let symbols = self.get_current_symbols();
        let entry = symbols
            .borrow()
            .get_entry(name)
            .expect(&format!("Variable {name} not found"));
        let segment_str = match entry.segment {
            Segment::Static => "static".to_string(),
            Segment::This => match self.curr_subroutine_category {
                Some(SubroutineCategory::Method) | Some(SubroutineCategory::Constructor) => {
                    "this".to_string()
                }
                _ => panic!("Cannot access field '{name}' in function"),
            },
            Segment::That => "that".to_string(),
            Segment::Argument => "argument".to_string(),
            Segment::Local => "local".to_string(),
            Segment::Pointer => "pointer".to_string(),
            Segment::Temp => "temp".to_string(),
            Segment::Constant => panic!("Constant segment not valid for variable access"),
        };
        (segment_str, entry.index)
    }

    fn compile_var_name_with_index(&mut self, var_name: &str, index_expression: &Expression) {
        self.compile_var_name(&var_name);
        self.compile_expression(index_expression);
        self.vm_write_str("add");
        self.vm_write_str("pop pointer 1");
        self.vm_write_str("push that 0");
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
            field String firstName;
            field boolean isMale;

            constructor Person new(String aName, String aFirstName, boolean aIsMale) {
              let name = aName;
              let firstName = aFirstName;
              let isMale = aIsMale;
              return this;
            }

            method String getName() {
              return name;
            }

            method String getPrefix() {
              if (isMale) {
                return "Mr. ";
              } else {
                return "Ms. ";
              }
            }

            function void main() {
              var Person p;
              var String name;
              let p = Person.new("Doe", "John", true);
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

        let expected_start =
            "function Person.new 0\npush constant 3\ncall Memory.alloc 1\npop pointer 0";
        assert!(vm_code.starts_with(expected_start));
    }

    #[test]
    fn test_compile_let_statement_with_index() {
        let code = r#"
        class Test {
            field Array arr;

            constructor Test new() {
                let arr = Array.new(10);
                return this;
            }

            method void setValue(int index, int value) {
                let arr[index] = value;
                return;
            }
        }
        "#;
        let class = parse_class(code);
        let mut compiler = Compiler::new();
        compiler.compile_class(&class);

        let vm_code = compiler.get_vm_code();

        let expected_code = vec![
            "function Test.setValue 0",
            "push this 0",
            "push argument 1",
            "add",
            "push argument 2",
            "pop temp 0",
            "pop pointer 1",
            "push temp 0",
            "pop that 0",
            "push constant 0",
            "return",
        ]
        .join("\n");

        assert!(vm_code.contains(&expected_code));
    }

    #[test]
    fn test_compile_while_statement() {
        let code = r#"
        class Test {
            function void countDown(int n) {
                var int i;
                let i = n;
                while (i > 0) {
                    do Output.printInt(i);
                    let i = i - 1;
                }
                return;
            }
        }
        "#;
        let class = parse_class(code);
        let mut compiler = Compiler::new();
        compiler.compile_class(&class);

        let vm_code = compiler.get_vm_code();
        
        let expected_code = vec![
            "label l1",
            "push local 0",
            "push constant 0",
            "gt",
            "not",
            "if-goto l2",
            "push local 0",
            "call Output.printInt 1",
            "pop temp 0",
            "push local 0",
            "push constant 1",
            "sub",
            "pop local 0",
            "goto l1",
            "label l2",
        ]
        .join("\n");

        assert!(vm_code.contains(&expected_code));
    }


    fn parse_class(code: &str) -> Class {
        let mut stream = StringCharStream::new(code);
        let mut lexer = Lexer::new(&mut stream);
        let mut parser = Parser::new(&mut lexer);
        parser.create_class_ast().unwrap()
    }
}
