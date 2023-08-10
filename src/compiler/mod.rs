use crate::code::definitions::{self, *};
use crate::code::opcode::Opcode;
use crate::common::error::CompileError;
use crate::common::object::Object;
use crate::parser::ast::expr::*;
use crate::parser::ast::stmt::Statement;
use crate::parser::ast::*;

pub mod tests;

pub struct Bytecode {
    pub instructions: Instructions,
    pub constants: Vec<Object>,
}

pub struct Compiler {
    instructions: Instructions,
    constants: Vec<Object>,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            instructions: Instructions::default(),
            constants: Vec::new(),
        }
    }

    pub fn bytecode(&self) -> Bytecode {
        Bytecode {
            instructions: self.instructions.clone(),
            constants: self.constants.clone(),
        }
    }

    // Helper to add a constant to the constants pool
    pub fn add_constant(&mut self, obj: Object) -> usize {
        self.constants.push(obj);
        self.constants.len() - 1
    }

    // Helper to add a instructions
    pub fn add_instruction(&mut self, ins: Instructions) -> usize {
        let new_pos = self.instructions.len();
        self.instructions.code.extend_from_slice(&ins.code);
        self.instructions.lines.extend_from_slice(&ins.lines);
        new_pos
    }

    // Helper to emit instruction and returnits starting position
    pub fn emit(&mut self, op: Opcode, operands: &[usize], line: usize) -> usize {
        let ins = definitions::make(op, operands, line);
        self.add_instruction(ins)
    }

    pub fn compile(&mut self, pgm: Program) -> Result<(), CompileError> {
        self.compile_program(pgm)?;
        Ok(())
    }

    pub fn compile_program(&mut self, program: Program) -> Result<(), CompileError> {
        self.compile_statements(program.statements)
    }

    fn compile_statements(&mut self, statements: Vec<Statement>) -> Result<(), CompileError> {
        self.compile_statements_nounwrap(statements)
        // if let Object::Return(retval) = result {
        //     return Ok(*retval);
        // }
        // Ok(result)
    }

    fn compile_statements_nounwrap(
        &mut self,
        statements: Vec<Statement>,
    ) -> Result<(), CompileError> {
        // let mut result = Object::Nil;
        for stmt in statements {
            // let result =
            self.compile_statement(stmt)?;
            // if let Object::Return(_) = result {
            //     return Ok(result);
            // }
        }
        Ok(())
    }

    fn compile_statement(&mut self, stmt: Statement) -> Result<(), CompileError> {
        match stmt {
            Statement::Expr(stmt) => {
                self.compile_expression(stmt.value)?;
                self.emit(Opcode::Pop, &[0], stmt.token.line);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn compile_expression(&mut self, expr: Expression) -> Result<(), CompileError> {
        match expr {
            Expression::Number(num) => {
                let obj = Object::Number(num.value);
                let idx = self.add_constant(obj);
                self.emit(Opcode::Constant, &[idx], num.token.line);
            }
            Expression::Binary(binary) => {
                // In case of '<', re order the operands to reuse the '>' operator
                match binary.operator.as_ref() {
                    "<" => {
                        self.compile_expression(*binary.right)?;
                        self.compile_expression(*binary.left)?;
                    }
                    _ => {
                        self.compile_expression(*binary.left)?;
                        self.compile_expression(*binary.right)?;
                    }
                }
                self.compile_infix_expr(&binary.operator, binary.token.line)?;
            }
            Expression::Bool(b) => {
                if b.value {
                    self.emit(Opcode::True, &[0], b.token.line);
                } else {
                    self.emit(Opcode::False, &[0], b.token.line);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn compile_infix_expr(&mut self, operator: &str, line: usize) -> Result<(), CompileError> {
        match operator {
            "+" => {
                self.emit(Opcode::Add, &[0], line);
            }
            "-" => {
                self.emit(Opcode::Sub, &[0], line);
            }
            "*" => {
                self.emit(Opcode::Mul, &[0], line);
            }
            "/" => {
                self.emit(Opcode::Div, &[0], line);
            }
            "==" => {
                self.emit(Opcode::Equal, &[0], line);
            }
            "!=" => {
                self.emit(Opcode::NotEqual, &[0], line);
            }
            ">" | "<" => {
                self.emit(Opcode::Greater, &[0], line);
            }
            _ => return Err(CompileError::new("invalid binary operator", line)),
        }
        Ok(())
    }
}
