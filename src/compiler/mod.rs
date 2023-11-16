use std::rc::Rc;

use self::symtab::Symbol;
use self::symtab::SymbolScope;
use crate::code::definitions::{self, *};
use crate::code::opcode::Opcode;
use crate::common::builtins::functions::BUILTINFNS;
use crate::common::builtins::variables::BuiltinVarType;
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
    pub constants: Vec<Rc<Object>>,
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

// Keep track of the positions of the 'loop' instruction
#[derive(Default, Clone)]
struct LoopContext {
    label: Option<String>,
    begin: usize,
    // positions of 'break' instructions in the current loop
    break_positions: Vec<usize>,
}

impl LoopContext {
    fn new(label: Option<String>, position: usize) -> Self {
        Self {
            label,
            begin: position,
            break_positions: Vec::new(),
        }
    }
}

// Before compiling a function body (i.e. enter a new scope),
// push a new object of type CompilationScope onto the scopes stack
#[derive(Default, Clone)]
struct CompilationScope {
    instructions: Instructions,
    last_ins: EmittedInstruction, // instruction before the current
    prev_ins: EmittedInstruction, // instruction before the last
    loop_stack: Vec<LoopContext>, // stack of 'loop' instructions
    scope_depth: usize,           // depth within the current scope
}

pub struct Compiler {
    pub constants: Vec<Rc<Object>>,
    pub symtab: SymbolTable,
    scopes: Vec<CompilationScope>,
    scope_index: usize,
}

impl Compiler {
    pub fn new() -> Compiler {
        let mut symtab = SymbolTable::default();

        for (i, sym) in BUILTINFNS.iter().enumerate() {
            // Define the built-in function via an index into the 'BUILTINS' array
            symtab.define_builtin_fn(i, sym.name);
        }

        // Define the built-in variables
        for n in BuiltinVarType::range() {
            let name: &str = BuiltinVarType::from(n).into();
            symtab.define_builtin_var(n, name);
        }

        let main_scope = CompilationScope::default();
        Compiler {
            constants: Vec::new(),
            symtab,
            scopes: vec![main_scope],
            scope_index: 0,
        }
    }

    pub fn new_with_state(symtab: SymbolTable, constants: Vec<Rc<Object>>) -> Compiler {
        let mut compiler = Self::new();
        compiler.constants = constants;
        compiler.symtab = symtab;
        compiler
    }

    pub fn enter_scope(&mut self) {
        let scope = CompilationScope::default();
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
        let instructions = self.get_curr_instructions();
        let constants = self.constants.clone();
        #[cfg(feature = "debug_print_code")]
        {
            instructions.disassemble();
            self.print_constants();
        }
        Bytecode {
            instructions,
            constants,
        }
    }

    #[allow(dead_code)]
    fn print_constants(&self) {
        let constants = self.constants.clone();
        println!(
            "----------- Constants [len: {:<4}] --------------------",
            constants.len(),
        );

        for (i, obj) in constants.iter().enumerate() {
            println!("[{}] {}", i, obj);
            match obj.as_ref() {
                Object::Clos(cl) => {
                    let closure = cl.clone();
                    closure.func.instructions.disassemble();
                }
                Object::Func(func) => {
                    func.instructions.disassemble();
                }
                _ => {}
            }
        }
        println!("------------------------------------------------------");
    }

    // Helper to add a constant to the constants pool
    pub fn add_constant(&mut self, obj: Object) -> usize {
        self.constants.push(Rc::new(obj));
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

    fn load_symbol(&mut self, sym: Rc<Symbol>, line: usize) {
        match sym.scope {
            SymbolScope::Global => self.emit(Opcode::GetGlobal, &[sym.index], line),
            SymbolScope::Local => self.emit(Opcode::GetLocal, &[sym.index], line),
            SymbolScope::BuiltinFn => self.emit(Opcode::GetBuiltinFn, &[sym.index], line),
            SymbolScope::BuiltinVar => self.emit(Opcode::GetBuiltinVar, &[sym.index], line),
            SymbolScope::Free => self.emit(Opcode::GetFree, &[sym.index], line),
            SymbolScope::Function => self.emit(Opcode::CurrClosure, &[sym.index], line),
        };
    }

    // Save symbol as part of an assignment. Note that these operations are
    // different from the ones used to define a symbol's value.
    fn save_symbol(&mut self, sym: Rc<Symbol>, line: usize) -> Result<(), CompileError> {
        match sym.scope {
            SymbolScope::Global => self.emit(Opcode::SetGlobal, &[sym.index], line),
            SymbolScope::Local => self.emit(Opcode::SetLocal, &[sym.index], line),
            SymbolScope::Free => self.emit(Opcode::SetFree, &[sym.index], line),
            _ => {
                return Err(CompileError::new("Invalid lvalue", line));
            }
        };
        Ok(())
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

    fn patch_jump(&mut self, pos: usize) {
        // offset of the next-to-be-emitted instruction
        let after_pos = self.get_curr_instructions().len();
        // Replace the operand of the instruction at position 'pos'
        // with the position of the next-to-be-emitted instruction.
        self.change_operand(pos, after_pos);
    }

    pub fn compile(&mut self, pgm: Program) -> Result<(), CompileError> {
        self.compile_program(pgm)?;
        Ok(())
    }

    pub fn compile_program(&mut self, program: Program) -> Result<(), CompileError> {
        self.compile_statements(program.statements)
    }

    fn compile_block_statement(&mut self, stmt: BlockStatement) -> Result<(), CompileError> {
        self.scopes[self.scope_index].scope_depth += 1;
        for stmt in stmt.statements {
            self.compile_statement(stmt)?;
        }
        self.scopes[self.scope_index].scope_depth -= 1;
        Ok(())
    }

    fn compile_statements(&mut self, statements: Vec<Statement>) -> Result<(), CompileError> {
        for stmt in statements {
            self.compile_statement(stmt)?;
        }
        Ok(())
    }

    fn compile_statement(&mut self, stmt: Statement) -> Result<(), CompileError> {
        match stmt {
            Statement::Expr(stmt) => {
                self.compile_expression(stmt.value)?;
                // Unlike the 'let' and the 'return' statments, expression
                // statements do not consume the result of the expression
                // So emit a Pop instruction to cleanup the stack.
                self.emit(Opcode::Pop, &[0], stmt.token.line);
            }
            Statement::Let(stmt) => {
                // Defining the symbol before the value allows compiling
                // recursive functions that has reference to its own name.
                let depth = self.scopes[self.scope_index].scope_depth;
                let symbol = self.symtab.define(&stmt.name.value, depth);
                self.compile_let_stmt(stmt.value)?;

                // Use a Symbol's scope to emit the right instruction
                if symbol.scope == SymbolScope::Global {
                    self.emit(Opcode::DefineGlobal, &[symbol.index], stmt.token.line);
                } else {
                    self.emit(Opcode::DefineLocal, &[symbol.index], stmt.token.line);
                }
            }
            Statement::Return(stmt) => {
                if self.scope_index == 0 {
                    return Err(CompileError::new(
                        "return statement outside of function",
                        stmt.token.line,
                    ));
                }
                match stmt.value {
                    Some(expr) => {
                        self.compile_expression(expr)?;
                    }
                    None => {
                        // Empty return statement. Emit a Null
                        self.emit(Opcode::Null, &[0], stmt.token.line);
                    }
                }
                self.emit(Opcode::ReturnValue, &[0], stmt.token.line);
            }
            Statement::Loop(stmt) => {
                // Record the position of the beginning of the loop so a 'Jump'
                // instruction can be used to jump to the beginning of the loop
                // It also indicates that the compiler is compiling a loop
                let loop_begin = self.get_curr_instructions().len();
                // Push a new LoopLabel onto the loop stack.
                let loop_label = if let Some(label) = stmt.label {
                    LoopContext::new(Some(label.literal), loop_begin)
                } else {
                    LoopContext::new(None, loop_begin)
                };
                self.scopes[self.scope_index].loop_stack.push(loop_label);
                // Compile the body of the loop
                self.compile_block_statement(stmt.body)?;
                // Instruction to jump to beginning of the loop
                self.emit(Opcode::Jump, &[loop_begin], stmt.token.line);

                // Pop the current loop label off the loop stack
                if let Some(loop_curr) = self.scopes[self.scope_index].loop_stack.pop() {
                    // Patch all the anonymous 'break' instructions
                    let break_pos = loop_curr.break_positions;
                    for pos in break_pos.iter() {
                        self.patch_jump(*pos);
                    }
                }
            }
            Statement::While(stmt) => {
                // Record the position of the beginning of the loop so a 'Jump'
                let loop_begin = self.get_curr_instructions().len();
                // Push a new LoopLabel onto the loop stack.
                let loop_label = if let Some(label) = stmt.label {
                    LoopContext::new(Some(label.literal), loop_begin)
                } else {
                    LoopContext::new(None, loop_begin)
                };
                self.scopes[self.scope_index].loop_stack.push(loop_label);

                // Compile the condition expression
                self.compile_expression(stmt.condition)?;
                // Jump to end of loop if false
                let condition_pos = self.emit(Opcode::JumpIfFalse, &[0xFFFF], stmt.token.line);
                // Compile the body of the loop
                self.compile_block_statement(stmt.body)?;

                // Instruction to jump to beginning of the loop
                self.emit(Opcode::Jump, &[loop_begin], stmt.token.line);
                // patch the jump to end of the loop
                self.patch_jump(condition_pos);

                // Pop the current loop label off the loop stack
                if let Some(loop_curr) = self.scopes[self.scope_index].loop_stack.pop() {
                    // Patch all the anonymous 'break' instructions
                    let break_pos = loop_curr.break_positions;
                    for pos in break_pos.iter() {
                        self.patch_jump(*pos);
                    }
                }
            }
            Statement::Break(stmt) => {
                // If loop stack is empty, the control is outside a loop
                if self.scopes[self.scope_index].loop_stack.is_empty() {
                    return Err(CompileError::new(
                        "break statement outside of loop",
                        stmt.token.line,
                    ));
                } else {
                    // Placeholder instruction to jump to end of the loop
                    let pos = self.emit(Opcode::Jump, &[0xFFFF], stmt.token.line);

                    // Save the position of the 'break' instruction so it can be patched later
                    // Add the break position to the current inner most loop
                    if let Some(label) = stmt.label {
                        // Labeled break statements. Find the loop label and add the break position
                        let loop_stack = &mut self.scopes[self.scope_index].loop_stack;
                        for loop_label in loop_stack.iter_mut().rev() {
                            if let Some(loop_label_name) = &loop_label.label {
                                if loop_label_name == &label.literal {
                                    loop_label.break_positions.push(pos);
                                    return Ok(());
                                }
                            }
                        }
                        return Err(CompileError::new(
                            &format!("unknown loop label '{}'", label.literal),
                            stmt.token.line,
                        ));
                    } else {
                        // Anonymous break statements
                        if let Some(last) = self.scopes[self.scope_index].loop_stack.last_mut() {
                            last.break_positions.push(pos);
                        }
                    }
                }
            }
            Statement::Continue(stmt) => {
                // If loop stack is empty, the control is outside a loop
                if self.scopes[self.scope_index].loop_stack.is_empty() {
                    return Err(CompileError::new(
                        "continue statement outside of loop",
                        stmt.token.line,
                    ));
                } else {
                    // Save the position of the 'break' instruction so it can be patched later
                    // Add the break position to the current inner most loop
                    if let Some(label) = stmt.label {
                        // Labeled continue statements. Find the loop label and add the break position
                        let loop_stack = &self.scopes[self.scope_index].loop_stack;
                        for loop_label in loop_stack.iter().rev() {
                            if let Some(loop_label_name) = &loop_label.label {
                                if loop_label_name == &label.literal {
                                    self.emit(Opcode::Jump, &[loop_label.begin], stmt.token.line);
                                    return Ok(());
                                }
                            }
                        }
                        return Err(CompileError::new(
                            &format!("unknown loop label '{}'", label.literal),
                            stmt.token.line,
                        ));
                    } else {
                        // Anonymous continue statements.
                        // Emit a 'Jump' instruction to the beginning of the current loop
                        let loop_stack = &self.scopes[self.scope_index].loop_stack;
                        if let Some(loop_label) = loop_stack.last() {
                            self.emit(Opcode::Jump, &[loop_label.begin], stmt.token.line);
                        }
                    }
                }
            }
            Statement::Function(func) => {
                // Defining the symbol before the value allows compiling
                // recursive functions that has reference to its own name.
                let depth = self.scopes[self.scope_index].scope_depth;
                let symbol = self.symtab.define(&func.name, depth);
                let line = func.token.line;
                self.compile_function_literal(func)?;

                // Use a Symbol's scope to emit the right instruction
                if symbol.scope == SymbolScope::Global {
                    self.emit(Opcode::DefineGlobal, &[symbol.index], line);
                } else {
                    self.emit(Opcode::DefineLocal, &[symbol.index], line);
                }
            }
            Statement::Invalid => {
                panic!("Invalid statement encountered");
            }
        }
        Ok(())
    }

    fn compile_expression(&mut self, expr: Expression) -> Result<(), CompileError> {
        match expr {
            Expression::Invalid => {}
            Expression::Null(null) => {
                self.emit(Opcode::Null, &[0], null.token.line);
            }
            Expression::Integer(num) => {
                let obj = Object::Integer(num.value);
                let idx = self.add_constant(obj);
                self.emit(Opcode::Constant, &[idx], num.token.line);
            }
            Expression::Float(num) => {
                let obj = Object::Float(num.value);
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
                match binary.operator.as_ref() {
                    "&&" => {
                        self.compile_logical_and(*binary.left, *binary.right, binary.token.line)?;
                    }
                    "||" => {
                        self.compile_logical_or(*binary.left, *binary.right, binary.token.line)?;
                    }
                    "<" | "<=" => {
                        // In case of '<' or '<=', re order the operands to reuse the '>' or '>='
                        self.compile_expression(*binary.right)?;
                        self.compile_expression(*binary.left)?;
                        self.compile_infix_expr(&binary.operator, binary.token.line)?;
                    }
                    _ => {
                        self.compile_expression(*binary.left)?;
                        self.compile_expression(*binary.right)?;
                        self.compile_infix_expr(&binary.operator, binary.token.line)?;
                    }
                }
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
                    "~" => {
                        self.emit(Opcode::Not, &[0], u.token.line);
                    }
                    _ => return Err(CompileError::new("invalid unary operator", u.token.line)),
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
                self.compile_if_expression(expr)?;
            }
            Expression::Ident(expr) => {
                self.compile_identifier(expr)?;
            }
            Expression::Index(expr) => {
                self.compile_index_expression(expr)?;
            }
            Expression::Assign(expr) => {
                // compile the expression on the right side of the assignment
                self.compile_expression(*expr.right)?;
                self.compile_expression(*expr.left)?;
            }
            Expression::Function(func) => {
                self.compile_function_literal(func)?;
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
            "%" => {
                self.emit(Opcode::Mod, &[0], line);
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
            ">=" | "<=" => {
                self.emit(Opcode::GreaterEq, &[0], line);
            }
            "&" => {
                self.emit(Opcode::And, &[0], line);
            }
            "|" => {
                self.emit(Opcode::Or, &[0], line);
            }
            "^" => {
                self.emit(Opcode::Xor, &[0], line);
            }
            "<<" => {
                self.emit(Opcode::ShiftLeft, &[0], line);
            }
            ">>" => {
                self.emit(Opcode::ShiftRight, &[0], line);
            }
            _ => return Err(CompileError::new("invalid binary operator", line)),
        }
        Ok(())
    }

    fn compile_let_stmt(&mut self, expr: Expression) -> Result<Object, CompileError> {
        self.compile_expression(expr)?;
        Ok(Object::Null)
    }

    fn compile_if_expression(&mut self, expr: IfExpr) -> Result<(), CompileError> {
        self.compile_expression(*expr.condition)?;
        // Emit an 'JumpIfFalse' with a placeholder. Save it's position so it can be altered later
        // The target for this jump is the 'pop' instruction following the 'then' statement
        let jump_if_false_pos = self.emit(Opcode::JumpIfFalse, &[0xFFFF], expr.token.line);
        // JumpIfFalse consumes the result of 'condition'.
        let is_then_empty = expr.then_stmt.statements.is_empty();
        self.compile_block_statement(expr.then_stmt)?;
        // Get rid of the extra Pop that comes with the result of compiling 'then_stmt'
        // This is so that we don't loose the result of the 'if' expression
        if self.is_last_instruction(Opcode::Pop) {
            self.remove_last_pop();
        }
        if is_then_empty {
            // If 'then' statement is empty, then use a Null
            self.emit(Opcode::Null, &[0], expr.token.line);
        }

        // Emit an 'Jump' with a placeholder. Save it's position so it can be altered later
        // The target for this jump is the instruction following the 'else' statement
        let jump_pos = self.emit(Opcode::Jump, &[0xFFFF], expr.token.line);
        // Replace the operand of the placeholder 'JumpIfFalse' instruction with the
        // position of the instruction that comes after the 'then' statement
        self.patch_jump(jump_if_false_pos);

        // Look for an 'else' branch
        match expr.else_if {
            ElseIfExpr::Empty => {
                // Result of if expression when there is no 'else' branch
                self.emit(Opcode::Null, &[0], expr.token.line);
            }
            ElseIfExpr::Else(else_stmt) => {
                let is_else_empty = else_stmt.statements.is_empty();
                // TODO: Find line number of 'else_stmt'
                self.compile_block_statement(else_stmt)?;
                if self.is_last_instruction(Opcode::Pop) {
                    self.remove_last_pop();
                }
                if is_else_empty {
                    // If 'else' statement is empty, then use a Null
                    self.emit(Opcode::Null, &[0], expr.token.line);
                }
            }
            ElseIfExpr::ElseIf(else_if) => {
                // Compile the 'else_if' expression
                if let Expression::If(else_if) = *else_if {
                    self.compile_if_expression(else_if)?;
                } else {
                    return Err(CompileError::new(
                        "invalid else if expression",
                        expr.token.line,
                    ));
                }
            }
        }
        // change the operand of the Jump instruction to jump over the
        // else branch – it could be Null or a real 'else_stmt'
        self.patch_jump(jump_pos);
        Ok(())
    }

    fn compile_identifier(&mut self, expr: Identifier) -> Result<(), CompileError> {
        let depth = self.scopes[self.scope_index].scope_depth;
        if let Some(symbol) = self.symtab.resolve(&expr.token.literal, depth) {
            match expr.access {
                AccessType::Get => {
                    self.load_symbol(symbol, expr.token.line);
                }
                AccessType::Set => {
                    self.save_symbol(symbol, expr.token.line)?;
                }
            }
        } else {
            return Err(CompileError::new(
                &format!("undefined identifier '{}'", expr.token.literal),
                expr.token.line,
            ));
        }
        Ok(())
    }

    fn compile_index_expression(&mut self, expr: IndexExpr) -> Result<(), CompileError> {
        // Compile the expression being indexed
        self.compile_expression(*expr.left)?;
        // Compile the index expression
        self.compile_expression(*expr.index)?;
        // Emit the index operator
        match expr.access {
            AccessType::Get => {
                self.emit(Opcode::GetIndex, &[], expr.token.line);
            }
            AccessType::Set => {
                self.emit(Opcode::SetIndex, &[], expr.token.line);
            }
        }
        Ok(())
    }

    fn compile_function_literal(&mut self, func: FunctionLiteral) -> Result<(), CompileError> {
        // enter scope of a function
        self.enter_scope();

        if !func.name.is_empty() {
            self.symtab.define_function_name(&func.name);
        }

        // Tell the compiler to turn the local references to the function
        // parameters into OpGetLocal instructions that load the arguments
        // onto the stack. Since these definitions are done in the scope of
        // the newly compiled function, they become part of the local
        // variables (num_locals) of the function.
        let num_params = func.params.len();
        for p in func.params {
            self.symtab.define(&p.value, 0);
        }
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
        // Make sure to also load free variables on to the stack after
        // compiling the function so they are accessible to 'OpClosure'.
        let num_locals = self.symtab.get_num_definitions();
        // It is important to get the free symbols before leaving the scope
        let free_symbols = self.symtab.free_symbols.clone();
        let instructions = self.leave_scope();

        // load free symbols on stack
        for f in &free_symbols {
            self.load_symbol(f.clone(), func.token.line);
        }
        let compiled_fn = Object::Func(Rc::new(CompiledFunction::new(
            instructions,
            num_locals,
            num_params,
        )));
        let idx = self.add_constant(compiled_fn);
        // emit closure instruction with the index to the compiled fn
        // and with number of free variables
        self.emit(Opcode::Closure, &[idx, free_symbols.len()], func.token.line);
        Ok(())
    }

    // The the left-hand side expression is compiled first. That means, at
    // runtime, its value will be on top of the stack. If that value is falsey,
    // then the entire expression must be false and so the right-hand side is
    // not evaluated at all. Otherwise, if lhs is truthy, the discard the value
    // of the lhs expression and evaluate the rhs expression and the result
    // becomes the value of the entire 'and' expression.
    //
    // Control Flow:
    // left operand expression
    // JumpIfFalseNoPop     ------+
    // OpPop                      |
    // right operand expression   |
    // continue            <------+
    //
    fn compile_logical_and(
        &mut self,
        left: Expression,
        right: Expression,
        line: usize,
    ) -> Result<(), CompileError> {
        self.compile_expression(left)?;
        // Emit an 'JumpIfFalseNoPop' with a placeholder. Save it's position so it can be altered later
        // Jump over the right hand side expression if the left hand side is false
        let jump_if_false_pos = self.emit(Opcode::JumpIfFalseNoPop, &[0xFFFF], line);
        // In the case of a 'false' result, we want to keep the result of the
        // left-hand side as the result of the entire '&&' expression.
        // So don't pop it. But inf the case of a 'true' result we do not want
        // the result of the left-hand side expression on the stack. So pop it.
        self.emit(Opcode::Pop, &[0], line);
        // If the result is true, then right hand side gets evaluated
        self.compile_expression(right)?;
        // Replace the operand of the placeholder 'JumpIfFalseNoPop' instruction with the
        // position of the instruction that comes after the '&&' expression
        self.patch_jump(jump_if_false_pos);
        Ok(())
    }

    // The the left-hand side expression is compiled first. That means, at
    // runtime, its value will be on top of the stack. If that value is truthy,
    // then the entire expression must be true and so the right-hand side is
    // not evaluated at all. Otherwise, if lhs is falsey, the discard the value
    // of the lhs expression and evaluate the rhs expression and the result
    // becomes the value of the entire 'or' expression.
    //
    // Control Flow:
    // left operand expression
    // JumpIfFalseNoPop   --------+
    // OpJump   ------------------|---+
    // OpPop              <-------+   |
    // right operand expression       |
    // continue           <-----------+
    //
    fn compile_logical_or(
        &mut self,
        left: Expression,
        right: Expression,
        line: usize,
    ) -> Result<(), CompileError> {
        self.compile_expression(left)?;
        // If lhs is false, jump to the rhs expression to evaluate that
        let rhs_pos = self.emit(Opcode::JumpIfFalseNoPop, &[0xFFFF], line);
        // If true, then use the result on the stack as the value of the entire expression
        // Jump over to the end of the expression since we have the value we need
        let end_pos = self.emit(Opcode::Jump, &[0xFFFF], line);
        // Patch the 'JumpIfFalseNoPop' instruction
        self.patch_jump(rhs_pos);
        // pop result of lhs since it is false; now, rhs needs to be evaluated.
        self.emit(Opcode::Pop, &[0], line);
        // If the result is true, then right hand side gets evaluated
        self.compile_expression(right)?;
        // Patch the Jump instruction
        self.patch_jump(end_pos);
        Ok(())
    }
}
