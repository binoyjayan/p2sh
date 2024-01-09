use std::rc::Rc;

use self::symtab::Symbol;
use self::symtab::SymbolScope;
use crate::builtins::functions::BUILTINFNS;
use crate::builtins::variables::BuiltinVarType;
use crate::code::definitions::{self, *};
use crate::code::opcode::Opcode;
use crate::compiler::error::CompileError;
use crate::compiler::symtab::SymbolTable;
use crate::object::file::FileHandle;
use crate::object::func::CompiledFunction;
use crate::object::Object;
use crate::parser::ast::expr::*;
use crate::parser::ast::stmt::BlockStatement;
use crate::parser::ast::stmt::FilterPattern;
use crate::parser::ast::stmt::FilterStmt;
use crate::parser::ast::stmt::Statement;
use crate::parser::ast::*;

pub mod error;
pub mod symtab;
pub mod symtab_test;
pub mod tests;

pub struct Bytecode {
    pub instructions: Instructions,
    pub constants: Vec<Rc<Object>>,
    pub filters: Vec<Rc<CompiledFunction>>,
    pub filter_end: Option<Rc<CompiledFunction>>,
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
    pub filters: Vec<Rc<CompiledFunction>>,
    pub filter_end: Option<Rc<CompiledFunction>>,
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
            filters: Vec::new(),
            filter_end: None,
        }
    }

    pub fn new_with_state(symtab: SymbolTable, constants: Vec<Rc<Object>>) -> Compiler {
        let mut compiler = Self::new();
        compiler.constants = constants;
        compiler.symtab = symtab;
        compiler
    }

    /// Enter a local scope. This is used when compiling a function body.
    pub fn enter_scope(&mut self) {
        let scope = CompilationScope::default();
        self.scopes.push(scope);
        self.scope_index += 1;
        self.symtab = SymbolTable::new_enclosed(self.symtab.clone());
    }

    /// Leave a local scope used while compiling a function body.
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
        let filters = self.filters.clone();
        let filter_end = self.filter_end.clone();
        #[cfg(feature = "debug_print_code")]
        {
            instructions.disassemble();
            self.print_constants();
            self.print_filters();
        }
        Bytecode {
            instructions,
            constants,
            filters,
            filter_end,
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

    #[allow(dead_code)]
    fn print_filters(&self) {
        let filters = self.filters.clone();
        if filters.is_empty() {
            return;
        }
        println!(
            "------------- Filters [len: {:<4}] --------------------",
            filters.len(),
        );

        for (i, func) in filters.iter().enumerate() {
            println!("[{}] {}", i, func);
            func.instructions.disassemble();
        }
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
            Statement::Block(stmt) => {
                self.compile_block_statement(stmt)?;
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
            Statement::Filter(f) => {
                self.compile_filter_statement(f)?;
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
            Expression::Score(expr) => {
                return Err(CompileError::new(
                    "underscore is not supported here",
                    expr.token.line,
                ));
            }
            Expression::Null(null) => {
                self.emit(Opcode::Null, &[0], null.token.line);
            }
            Expression::Builtin(bid) => {
                let obj = match bid.value.as_str() {
                    "stdin" => Object::File(Rc::new(FileHandle::Stdin)),
                    "stdout" => Object::File(Rc::new(FileHandle::Stdout)),
                    "stderr" => Object::File(Rc::new(FileHandle::Stderr)),
                    _ => {
                        panic!("invalid builtin identifier {}", bid.token.line);
                    }
                };

                let idx = self.add_constant(obj);
                self.emit(Opcode::Constant, &[idx], bid.token.line);
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
            Expression::Char(c) => {
                let obj = Object::Char(c.value);
                let idx = self.add_constant(obj);
                self.emit(Opcode::Constant, &[idx], c.token.line);
            }
            Expression::Byte(b) => {
                let obj = Object::Byte(b.value);
                let idx = self.add_constant(obj);
                self.emit(Opcode::Constant, &[idx], b.token.line);
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
                    "$" => {
                        self.emit(Opcode::Dollar, &[0], u.token.line);
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
            Expression::Match(expr) => {
                self.compile_match_expression(expr)?;
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
            Expression::Range(expr) => {
                // Range expressions not to be used here
                return Err(CompileError::new(
                    "range expression are not supported here",
                    expr.token.line,
                ));
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
            Expression::Dot(expr) => {
                self.compile_dot_expression(expr)?;
            }
            Expression::Prop(expr) => {
                self.compile_prop_expression(expr)?;
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
        // If the 'then' statement is empty, or if the last statement
        // does not produce a value, then emit a Null.
        let no_value = if let Some(last) = expr.then_stmt.statements.last() {
            !last.is_expression()
        } else {
            true
        };
        self.compile_block_statement(expr.then_stmt)?;
        // Get rid of the extra Pop that is emitted as a result of compiling 'then_stmt'.
        // This is so that we don't loose the result of the 'if' expression
        if self.is_last_instruction(Opcode::Pop) {
            self.remove_last_pop();
        }
        // If 'then' statement does not produce a value, then use a Null
        if no_value {
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
                let no_value = if let Some(last) = else_stmt.statements.last() {
                    !last.is_expression()
                } else {
                    true
                };
                // TODO: Find line number of 'else_stmt'
                self.compile_block_statement(else_stmt)?;
                if self.is_last_instruction(Opcode::Pop) {
                    self.remove_last_pop();
                }
                // If 'else' statement does not produce a value, then use a Null
                if no_value {
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

    fn compile_match_expression(&mut self, match_expr: MatchExpr) -> Result<(), CompileError> {
        // Jump vector to jump to the end of the match expression
        // for all patterns in all of the match arms
        let mut jump_end_v = Vec::new();
        // Compile the scrutinee expression. The result of this expression is
        // duplicated on the stack for each pattern variant in each arm of the
        // match expression. This is so that the result of the condition is
        // available for comparison until a match is found. It is popped just
        // before executing the block statement corresponding to the arm.
        self.compile_expression(*match_expr.expr)?;

        // The parser should have added atleast the default arm and
        // atleast one pattern in the arm. So it is safe to unwrap.
        let first = match_expr.arms.first().unwrap().patterns.first().unwrap();

        // MatchIfFalse consumes the result of 'condition'.
        // Compile each arm of the match expression
        for (idx, arm) in match_expr.arms.iter().enumerate() {
            // Jump vector to jump to the match arm body if there is a match
            // There is a jump instruction for every match arm variant.
            let mut jump_body_v = Vec::new();
            // Compile the patterns for this arm
            for pattern_variant in &arm.patterns {
                // compare type of the first pattern with all the other patterns
                if !first.matches_type(pattern_variant) {
                    return Err(CompileError::new(
                        "all patterns in the match expresion must have the same type",
                        arm.token.line,
                    ));
                }
                match pattern_variant {
                    MatchPattern::Boolean(b) => {
                        // Duplicate the scrutinee expression on the stack
                        self.emit(Opcode::Dup, &[0], arm.token.line);
                        // Push the boolean pattern variant onto the stack
                        if b.value {
                            self.emit(Opcode::True, &[0], b.token.line);
                        } else {
                            self.emit(Opcode::False, &[0], b.token.line);
                        }
                        // Compare with OpNotEqual (inverse of OpEqual)
                        self.emit(Opcode::NotEqual, &[0], b.token.line);
                        // If the result of OpNotEqual is false, i.e. If Equal,
                        // then jump to the block statement. Otherwise,
                        // continue to the next pattern variant
                        let jump_pos = self.emit(Opcode::JumpIfFalse, &[0xFFFF], b.token.line);
                        jump_body_v.push(jump_pos);
                    }
                    MatchPattern::Integer(num) => {
                        // Duplicate the scrutinee expression on the stack
                        self.emit(Opcode::Dup, &[0], arm.token.line);
                        // Push the integer pattern variant onto the stack
                        let idx = self.add_constant(Object::Integer(num.value));
                        self.emit(Opcode::Constant, &[idx], num.token.line);
                        // Compare with OpNotEqual (inverse of OpEqual)
                        self.emit(Opcode::NotEqual, &[0], num.token.line);
                        // If the result of OpNotEqual is false, i.e. If Equal,
                        // then jump to the block statement. Otherwise,
                        // continue to the next pattern variant
                        let jump_pos = self.emit(Opcode::JumpIfFalse, &[0xFFFF], num.token.line);
                        jump_body_v.push(jump_pos);
                    }
                    MatchPattern::Str(s) => {
                        self.emit(Opcode::Dup, &[0], arm.token.line);
                        let idx = self.add_constant(Object::Str(s.value.clone()));
                        self.emit(Opcode::Constant, &[idx], s.token.line);
                        self.emit(Opcode::NotEqual, &[0], s.token.line);
                        let jump_pos = self.emit(Opcode::JumpIfFalse, &[0xFFFF], s.token.line);
                        jump_body_v.push(jump_pos);
                    }
                    MatchPattern::Char(ch) => {
                        self.emit(Opcode::Dup, &[0], arm.token.line);
                        let idx = self.add_constant(Object::Char(ch.value));
                        self.emit(Opcode::Constant, &[idx], ch.token.line);
                        self.emit(Opcode::NotEqual, &[0], ch.token.line);
                        let jump_pos = self.emit(Opcode::JumpIfFalse, &[0xFFFF], ch.token.line);
                        jump_body_v.push(jump_pos);
                    }
                    MatchPattern::Byte(b) => {
                        self.emit(Opcode::Dup, &[0], arm.token.line);
                        let idx = self.add_constant(Object::Byte(b.value));
                        self.emit(Opcode::Constant, &[idx], b.token.line);
                        self.emit(Opcode::NotEqual, &[0], b.token.line);
                        let jump_pos = self.emit(Opcode::JumpIfFalse, &[0xFFFF], b.token.line);
                        jump_body_v.push(jump_pos);
                    }
                    MatchPattern::Range(r) => {
                        let (idx_beg, idx_end) = match (&*r.begin, &*r.end) {
                            (Expression::Integer(begin), Expression::Integer(end)) => (
                                self.add_constant(Object::Integer(begin.value)),
                                self.add_constant(Object::Integer(end.value)),
                            ),
                            (Expression::Str(begin), Expression::Str(end)) => (
                                self.add_constant(Object::Str(begin.value.clone())),
                                self.add_constant(Object::Str(end.value.clone())),
                            ),
                            (Expression::Char(begin), Expression::Char(end)) => (
                                self.add_constant(Object::Char(begin.value)),
                                self.add_constant(Object::Char(end.value)),
                            ),
                            (Expression::Byte(begin), Expression::Byte(end)) => (
                                self.add_constant(Object::Byte(begin.value)),
                                self.add_constant(Object::Byte(end.value)),
                            ),
                            _ => {
                                return Err(CompileError::new(
                                    "invalid range expression",
                                    r.token.line,
                                ));
                            }
                        };

                        // Compare the beginning part of the range
                        self.emit(Opcode::Dup, &[0], arm.token.line);
                        self.emit(Opcode::Constant, &[idx_beg], r.token.line);
                        // If GreaterEq is false i.e. value < begin, then goto end
                        self.emit(Opcode::GreaterEq, &[0], r.token.line);
                        let jump_end = self.emit(Opcode::JumpIfFalse, &[0xFFFF], r.token.line);

                        // Now compare the end range
                        self.emit(Opcode::Dup, &[0], arm.token.line);
                        self.emit(Opcode::Constant, &[idx_end], r.token.line);
                        // Check if the range is exclusive or inclusive
                        if r.operator == ".." {
                            // 'value >= end' is false implies value < end; goto the block stmt
                            self.emit(Opcode::GreaterEq, &[0], r.token.line);
                        } else {
                            // 'value > end' is false implies value <= end; goto the block stmt
                            self.emit(Opcode::Greater, &[0], r.token.line);
                        }
                        let jump_pos = self.emit(Opcode::JumpIfFalse, &[0xFFFF], r.token.line);
                        jump_body_v.push(jump_pos);
                        // If the value is not in the range, then do nothing (continue to next pattern)
                        self.patch_jump(jump_end);
                    }
                    MatchPattern::Default(u) => {
                        // The condition is always true; jump to the block statement
                        let jump_pos = self.emit(Opcode::Jump, &[0xFFFF], u.token.line);
                        jump_body_v.push(jump_pos);
                    }
                }
            }

            // If none of the pattern variants match, then jump to the next arm
            // or to the end of the match expression if this is the last arm
            let jump_over_body = self.emit(Opcode::Jump, &[0xFFFF], arm.token.line);

            // Update the jump instructions to jump to the block statement
            for jump_pos in jump_body_v {
                self.patch_jump(jump_pos);
            }

            // Pop the original scrutinee expression when there is a match
            self.emit(Opcode::Pop, &[0], arm.token.line);
            // Check if the match arm body produces a value
            let match_arm_body = arm.body.clone();
            let no_value = if let Some(last) = match_arm_body.statements.last() {
                !last.is_expression()
            } else {
                true
            };
            // Compile the block statement for this match arm
            self.compile_block_statement(match_arm_body)?;
            // Get rid of the extra Pop that is emitted as a result of
            // compiling 'arm.body'. This is so that we don't loose the result
            // of the 'match' expression
            if self.is_last_instruction(Opcode::Pop) {
                self.remove_last_pop();
            }
            // If 'then' statement does not produce a value, then use a Null
            if no_value {
                self.emit(Opcode::Null, &[0], arm.token.line);
            }

            // Jump to the end of the match expression after executing the block
            // statement for this arm. If this arm is the last one, the
            // instruction pointer is already at the end of the match expression.
            // Therefore, a jump is not required.
            if idx < match_expr.arms.len() - 1 {
                let jump_pos = self.emit(Opcode::Jump, &[0xFFFF], match_expr.token.line);
                jump_end_v.push(jump_pos);
            }

            // Patch the jump instruction to jump over the block statement
            self.patch_jump(jump_over_body);
        }
        // Update the jump instructions to jump to the end of the match expression
        for jump_pos in jump_end_v {
            self.patch_jump(jump_pos);
        }

        Ok(())
    }

    fn compile_identifier(&mut self, expr: Identifier) -> Result<(), CompileError> {
        let depth = self.scopes[self.scope_index].scope_depth;
        if let Some(symbol) = self.symtab.resolve(&expr.token.literal, depth) {
            match expr.context.access {
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
        match expr.context.access {
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
            func.token.line,
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

    fn compile_dot_expression(&mut self, expr: DotExpr) -> Result<(), CompileError> {
        // Compile the expression whose property is being accessed
        self.compile_expression(*expr.left)?;
        // Compile the property expression
        self.compile_expression(*expr.property)?;
        Ok(())
    }

    fn compile_prop_expression(&mut self, expr: PktPropExpr) -> Result<(), CompileError> {
        let val = expr.value as usize;
        // Emit the property opcode
        match expr.context.access {
            AccessType::Get => {
                self.emit(Opcode::GetProp, &[val], expr.token.line);
            }
            AccessType::Set => {
                self.emit(Opcode::SetProp, &[val], expr.token.line);
            }
        }
        Ok(())
    }

    /// Compile the filter statement in a different scope so that the bytecode
    /// for the filters and actions can be captured separately. This is done
    /// The compilation happens in the current scope and while leaving the scope
    /// the bytecode for the filter statement is captured and stored separately.
    fn compile_filter_statement(&mut self, expr: FilterStmt) -> Result<(), CompileError> {
        self.enter_scope();

        // If there is no filter pattern, and if it is not an 'end' pattern,
        // then the control flow executes the action statement unconditionally.
        // The absence of a pattern default to a true pattern.
        if let FilterPattern::Expr(filter) = expr.pattern.clone() {
            self.compile_expression(*filter)?;
        }

        // Emit an 'JumpIfFalseNoPop' with a placeholder. Save it's position so it can be altered later
        // The target for this jump is the 'pop' instruction following the 'action' statement
        // Do not pop the result of the filter since it is returned by the filter
        // statement when the action is 'None'. In this case the caller of the filter
        // statement is responsible for popping the result.
        if expr.pattern.is_none() || expr.pattern.is_end() {
            // Always execute the action if the filter pattern is 'end'
            // or if there is no filter pattern that defaults to true
            // Since a pattern was not evaulated, do not pop the result of the
            // pattern expression. So, pass 'false'.
            self.emit_action_stmt(expr.action, false, expr.token.line)?;
        } else {
            let jump_if_false_pos = self.emit(Opcode::JumpIfFalseNoPop, &[0xFFFF], expr.token.line);
            self.emit_action_stmt(expr.action, true, expr.token.line)?;
            // Replace the operand of the placeholder 'JumpIfFalse' instruction with the
            // position of the instruction that comes after the 'then' statement
            self.patch_jump(jump_if_false_pos);
        }
        // Get the number of locals and create the function
        let num_locals = self.symtab.get_num_definitions();
        let instructions = self.leave_scope();
        // There are not free variables for the function wrapping a filter
        // The filter statements are compiled as closures that takes no parameters
        let filter = Rc::new(CompiledFunction::new(
            instructions,
            num_locals,
            0,
            expr.token.line,
        ));

        // Add the filter to the list of filters except for the 'end' pattern
        if expr.pattern.is_end() {
            if self.filter_end.is_some() {
                return Err(CompileError::new(
                    "multiple 'end' patterns in filter statement",
                    expr.token.line,
                ));
            }
            self.filter_end = Some(filter);
        } else {
            self.filters.push(filter);
        }
        Ok(())
    }

    /// Emit the action statement for a filter statement. If the action is
    /// JumpIfFalseNoPop does not consume the result of 'filter'.
    // Do not pop the result of the filter action is None
    fn emit_action_stmt(
        &mut self,
        action: Option<BlockStatement>,
        pop: bool,
        line: usize,
    ) -> Result<(), CompileError> {
        if let Some(action) = action {
            // Consume the result of the filter pattern
            if pop {
                self.emit(Opcode::Pop, &[0], line);
            }
            self.compile_block_statement(action)?;
            // Emit false to indicate that no action needs to be performed by
            // the caller of the filter statement since it is already done here.
            self.emit(Opcode::False, &[0], line);
        }
        Ok(())
    }
}
