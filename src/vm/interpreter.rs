use byteorder::BigEndian;
use byteorder::ByteOrder;
use std::rc::Rc;

use crate::code::definitions::*;
use crate::code::opcode::Opcode;
use crate::common::error::RTError;
use crate::common::object::Object;

const STACK_SIZE: usize = 4096;

/*
 * The virtual machine has the constants and instructions generated by the
 * compiler and has a stack. The stack pointer always points to the next
 * available free slot. So, the top of stack is stack[len - 1]. stack pointer
 * is assumed to be '0' when stack is empt and stack_top() would return Nil.
 */
pub struct VM {
    constants: Vec<Object>,
    stack: Vec<Rc<Object>>,
    sp: usize,
}

#[allow(dead_code)]
enum OperandType {
    TwoNumbers,
    NumbersOrStrings,
}

impl VM {
    pub fn new(constants: Vec<Object>) -> VM {
        VM {
            constants,
            stack: Vec::with_capacity(STACK_SIZE),
            sp: 0,
        }
    }

    pub fn peek(&self, distance: usize) -> Rc<Object> {
        if self.sp - distance == 0 {
            Rc::new(Object::Nil)
        } else {
            Rc::clone(&self.stack[self.sp - 1])
        }
    }

    /*
     * If there isn't enough space on stack then push elements on to it
     * Otherwise, set the element on stack based on the stack pointer (sp).
     * In either case, increment 'sp' to point to the newly available slot.
     */
    pub fn push(&mut self, obj: Rc<Object>) {
        if self.sp >= self.stack.len() {
            self.stack.push(obj);
        } else {
            self.stack[self.sp] = obj;
        }
        self.sp += 1;
    }

    pub fn pop(&mut self) -> Rc<Object> {
        if self.sp == 0 {
            panic!("Stack underflow!");
        }
        let obj = self.stack[self.sp - 1].clone();
        self.sp -= 1;
        obj
    }

    pub fn last_popped(&mut self) -> Rc<Object> {
        self.stack[self.sp].clone()
    }

    /*
     * The main run loop for the interpreter. Since this is the hot path,
     * do not use functions such as lookup() or read_operands() for decoding
     * instructions and operands.
     */
    pub fn run(&mut self, instructions: &Instructions) -> Result<(), RTError> {
        let mut ip = 0;
        while ip < instructions.len() {
            let op = Opcode::from(instructions.code[ip]);
            let line = instructions.lines[ip];
            match op {
                Opcode::Constant => {
                    let const_index =
                        BigEndian::read_u16(&instructions.code[ip + 1..ip + 3]) as usize;
                    let constant = self
                        .constants
                        .get(const_index)
                        .ok_or_else(|| RTError::new("constant not found", line))?;
                    self.push(Rc::new(constant.clone()));
                    ip += 2;
                }
                Opcode::Pop => {
                    self.pop();
                }
                Opcode::Add => {
                    self.binary_op(OperandType::TwoNumbers, |a, b| a + b, line)?;
                }
                Opcode::Sub => {
                    self.binary_op(OperandType::TwoNumbers, |a, b| a - b, line)?;
                }
                Opcode::Mul => {
                    self.binary_op(OperandType::TwoNumbers, |a, b| a * b, line)?;
                }
                Opcode::Div => {
                    self.binary_op(OperandType::TwoNumbers, |a, b| a / b, line)?;
                }
                Opcode::Invalid => {
                    return Err(RTError::new(&format!("opcode {:?} undefined", op), line))
                }
            }
            ip += 1;
        }

        Ok(())
    }

    fn binary_op(
        &mut self,
        optype: OperandType,
        op: fn(a: &Object, b: &Object) -> Object,
        line: usize,
    ) -> Result<(), RTError> {
        if self.peek(0).is_string() && self.peek(1).is_string() {
            // pop b before a
            let b = self.pop();
            let a = self.pop();
            self.push(Rc::new(Object::Str(format!("{}{}", a, b))));
            Ok(())
        } else if self.peek(0).is_number() && self.peek(1).is_number() {
            // pop b before a
            let b = self.pop();
            let a = self.pop();
            self.push(Rc::new(op(&a, &b)));
            Ok(())
        } else {
            match optype {
                OperandType::TwoNumbers => Err(RTError::new("Operands must be numbers.", line)),
                OperandType::NumbersOrStrings => Err(RTError::new(
                    "Operands must be two numbers or two strings.",
                    1,
                )),
            }
        }
    }
}
