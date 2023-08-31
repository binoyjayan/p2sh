use std::rc::Rc;

use self::symtab::SymbolScope;
use crate::code::definitions::{self, *};
use crate::code::opcode::Opcode;
use crate::common::error::CompileError;
use crate::common::object::CompiledFunction;
use crate::common::object::Object;
use crate::compiler::symtab::SymbolTable;
use crate::parser::ast::expr::*;
use crate::parser::ast::stmt::BlockStatement;
use crate::parser::ast::stmt::Statement;
use crate::parser::ast::*;

pub mod symtab;
pub mod symtab_test;
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

// Before compiling a function body (i.e. enter a new scope),
// push a new object of type CompilationScope onto the scopes stack
#[derive(Default, Clone)]
struct CompilationScope {
    instructions: Instructions,
    last_ins: EmittedInstruction, // instruction before the current
    prev_ins: EmittedInstruction, // instruction before the last
}

pub struct Compiler {
    pub constants: Vec<Object>,
    pub symtab: SymbolTable,
    scopes: Vec<CompilationScope>,
    scope_index: usize,
}

impl Compiler {
    pub fn new() -> Compiler {
        let main_scope = CompilationScope {
            instructions: Instructions::default(),
            last_ins: EmittedInstruction::default(),
            prev_ins: EmittedInstruction::default(),
        };
        Compiler {
            constants: Vec::new(),
            symtab: SymbolTable::default(),
            scopes: vec![main_scope],
            scope_index: 0,
        }
    }

    pub fn new_with_state(symtab: SymbolTable, constants: Vec<Object>) -> Compiler {
        let mut compiler = Self::new();
        compiler.constants = constants;
        compiler.symtab = symtab;
        compiler
    }

    pub fn enter_scope(&mut self) {
        let scope = CompilationScope {
            instructions: Instructions::default(),
            last_ins: EmittedInstruction::default(),
            prev_ins: EmittedInstruction::default(),
        };
        self.scopes.push(scope);
        self.scope_index += 1;
        self.symtab = SymbolTable::new_enclosed(self.symtab.clone());
    }

    pub fn leave_scope(&mut self) -> Instructions {
        let instructions = self.get_curr_instructions();
        self.scopes.truncate(self.scopes.len() - 1);
        self.scope_index -= 1;
        let outer = self.symtab.outer.as_ref().unwrap().as_ref().clone();
        self.symtab = outer;
        instructions
    }

    pub fn get_curr_instructions(&self) -> Instructions {
        self.scopes[self.scope_index].instructions.clone()
    }

    pub fn bytecode(&self) -> Bytecode {
        Bytecode {
            instructions: self.get_curr_instructions(),
            constants: self.constants.clone(),
        }
    }

    // Helper to add a constant to the constants pool
    pub fn add_constant(&mut self, obj: Object) -> usize {
        self.constants.push(obj);
        self.constants.len() - 1
    }

    // Helper to add instructions
    pub fn add_instruction(&mut self, ins: Instructions) -> usize {
        let mut curr_ins = self.get_curr_instructions();
        let new_pos = curr_ins.len();
        curr_ins.code.extend_from_slice(&ins.code);
        curr_ins.lines.extend_from_slice(&ins.lines);
        self.scopes[self.scope_index].instructions = curr_ins;
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
        let prev_ins = self.scopes[self.scope_index].last_ins.clone();
        let last_ins = EmittedInstruction::new(op, pos);
        self.scopes[self.scope_index].prev_ins = prev_ins;
        self.scopes[self.scope_index].last_ins = last_ins;
    }

    fn is_last_instruction(&self, opcode: Opcode) -> bool {
        // Check for and empty scope (e.g. functions that doesn't have a body)
        if self.scopes[self.scope_index].instructions.code.is_empty() {
            return false;
        }
        self.scopes[self.scope_index].last_ins.opcode == opcode
    }

    // shortens 'instructions' to cut off the last instruction
    fn remove_last_pop(&mut self) {
        let last_ins = self.scopes[self.scope_index].last_ins.clone();
        let prev_ins = self.scopes[self.scope_index].prev_ins.clone();

        let old_ins = self.get_curr_instructions();
        let new_ins = Instructions {
            code: old_ins.code[..last_ins.position].to_vec(),
            lines: old_ins.lines[..last_ins.position].to_vec(),
        };

        self.scopes[self.scope_index].instructions = new_ins;
        self.scopes[self.scope_index].last_ins = prev_ins;
    }

    // Helper to replace an instruction at an arbitrary offset
    fn replace_instruction(&mut self, pos: usize, new_instruction: &[u8]) {
        let mut curr_ins = self.get_curr_instructions();

        for (i, &byte) in new_instruction.iter().enumerate() {
            // lines remain the same
            curr_ins.code[pos + i] = byte;
        }
        self.scopes[self.scope_index].instructions = curr_ins;
    }

    // Helper to replace the last Opcode::Pop with 'Opcode::ReturnValue'
    fn replace_last_pop_with_return(&mut self) {
        let last_pos = self.scopes[self.scope_index].last_ins.position;
        let new_instruction = definitions::make(Opcode::ReturnValue, &[0], 1);
        self.replace_instruction(last_pos, &new_instruction.code);
        self.scopes[self.scope_index].last_ins.opcode = Opcode::ReturnValue;
    }

    // Recreate instruction with new operand and use 'replace_instruction()'
    // to swap an old instuction for the new one - including the operand
    // The underlying assumption is that only instructions that are of
    // the same type and length are replaced
    fn change_operand(&mut self, op_pos: usize, operand: usize) {
        let op = Opcode::from(self.get_curr_instructions().code[op_pos]);
        let line = self.get_curr_instructions().lines[op_pos];
        let new_instruction = definitions::make(op, &[operand], line);
        // lines remain the same
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
            }
            Statement::Let(stmt) => {
                self.compile_let_stmt(stmt.value)?;
                let symbol = self.symtab.define(&stmt.name.value);
                // Use a Symbol's scope to emit the right instruction
                if symbol.scope == SymbolScope::GlobalScope {
                    self.emit(Opcode::SetGlobal, &[symbol.index], stmt.token.line);
                } else {
                    self.emit(Opcode::SetLocal, &[symbol.index], stmt.token.line);
                }
            }
            Statement::Return(stmt) => {
                self.compile_expression(stmt.value)?;
                self.emit(Opcode::ReturnValue, &[0], stmt.token.line);
            }
            _ => {}
        }
        Ok(())
    }

    fn compile_expression(&mut self, expr: Expression) -> Result<(), CompileError> {
        match expr {
            Expression::Number(num) => {
                let obj = Object::Number(num.value);
                let idx = self.add_constant(obj);
                self.emit(Opcode::Constant, &[idx], num.token.line);
            }
            Expression::Str(s) => {
                let obj = Object::Str(s.value);
                let idx = self.add_constant(obj);
                self.emit(Opcode::Constant, &[idx], s.token.line);
            }
            Expression::Array(arr) => {
                let len = arr.elements.len();
                for e in arr.elements {
                    self.compile_expression(e)?;
                }
                self.emit(Opcode::Array, &[len], arr.token.line);
            }
            Expression::Hash(map) => {
                let len = map.pairs.len() * 2;
                for (key, value) in map.pairs {
                    self.compile_expression(key)?;
                    self.compile_expression(value)?;
                }
                self.emit(Opcode::Map, &[len], map.token.line);
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
                if self.is_last_instruction(Opcode::Pop) {
                    self.remove_last_pop();
                }

                // Emit an 'Jump' with a placeholder. Save it's position so it can be altered later
                let jump_pos = self.emit(Opcode::Jump, &[0xFFFF], expr.token.line);

                // offset of the next-to-be-emitted instruction
                let after_then_pos = self.get_curr_instructions().len();
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
                        if self.is_last_instruction(Opcode::Pop) {
                            self.remove_last_pop();
                        }
                    }
                }
                // offset of the next-to-be-emitted instruction
                let after_else_pos = self.get_curr_instructions().len();
                // change the operand of the Jump instruction to jump over the
                // else branch – it could be Nil or a real 'else_stmt'
                self.change_operand(jump_pos, after_else_pos);
            }
            Expression::Ident(expr) => {
                if let Some(symbol) = self.symtab.resolve(&expr.token.literal) {
                    if symbol.scope == SymbolScope::GlobalScope {
                        self.emit(Opcode::GetGlobal, &[symbol.index], expr.token.line);
                    } else {
                        self.emit(Opcode::GetLocal, &[symbol.index], expr.token.line);
                    }
                } else {
                    return Err(CompileError::new(
                        &format!("undefined variable {}", expr.token.literal),
                        expr.token.line,
                    ));
                }
            }
            Expression::Index(expr) => {
                // Compile the expression being indexed
                self.compile_expression(*expr.left)?;
                // Compile the index expression
                self.compile_expression(*expr.index)?;
                // Emit the index operator
                self.emit(Opcode::Index, &[0], expr.token.line);
            }
            Expression::Function(func) => {
                // enter scope of a function
                self.enter_scope();
                self.compile_block_statement(func.body)?;
                // Leave function scope. If the last expression statement in a
                // function is not turned into an implicit return value, but
                // is still followed by an OpPop instruction, the fix the
                // instruction after compiling the function’s body but before
                // leaving the scope.
                if self.is_last_instruction(Opcode::Pop) {
                    self.replace_last_pop_with_return();
                }
                if !self.is_last_instruction(Opcode::ReturnValue) {
                    self.emit(Opcode::Return, &[0], func.token.line);
                }
                // Take the current symbol table's num_definitions, save it to
                // Object::CompiledFunction. That gives the info on the number
                // of local bindings a function is going to create and use in the VM
                let num_locals = self.symtab.num_definitions;
                let instructions = self.leave_scope();
                let compiled_fn =
                    Object::CompiledFunc(Rc::new(CompiledFunction::new(instructions, num_locals)));
                let idx = self.add_constant(compiled_fn);
                self.emit(Opcode::Constant, &[idx], func.token.line);
            }
            Expression::Call(call) => {
                self.compile_expression(*call.func)?;
                let num_args = call.args.len();
                for arg in call.args {
                    self.compile_expression(arg)?;
                }
                // First operand to OpCall is the number of arguments
                self.emit(Opcode::Call, &[num_args], call.token.line);
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

    fn compile_let_stmt(&mut self, expr: Expression) -> Result<Object, CompileError> {
        self.compile_expression(expr)?;
        Ok(Object::Nil)
    }
}
