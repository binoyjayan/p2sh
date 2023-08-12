use crate::code::definitions::{self, *};
use crate::code::opcode::Opcode;
use crate::common::error::CompileError;
use crate::common::object::Object;
use crate::parser::ast::expr::*;
use crate::parser::ast::stmt::BlockStatement;
use crate::parser::ast::stmt::Statement;
use crate::parser::ast::*;

pub mod tests;

pub struct Bytecode {
    pub instructions: Instructions,
    pub constants: Vec<Object>,
}

#[derive(Default, Clone)]
struct EmittedInstruction {
    opcode: Opcode,
    position: usize,
}

impl EmittedInstruction {
    fn new(opcode: Opcode, position: usize) -> Self {
        Self { opcode, position }
    }
}

pub struct Compiler {
    instructions: Instructions,
    constants: Vec<Object>,
    last_ins: EmittedInstruction, // instruction before the current
    prev_ins: EmittedInstruction, // instruction before the last
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            instructions: Instructions::default(),
            constants: Vec::new(),
            last_ins: EmittedInstruction::default(),
            prev_ins: EmittedInstruction::default(),
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

    // Helper to emit instruction and return its starting position
    pub fn emit(&mut self, op: Opcode, operands: &[usize], line: usize) -> usize {
        let ins = definitions::make(op, operands, line);
        let pos = self.add_instruction(ins);
        self.set_last_instruction(op, pos);
        pos
    }

    // Save the last and the previous instructions
    fn set_last_instruction(&mut self, op: Opcode, pos: usize) {
        let prev_ins = self.last_ins.clone();
        let last_ins = EmittedInstruction::new(op, pos);
        self.prev_ins = prev_ins;
        self.last_ins = last_ins;
    }

    fn is_last_instruction_pop(&self) -> bool {
        self.last_ins.opcode == Opcode::Pop
    }

    // shortens 'instructions' to cut off the last instruction
    fn remove_last_pop(&mut self) {
        self.instructions.code.truncate(self.last_ins.position);
        self.instructions.lines.truncate(self.last_ins.position);
        self.last_ins = self.prev_ins.clone();
    }

    // Helper to replace an instruction at an arbitrary offset
    fn replace_instruction(&mut self, pos: usize, new_instruction: &[u8]) {
        for (i, &byte) in new_instruction.iter().enumerate() {
            self.instructions.code[pos + i] = byte;
        }
    }

    // Recreate instruction with new operand and use 'replace_instruction()'
    // to swap an old instuction for the new one - including the operand
    // The underlying assumption is that only instructions that are of
    // the same type and length are replaced
    fn change_operand(&mut self, op_pos: usize, operand: usize) {
        let op = Opcode::from(self.instructions.code[op_pos]);
        let line = self.instructions.lines[op_pos];
        let new_instruction = definitions::make(op, &[operand], line);
        self.replace_instruction(op_pos, &new_instruction.code);
    }

    pub fn compile(&mut self, pgm: Program) -> Result<(), CompileError> {
        self.compile_program(pgm)?;
        Ok(())
    }

    pub fn compile_program(&mut self, program: Program) -> Result<(), CompileError> {
        self.compile_statements(program.statements)
    }

    fn compile_block_statement(&mut self, stmt: BlockStatement) -> Result<(), CompileError> {
        self.compile_statements_nounwrap(stmt.statements)
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
            Expression::Unary(u) => {
                self.compile_expression(*u.right)?;
                match u.operator.as_ref() {
                    "!" => {
                        self.emit(Opcode::Bang, &[0], u.token.line);
                    }
                    "-" => {
                        self.emit(Opcode::Minus, &[0], u.token.line);
                    }
                    _ => return Err(CompileError::new("invalid binary operator", u.token.line)),
                }
            }
            Expression::Bool(b) => {
                if b.value {
                    self.emit(Opcode::True, &[0], b.token.line);
                } else {
                    self.emit(Opcode::False, &[0], b.token.line);
                }
            }
            Expression::If(expr) => {
                self.compile_expression(*expr.condition)?;
                // Emit an 'JumpIfFalse' with a placeholder. Save it's position so it can be altered later
                let jump_if_false_pos = self.emit(Opcode::JumpIfFalse, &[0xFFFF], expr.token.line);
                self.compile_block_statement(expr.then_stmt)?;
                // Get rid of the extra Pop that comes with the result of compiling 'then_stmt'
                // This is so that we don't loose the result of the 'if' expression
                if self.is_last_instruction_pop() {
                    self.remove_last_pop();
                }

                // Emit an 'Jump' with a placeholder. Save it's position so it can be altered later
                let jump_pos = self.emit(Opcode::Jump, &[0xFFFF], expr.token.line);

                // offset of the next-to-be-emitted instruction
                let after_then_pos = self.instructions.len();
                // Replace the operand of the placeholder 'JumpIfFalse' instruction with the
                // position of the instruction that comes after the 'then' statement
                self.change_operand(jump_if_false_pos, after_then_pos);

                // Look for an 'else' branch
                match expr.else_stmt {
                    None => {
                        // Result of if expression when there is no 'else' branch
                        self.emit(Opcode::Nil, &[0], expr.token.line);
                    }
                    Some(else_stmt) => {
                        // TODO: Find line number of 'else_stmt'
                        self.compile_block_statement(else_stmt)?;
                        if self.is_last_instruction_pop() {
                            self.remove_last_pop();
                        }
                    }
                }
                // offset of the next-to-be-emitted instruction
                let after_else_pos = self.instructions.len();
                // change the operand of the Jump instruction to jump over the
                // else branch – it could be Nil or a real 'else_stmt'
                self.change_operand(jump_pos, after_else_pos);
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
