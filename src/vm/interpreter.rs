use byteorder::BigEndian;
use byteorder::ByteOrder;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::builtins::functions::BUILTINFNS;
use crate::builtins::pcap::PcapPacket;
use crate::builtins::variables::BuiltinVarType;
use crate::code::opcode::Opcode;
use crate::compiler::Bytecode;
use crate::object::array::Array;
use crate::object::func::BuiltinFunction;
use crate::object::func::Closure;
use crate::object::func::CompiledFunction;
use crate::object::hmap::HMap;
use crate::object::Object;
use crate::vm::error::RTError;
use crate::vm::frame::Frame;
use crate::vm::pktprop::MAX_PROTO_DEPTH;

const STACK_SIZE: usize = 4096;
const MAX_FRAMES: usize = 4096;
pub const GLOBALS_SIZE: usize = 65536;
pub const BUILTINS_SIZE: usize = 256;

/*
 * The virtual machine has the constants and instructions generated by the
 * compiler and has a stack. The stack pointer always points to the next
 * available free slot. So, the top of stack is stack[len - 1]. stack pointer
 * is assumed to be '0' when stack is empt and stack_top() would return Null.
 */
pub struct VM {
    constants: Vec<Rc<Object>>,
    stack: Vec<Rc<Object>>,
    sp: usize,
    pub globals: Vec<Rc<Object>>,
    pub builtinvars: RefCell<Vec<Rc<Object>>>,
    frames: Vec<Frame>,
    frames_index: usize,
    curr_pkt: RefCell<Option<Rc<Object>>>,
}

enum BinaryOperation {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Relational,
}

impl VM {
    pub fn new(bytecode: Bytecode) -> VM {
        let data = Rc::new(Object::Null);
        let fn_main = Rc::new(CompiledFunction::new(bytecode.instructions, 0, 0, 0));
        let closure_m: Rc<Closure> = Rc::new(Closure::new(fn_main, Vec::new()));
        let frame_e = Frame::default();
        let frame_m = Frame::new(closure_m, 0);
        let mut frames = vec![frame_e; MAX_FRAMES];
        frames[0] = frame_m;

        VM {
            constants: bytecode.constants,
            stack: vec![data.clone(); STACK_SIZE],
            sp: 0,
            globals: vec![data.clone(); GLOBALS_SIZE],
            builtinvars: RefCell::new(vec![data; BUILTINS_SIZE]),
            frames,
            frames_index: 1,
            curr_pkt: RefCell::new(None),
        }
    }

    pub fn new_with_global_store(bytecode: Bytecode, globals: Vec<Rc<Object>>) -> VM {
        let mut vm = VM::new(bytecode);
        vm.globals = globals;
        vm
    }

    // peek element from the top and return Null if underflow
    pub fn peek(&self, distance: usize) -> Rc<Object> {
        if self.sp - distance == 0 {
            Rc::new(Object::Null)
        } else {
            Rc::clone(&self.stack[self.sp - distance - 1])
        }
    }

    // peek element from the top and return error if underflow
    pub fn top(&self, distance: usize, line: usize) -> Result<Rc<Object>, RTError> {
        if self.sp - distance == 0 {
            Err(RTError::new("Stack underflow!", line))
        } else {
            Ok(Rc::clone(&self.stack[self.sp - distance - 1]))
        }
    }

    /*
     * If there isn't enough space on stack then push elements on to it
     * Otherwise, set the element on stack based on the stack pointer (sp).
     * In either case, increment 'sp' to point to the newly available slot.
     */
    pub fn push(&mut self, obj: Rc<Object>, line: usize) -> Result<(), RTError> {
        if self.sp >= self.stack.len() {
            return Err(RTError::new("Stack overflow!", line));
        } else {
            self.stack[self.sp] = obj;
        }
        self.sp += 1;
        Ok(())
    }

    pub fn pop(&mut self, line: usize) -> Result<Rc<Object>, RTError> {
        if self.sp == 0 {
            return Err(RTError::new("Stack underflow!", line));
        }
        let obj = self.stack[self.sp - 1].clone();
        self.sp -= 1;
        Ok(obj)
    }

    pub fn last_popped(&mut self) -> Rc<Object> {
        self.stack[self.sp].clone()
    }

    pub fn current_frame(&mut self) -> &mut Frame {
        &mut self.frames[self.frames_index - 1]
    }

    pub fn push_frame(&mut self, f: Frame) {
        self.frames[self.frames_index] = f;
        self.frames_index += 1;
    }

    pub fn pop_frame(&mut self) -> Frame {
        self.frames_index -= 1;
        self.frames[self.frames_index].clone()
    }

    #[allow(dead_code)]
    pub fn print_stack(&self) {
        eprintln!(
            "------------ Stack [sp: {:<4}, bp:{:<4}] ---------------",
            self.sp,
            self.frames[self.frames_index - 1].bp,
        );
        if self.sp == 0 {
            eprintln!("[<empty>]");
            return;
        }
        for i in 0..self.sp {
            eprint!("[ ");
            eprint!("{}", self.stack[i]);
            eprint!(" ]");
        }
        eprintln!();
    }

    /*
     * The main run loop for the interpreter. Since this is the hot path,
     * do not use functions such as lookup() or read_operands() for decoding
     * instructions and operands.
     */
    pub fn run(&mut self) -> Result<(), RTError> {
        while self.current_frame().ip < self.current_frame().instructions().len() {
            // Helpers
            let ip = self.current_frame().ip;
            let instructions = self.current_frame().instructions().clone();

            #[cfg(feature = "debug_trace_execution")]
            {
                self.print_stack();
                instructions.print(ip);
            }

            let op = Opcode::from(instructions.code[ip]);
            let line = instructions.lines[ip];
            match op {
                Opcode::Constant => {
                    let const_index =
                        BigEndian::read_u16(&instructions.code[ip + 1..ip + 3]) as usize;
                    let constant = self.constants.get(const_index).ok_or_else(|| {
                        RTError::new(&format!("constant not found [idx: {}]", const_index), line)
                    })?;
                    self.push(constant.clone(), line)?;
                    // skip over the two bytes of the operand in the next cycle
                    self.current_frame().ip += 2;
                }
                Opcode::Pop => {
                    self.pop(line)?;
                }
                Opcode::Add => {
                    self.binary_op(BinaryOperation::Add, |a, b| a + b, line)?;
                }
                Opcode::Sub => {
                    self.binary_op(BinaryOperation::Sub, |a, b| a - b, line)?;
                }
                Opcode::Mul => {
                    self.binary_op(BinaryOperation::Mul, |a, b| a * b, line)?;
                }
                Opcode::Div => {
                    self.binary_op(BinaryOperation::Div, |a, b| a / b, line)?;
                }
                Opcode::Mod => {
                    self.binary_op(BinaryOperation::Mod, |a, b| a % b, line)?;
                }
                Opcode::True => self.push(Rc::new(Object::Bool(true)), line)?,
                Opcode::False => self.push(Rc::new(Object::Bool(false)), line)?,
                Opcode::Equal => {
                    let b = self.pop(line)?;
                    let a = self.pop(line)?;
                    self.push(Rc::new(Object::Bool(a.as_ref() == b.as_ref())), line)?;
                }
                Opcode::NotEqual => {
                    let b = self.pop(line)?;
                    let a = self.pop(line)?;
                    self.push(Rc::new(Object::Bool(a != b)), line)?;
                }
                Opcode::Greater => {
                    self.binary_op(
                        BinaryOperation::Relational,
                        |a, b| Object::Bool(a > b),
                        line,
                    )?;
                }
                Opcode::GreaterEq => {
                    self.binary_op(
                        BinaryOperation::Relational,
                        |a, b| Object::Bool(a >= b),
                        line,
                    )?;
                }
                Opcode::Minus => {
                    if !self.peek(0).is_number() {
                        return Err(RTError::new("bad operand type for unary '-'", line));
                    }
                    let obj = self.pop(line)?.clone();
                    let val = -&*obj;
                    self.push(Rc::new(val), line)?;
                }
                Opcode::Bang => {
                    // Logical not (!)
                    let obj = self.pop(line)?;
                    self.push(Rc::new(Object::Bool(obj.is_falsey())), line)?;
                }
                Opcode::Jump => {
                    let bytes = &instructions.code[ip + 1..ip + 3];
                    // decode the operand (jump address) right after the opcode
                    self.current_frame().ip = u16::from_be_bytes([bytes[0], bytes[1]]) as usize;
                    // Do not increment ip at the end of the loop since the
                    // control is transferred to a jump statement. This allows
                    // us to have statements such as 'loop {}' as the only
                    // statement in a program.
                    continue;
                }
                Opcode::JumpIfFalse => {
                    let bytes = &instructions.code[ip + 1..ip + 3];
                    // decode the operand (jump address) right after the opcode
                    let pos: usize = u16::from_be_bytes([bytes[0], bytes[1]]) as usize;
                    // skip over the two bytes of the operand in the next cycle
                    self.current_frame().ip += 2;
                    // Pop the condition off the stack as it is not used in if-else
                    let condition = self.pop(line)?;
                    // Jump if the condition is false but continue program
                    // execution otherwise.
                    if condition.is_falsey() {
                        self.current_frame().ip = pos;
                        // Do not increment ip since the control is being
                        // transferred to the jump address.
                        continue;
                    }
                }
                Opcode::JumpIfFalseNoPop => {
                    let bytes = &instructions.code[ip + 1..ip + 3];
                    // decode the operand (jump address) right after the opcode
                    let pos: usize = u16::from_be_bytes([bytes[0], bytes[1]]) as usize;
                    // skip over the two bytes of the operand in the next cycle
                    self.current_frame().ip += 2;
                    // Do not pop the condition off the stack as it is used
                    // by the logical operators '&&' and '||'
                    let condition = self.top(0, line)?;
                    if condition.is_falsey() {
                        self.current_frame().ip = pos;
                        // Do not increment ip since the control is being
                        // transferred to the jump address.
                        continue;
                    }
                }
                Opcode::Null => {
                    self.push(Rc::new(Object::Null), line)?;
                }
                Opcode::DefineGlobal => {
                    let bytes = &instructions.code[ip + 1..ip + 3];
                    // decode the operand (index to globals)
                    let globals_index: usize = u16::from_be_bytes([bytes[0], bytes[1]]) as usize;
                    self.current_frame().ip += 2;
                    self.globals[globals_index] = self.pop(line)?;
                }
                Opcode::GetGlobal => {
                    let bytes = &instructions.code[ip + 1..ip + 3];
                    // decode the operand (index to globals)
                    let globals_index: usize = u16::from_be_bytes([bytes[0], bytes[1]]) as usize;
                    self.current_frame().ip += 2;
                    self.push(self.globals[globals_index].clone(), line)?;
                }
                Opcode::SetGlobal => {
                    let bytes = &instructions.code[ip + 1..ip + 3];
                    // decode the operand (index to globals)
                    let globals_index: usize = u16::from_be_bytes([bytes[0], bytes[1]]) as usize;
                    self.current_frame().ip += 2;
                    // Use the element on top of the stack for the assignment
                    // but do not pop the element off the stack since the assigment
                    // expression also evaluates to the value that is assigned
                    self.globals[globals_index] = self.top(0, line)?;
                }
                Opcode::Array => {
                    // Read the first operand i.e. the number of array elements
                    let num_elements = BigEndian::read_u16(&instructions.code[ip + 1..]) as usize;
                    let elements = self.build_array(self.sp - num_elements, self.sp);
                    // pop 'num_elements' off the stack
                    self.sp -= num_elements;
                    // Push the array back onto the stack as an object
                    self.push(Rc::new(Object::Arr(Rc::new(Array::new(elements)))), line)?;
                    // skip over the two bytes of the operand in the next cycle
                    self.current_frame().ip += 2;
                }
                Opcode::Map => {
                    // Read the first operand i.e. the number of pairs
                    let num_elements =
                        BigEndian::read_u16(&instructions.code[ip + 1..ip + 3]) as usize;
                    let pairs = self.build_map(self.sp - num_elements, self.sp, line)?;
                    // pop 'num_elements' off the stack
                    self.sp -= num_elements;
                    // Push the array back onto the stack as an object
                    self.push(Rc::new(Object::Map(Rc::new(HMap::new(pairs)))), line)?;
                    // skip over the two bytes of the operand in the next cycle
                    self.current_frame().ip += 2;
                }
                Opcode::Call => {
                    let num_args = instructions.code[ip + 1] as usize;
                    self.exec_call(num_args, line)?;
                    // Do not increment ip here since the vm is using a new frame
                    // and 'ip' should point to the first instruction in that frame
                    continue;
                }
                Opcode::ReturnValue => {
                    let ret_val = self.pop(line)?;
                    let frame = self.pop_frame();
                    // Reset stack frame by popping the local bindings and the
                    // the compiled function (the '-1' is for the compled function)
                    // Since the callee's frame has already been popped, the 'ip' used here
                    // refers to the caller's frame. So, do not increment that at the
                    // end of this loop and 'continue' immediately.
                    self.sp = frame.bp - 1;
                    self.push(ret_val, line)?;
                    continue;
                }
                Opcode::Return => {
                    // There is no return value to pop
                    let frame = self.pop_frame();
                    // Reset stack frame by popping the local bindings and the
                    // the compiled function (the '-1' is for the compled function)
                    self.sp = frame.bp - 1;
                    self.push(Rc::new(Object::Null), line)?;
                    // continue for the same reason as that of 'OpReturnValue'
                    continue;
                }
                Opcode::GetIndex => {
                    // The top most element on the stack is the index expression
                    // The next element is the expression itself.
                    let index = self.pop(line)?;
                    let left = self.pop(line)?;
                    self.exec_index_expr(left, index, None, line)?;
                }
                Opcode::SetIndex => {
                    // The top most element on the stack is the index expression
                    // The next element is the expression itself. The value to be
                    // set is further down the stack.
                    let index = self.pop(line)?;
                    let left = self.pop(line)?;
                    let setval = Some(self.pop(line)?);
                    self.exec_index_expr(left, index, setval, line)?;
                }
                Opcode::DefineLocal => {
                    // decode the operand (index to locals)
                    let locals_index = instructions.code[ip + 1] as usize;
                    self.current_frame().ip += 1;
                    let bp = self.current_frame().bp;
                    // Create the local binding
                    self.stack[bp + locals_index] = self.pop(line)?;
                }
                Opcode::GetLocal => {
                    // decode the operand (index to locals)
                    let locals_index = instructions.code[ip + 1] as usize;
                    self.current_frame().ip += 1;
                    let bp = self.current_frame().bp;
                    let obj = self.stack[bp + locals_index].clone();
                    self.push(obj, line)?;
                }
                Opcode::SetLocal => {
                    // decode the operand (index to locals)
                    let locals_index = instructions.code[ip + 1] as usize;
                    self.current_frame().ip += 1;
                    let bp = self.current_frame().bp;
                    // Use the element on top of the stack for the assignment
                    // but do not pop the element off the stack since the assigment
                    // expression also evaluates to the value that is assigned
                    self.stack[bp + locals_index] = self.top(0, line)?;
                }
                Opcode::GetBuiltinFn => {
                    // decode the operand (index to built-in functions)
                    let builtin_index = instructions.code[ip + 1] as usize;
                    self.current_frame().ip += 1;
                    if let Some(bt) = BUILTINFNS.get(builtin_index) {
                        // let builtin_func = bt.func;
                        self.push(Rc::new(Object::Builtin(Rc::new(bt.clone()))), line)?;
                    }
                }
                Opcode::GetBuiltinVar => {
                    let builtin_index = instructions.code[ip + 1] as usize;
                    self.current_frame().ip += 1;
                    let obj = self.builtinvars.borrow()[builtin_index].clone();
                    self.push(obj, line)?;
                }
                Opcode::Closure => {
                    // Decode first operand (index to closure in the constant pool)
                    let const_idx =
                        BigEndian::read_u16(&instructions.code[ip + 1..ip + 3]) as usize;
                    // Decode second operand (number of free varaibles)
                    let num_free = instructions.code[ip + 3] as usize;
                    // push the compiled function as a closure on stack
                    self.push_closure(const_idx, num_free, line)?;
                    self.current_frame().ip += 3;
                }
                Opcode::GetFree => {
                    let free_idx = instructions.code[ip + 1] as usize;
                    self.current_frame().ip += 1;
                    let curr_closure = self.current_frame().closure.clone();
                    self.push(curr_closure.free.borrow()[free_idx].clone(), line)?;
                }
                Opcode::SetFree => {
                    let free_idx = instructions.code[ip + 1] as usize;
                    self.current_frame().ip += 1;
                    let curr_closure = self.current_frame().closure.clone();
                    // Use the element on top of the stack for the assignment
                    // but do not pop the element off the stack since the assigment
                    // expression also evaluates to the value that is assigned
                    curr_closure.free.borrow_mut()[free_idx] = self.top(0, line)?;
                }
                Opcode::CurrClosure => {
                    let curr_closure = self.current_frame().closure.clone();
                    // push the current closure on stack
                    self.push(Rc::new(Object::Clos(curr_closure)), line)?;
                }
                Opcode::Not => {
                    // Bitwise not (~)
                    let obj = self.pop(line)?.clone();
                    if let Object::Integer(n) = &*obj {
                        self.push(Rc::new(Object::Integer(!n)), line)?;
                    } else {
                        return Err(RTError::new("bad operand type for unary '~'", line));
                    }
                }
                Opcode::And => {
                    self.bitwise_op(|a, b| a & b, line)?;
                }
                Opcode::Or => {
                    self.bitwise_op(|a, b| a | b, line)?;
                }
                Opcode::Xor => {
                    self.bitwise_op(|a, b| a ^ b, line)?;
                }
                Opcode::ShiftLeft => {
                    self.bitwise_op(|a, b| a << b, line)?;
                }
                Opcode::ShiftRight => {
                    self.bitwise_op(|a, b| a >> b, line)?;
                }
                Opcode::Dup => {
                    // Duplicate the top most element on the stack
                    let obj = self.peek(0);
                    self.push(obj, line)?;
                }
                Opcode::GetProp => {
                    // The property expression is the operand to this opcode.
                    // The stack contains the packet expression
                    let prop = instructions.code[ip + 1];
                    let left = self.pop(line)?;
                    let obj = self.exec_prop_expr(left, prop, None, line)?;
                    self.push(obj, line)?;
                    self.current_frame().ip += 1;
                }
                Opcode::SetProp => {
                    let prop = instructions.code[ip + 1];
                    let obj = self.pop(line)?.clone();
                    let left = self.pop(line)?.clone();
                    let obj = self.exec_prop_expr(left, prop, Some(obj), line)?;
                    self.push(obj, line)?;
                    self.current_frame().ip += 1;
                }
                Opcode::Dollar => {
                    self.exec_dollar_expr(line)?;
                }
                Opcode::Invalid => {
                    return Err(RTError::new(
                        &format!("opcode {} undefined", op as u8),
                        line,
                    ))
                }
            }
            self.current_frame().ip += 1;
        }

        Ok(())
    }

    fn binary_op(
        &mut self,
        optype: BinaryOperation,
        op: fn(a: &Object, b: &Object) -> Object,
        line: usize,
    ) -> Result<(), RTError> {
        // pop right before left
        let right = self.pop(line)?;
        let left = self.pop(line)?;

        match (&*left, &*right) {
            (
                Object::Integer(_) | Object::Float(_) | Object::Byte(_),
                Object::Integer(_) | Object::Float(_) | Object::Byte(_),
            ) => {
                if matches!(optype, BinaryOperation::Div) && right.is_zero() {
                    return Err(RTError::new("Division by zero.", line));
                }
                self.push(Rc::new(op(&left, &right)), line)
            }
            (Object::Str(s1), Object::Str(s2)) => match optype {
                BinaryOperation::Add => {
                    self.push(Rc::new(Object::Str(format!("{}{}", s1, s2))), line)
                }
                BinaryOperation::Relational => self.push(Rc::new(op(&left, &right)), line),
                _ => Err(RTError::new("Invalid operation on strings.", line)),
            },
            (Object::Char(c1), Object::Char(c2)) => match optype {
                BinaryOperation::Add => {
                    self.push(Rc::new(Object::Str(format!("{}{}", c1, c2))), line)
                }
                BinaryOperation::Relational => self.push(Rc::new(op(&left, &right)), line),
                _ => Err(RTError::new("Invalid operation on chars.", line)),
            },
            (Object::Str(s), Object::Integer(n)) | (Object::Integer(n), Object::Str(s)) => {
                if matches!(optype, BinaryOperation::Mul) {
                    self.push(Rc::new(Object::Str(s.repeat(*n as usize))), line)
                } else {
                    Err(RTError::new("Invalid operation on strings.", line))
                }
            }
            (Object::Arr(a), Object::Arr(b)) => {
                let mut e1 = a.elements.borrow().clone();
                let e2 = b.elements.borrow().clone();
                e1.extend_from_slice(&e2);
                self.push(Rc::new(Object::Arr(Rc::new(Array::new(e1)))), line)
            }
            _ => Err(RTError::new("Invalid binary operation.", line)),
        }
    }

    fn bitwise_op(
        &mut self,
        op: fn(a: &Object, b: &Object) -> Object,
        line: usize,
    ) -> Result<(), RTError> {
        // pop right before left
        let right = self.pop(line)?;
        let left = self.pop(line)?;

        match (&*left, &*right) {
            (Object::Integer(_), Object::Integer(_)) => {
                self.push(Rc::new(op(&left, &right)), line)?;
                Ok(())
            }
            _ => Err(RTError::new("Invalid bitwise operation.", line)),
        }
    }

    // Build array from elements on stack
    fn build_array(&self, start_index: usize, end_index: usize) -> Vec<Rc<Object>> {
        let mut elements = Vec::with_capacity(end_index - start_index);
        for i in start_index..end_index {
            elements.push(self.stack[i].clone());
        }
        elements
    }

    // Build map from objects on stack
    fn build_map(
        &self,
        start_index: usize,
        end_index: usize,
        line: usize,
    ) -> Result<HashMap<Rc<Object>, Rc<Object>>, RTError> {
        let mut elements = HashMap::with_capacity(end_index - start_index);
        for i in (start_index..end_index).step_by(2) {
            let key = self.stack[i].clone();
            if !key.is_a_valid_key() {
                return Err(RTError::new(
                    &format!("KeyError: not a valid key: {}.", key),
                    line,
                ));
            }
            let val = self.stack[i + 1].clone();
            elements.insert(key, val);
        }
        Ok(elements)
    }

    // Common code for indexing into arrays and maps
    fn exec_index_expr(
        &mut self,
        left: Rc<Object>,
        index: Rc<Object>,
        setval: Option<Rc<Object>>,
        line: usize,
    ) -> Result<(), RTError> {
        let obj = match (&*left, &*index) {
            (Object::Arr(arr), Object::Integer(idx)) => {
                self.exec_array_index(arr, *idx, setval, line)
            }
            (Object::Map(map), _) => self.exec_hash_index(map, &index, setval, line),
            _ => Err(RTError::new("IndexError: unsupported operation.", line)),
        };
        // Push the value onto the stack so it is available to
        // the expression statement that uses the index expression
        self.push(obj?, line)
    }

    fn exec_array_index(
        &mut self,
        arr: &Array,
        idx: i64,
        setval: Option<Rc<Object>>,
        line: usize,
    ) -> Result<Rc<Object>, RTError> {
        if idx < 0 {
            return Err(RTError::new("IndexError: index cannot be negative.", line));
        } else if idx as usize >= arr.len() {
            return Err(RTError::new("IndexError: array index out of range.", line));
        }

        // If it is a SetIndex operation, then set the value at the index
        let obj = if let Some(obj) = setval {
            arr.set(idx as usize, obj.clone());
            obj
        } else {
            arr.get(idx as usize)
        };
        // return the value being 'set' or 'get'
        Ok(obj)
    }

    fn exec_hash_index(
        &mut self,
        map: &HMap,
        key: &Rc<Object>,
        setval: Option<Rc<Object>>,
        line: usize,
    ) -> Result<Rc<Object>, RTError> {
        if !key.is_a_valid_key() {
            return Err(RTError::new(
                &format!("KeyError: not a valid key: {}.", key),
                line,
            ));
        }
        // If it is a SetIndex operation, then set the value at the index
        // SetIndex operation for a map does ntot require that the key
        // if present in the map already.
        let obj = if let Some(obj) = setval {
            map.insert(key.clone(), obj.clone());
            obj
        } else {
            let obj = map.get(key);
            if obj.is_null() {
                return Err(RTError::new("KeyError: key not found.", line));
            }
            obj
        };
        // return the value being 'set' or 'get'
        Ok(obj)
    }

    fn exec_call(&mut self, num_args: usize, line: usize) -> Result<(), RTError> {
        // Calculate the location of the function on the stack by decoding
        // the operand, 'num_args', and subtracting it from 'sp'. The additional
        // '-1' is there because 'sp' points to the next free slot on the stack.
        let callee = self.stack[self.sp - 1 - num_args].clone();

        match &*callee {
            Object::Clos(closure) => {
                self.call_func(closure, num_args, line)?;
            }
            Object::Builtin(builtin) => {
                self.call_builtin(builtin, num_args, line)?;
            }
            _ => {
                return Err(RTError::new("calling non-function", line));
            }
        }
        Ok(())
    }

    // The stack during the execution of a function call
    // looks like the following:
    //                                  <<------ sp
    //       <local var 2>              <<------ bp + 4
    //       <local var 1>              <<------ bp + 3
    //       <arg 2>                    <<------ bp + 2
    //       <arg 1>                    <<------ bp + 1
    //       <compiled-function>        <<------ bp
    fn call_func(
        &mut self,
        closure: &Rc<Closure>,
        num_args: usize,
        line: usize,
    ) -> Result<(), RTError> {
        // Make sure that the right number of arguments is sitting on the stack
        if num_args != closure.func.num_params {
            return Err(RTError::new(
                &format!(
                    "wrong number of arguments: want={}, got={}",
                    closure.func.num_params, num_args
                ),
                line,
            ));
        }

        // Save the current stack pointer before calling a function
        // The base pointer 'bp' is further down the stack and points to
        // the first argument to the function.
        let bp = self.sp - num_args;
        let frame = Frame::new(closure.clone(), bp);

        // Allocate space for local bindings on stack starting at the base
        // pointer 'bp' with 'num_locals' slots on the stack. Note that the
        // parameters to the function are also part of the local bindings,
        // i.e. 'num_locals' is the sum of #locals and #arguments
        // In the example above, num_locals = args(2) + locals(2) = 4.
        self.sp = frame.bp + closure.func.num_locals;

        // skip over the instruction and the 1-byte operand to OpCall 'before'
        // pushing a new frame so that the callee's frame is not meddled with
        self.current_frame().ip += 2;
        self.push_frame(frame);
        Ok(())
    }

    fn call_builtin(
        &mut self,
        builtin: &BuiltinFunction,
        num_args: usize,
        line: usize,
    ) -> Result<(), RTError> {
        // copy arguments from the stack into a vector
        let args = self.stack[self.sp - num_args..self.sp].to_vec();
        let builtin_func = builtin.func;
        match builtin_func(args) {
            Ok(obj) => {
                // pop the arguments and the function
                self.sp = self.sp - num_args - 1;
                self.push(obj, line)?;
            }
            Err(s) => {
                // Prefix error messaage with the function name
                let msg = format!("{}: {}", builtin.name, s);
                return Err(RTError::new(&msg, 1));
            }
        }
        self.current_frame().ip += 2;
        Ok(())
    }

    // const_idx: Index of the compiled function in the constant pool
    // num_free: number of free variables waiting on the stack
    fn push_closure(
        &mut self,
        const_idx: usize,
        num_free: usize,
        line: usize,
    ) -> Result<(), RTError> {
        let constant = self.constants[const_idx].clone();
        if let Object::Func(function) = constant.as_ref() {
            let mut free = Vec::with_capacity(num_free);

            // Take each free variable from stack and copy it to 'free'
            // copy in the same order they are referenced in GetFree
            for i in 0..num_free {
                let idx = self.sp - num_free + i;
                free.push(self.stack[idx].clone());
            }
            // cleanup stack of free variables
            self.sp -= num_free;

            let closure = Rc::new(Closure::new(function.clone(), free));
            self.push(Rc::new(Object::Clos(closure)), line)
        } else {
            Err(RTError::new(
                &format!("not a function: {:?}", constant),
                line,
            ))
        }
    }

    /// Push a frame used to run the filter statement onto the stack.
    /// The frame expects a closure, so convert the compiled function into one.
    /// A closure here is simply a compiled function with a list of free variables.
    /// There are no free variables in a function that wraps a filter statement.
    /// num_locals represents the local variables used in the filter statement.
    /// Save the stack pointer (sp) in the frame so that it can be restored later.
    pub fn push_filter_frame(&mut self, filter: &Rc<CompiledFunction>) -> Result<(), RTError> {
        let bp = self.sp;
        let num_locals = filter.num_locals;
        let closure: Rc<Closure> = Rc::new(Closure::new(filter.clone(), Vec::new()));
        let frame = Frame::new(closure.clone(), bp);
        self.sp = frame.bp + num_locals;
        self.push_frame(frame);
        Ok(())
    }

    /// Pop the frame used to run the filter statement from the stack.
    /// This is done after the filter statement has been executed.
    /// Also restore the stack by popping the local bindings.
    pub fn pop_filter_frame(&mut self) -> Result<bool, RTError> {
        let frame = self.pop_frame();
        let line = frame.closure.func.line;
        // Pop the result of the filter statement. This is either pushed as
        // the result of evaluating the pattern or by the body of the custom
        // action statement. The custom action statement has a false statement
        // at the end.
        let obj = self.pop(line)?;
        // Reset stack frame by popping the local bindings
        self.sp = frame.bp;
        if let Object::Bool(b) = &*obj {
            Ok(*b)
        } else {
            Err(RTError::new(
                "filter expression must evaluate to a boolean",
                line,
            ))
        }
    }

    /// Set the current packet and the builtin variables
    pub fn set_curr_pkt(&self, pkt: Rc<PcapPacket>) {
        self.update_builtin_var(BuiltinVarType::PL, pkt.get_caplen());
        self.update_builtin_var(BuiltinVarType::WL, pkt.get_wirelen());
        self.update_builtin_var(BuiltinVarType::Tss, pkt.get_ts_sec());
        self.update_builtin_var(BuiltinVarType::Tsu, pkt.get_ts_usec());
        let obj = Object::Packet(pkt);
        self.curr_pkt.borrow_mut().replace(Rc::new(obj));
    }

    /// Evaluate expressions such as $0, $n etc
    /// The top of the stack contains the index of the dollar expression
    /// The stack is popped and the result of the dollar expression is pushed
    /// back onto the stack.
    fn exec_dollar_expr(&mut self, line: usize) -> Result<(), RTError> {
        // The stack contains the index of the dollar expression
        let obj = self.pop(line)?;
        let depth = match obj.as_ref() {
            Object::Integer(n) => *n as usize,
            _ => return Err(RTError::new("IndexError: invalid index.", line)),
        };

        let curr = self.curr_pkt.borrow().as_ref().cloned();
        let obj = if let Some(obj) = curr {
            if depth > MAX_PROTO_DEPTH {
                return Err(RTError::new(
                    &format!("IndexError: exceeded max protocol depth - '${}'", depth),
                    line,
                ));
            }
            self.get_inner(&obj, depth, line)?
        } else {
            Rc::new(Object::Null)
        };
        self.push(obj, line)?;
        Ok(())
    }

    pub fn update_builtin_var(&self, vt: BuiltinVarType, obj: Rc<Object>) {
        self.builtinvars.borrow_mut()[vt as usize] = obj;
    }
}
