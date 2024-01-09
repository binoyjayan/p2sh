use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fmt;

use super::opcode::*;
use byteorder::{BigEndian, WriteBytesExt};

#[derive(Debug)]
pub struct Definition {
    name: &'static str,
    operand_widths: &'static [usize],
}

impl Definition {
    fn new(name: &'static str, operand_widths: &'static [usize]) -> Definition {
        Definition {
            name,
            operand_widths,
        }
    }
}

lazy_static! {
    static ref DEFINITIONS: HashMap<Opcode, Definition> = {
        let mut map = HashMap::new();
        map.insert(Opcode::Constant, Definition::new("OpConstant", &[2]));
        map.insert(Opcode::Pop, Definition::new("OpPop", &[]));
        map.insert(Opcode::Add, Definition::new("OpAdd", &[]));
        map.insert(Opcode::Sub, Definition::new("OpSub", &[]));
        map.insert(Opcode::Mul, Definition::new("OpMul", &[]));
        map.insert(Opcode::Div, Definition::new("OpDiv", &[]));
        map.insert(Opcode::Mod, Definition::new("OpMod", &[]));
        map.insert(Opcode::True, Definition::new("OpTrue", &[]));
        map.insert(Opcode::False, Definition::new("OpFalse", &[]));
        map.insert(Opcode::Equal, Definition::new("OpEqual", &[]));
        map.insert(Opcode::NotEqual, Definition::new("OpNotEqual", &[]));
        map.insert(Opcode::Greater, Definition::new("OpGreater", &[]));
        map.insert(Opcode::GreaterEq, Definition::new("OpGreaterEq", &[]));
        map.insert(Opcode::Minus, Definition::new("OpMinus", &[]));
        map.insert(Opcode::Bang, Definition::new("OpBang", &[]));
        map.insert(Opcode::Jump, Definition::new("OpJump", &[2]));
        map.insert(Opcode::JumpIfFalse, Definition::new("OpJumpIfFalse", &[2]));
        map.insert(Opcode::JumpIfFalseNoPop, Definition::new("OpJumpIfFalseNoPop", &[2]));
        map.insert(Opcode::Null, Definition::new("OpNull", &[]));
        map.insert(Opcode::DefineGlobal, Definition::new("OpDefineGlobal", &[2]));
        map.insert(Opcode::GetGlobal, Definition::new("OpGetGlobal", &[2]));
        map.insert(Opcode::SetGlobal, Definition::new("OpSetGlobal", &[2]));
        map.insert(Opcode::Array, Definition::new("OpArray", &[2]));
        map.insert(Opcode::Map, Definition::new("OpMap", &[2]));
        map.insert(Opcode::GetIndex, Definition::new("OpGetIndex", &[]));
        map.insert(Opcode::SetIndex, Definition::new("OpSetIndex", &[]));
        map.insert(Opcode::Call, Definition::new("OpCall", &[1]));
        map.insert(Opcode::ReturnValue, Definition::new("OpReturnValue", &[]));
        map.insert(Opcode::Return, Definition::new("OpReturn", &[]));
        map.insert(Opcode::DefineLocal, Definition::new("OpDefineLocal", &[1]));
        map.insert(Opcode::GetLocal, Definition::new("OpGetLocal", &[1]));
        map.insert(Opcode::SetLocal, Definition::new("OpSetLocal", &[1]));
        map.insert(Opcode::GetBuiltinFn, Definition::new("OpGetBuiltinFn", &[1]));
        map.insert(Opcode::GetBuiltinVar, Definition::new("OpGetBuiltinVar", &[1]));
        // 'OpClosure' has two operands - a 2-byte constant index and #free-variables
        map.insert(Opcode::Closure, Definition::new("OpClosure", &[2, 1]));
        map.insert(Opcode::GetFree, Definition::new("OpGetFree", &[1]));
        map.insert(Opcode::SetFree, Definition::new("OpSetFree", &[1]));
        map.insert(Opcode::CurrClosure, Definition::new("OpCurrClosure", &[]));
        map.insert(Opcode::Not, Definition::new("OpNot", &[]));
        map.insert(Opcode::And, Definition::new("OpAnd", &[]));
        map.insert(Opcode::Or, Definition::new("OpOr", &[]));
        map.insert(Opcode::Xor, Definition::new("OpXor", &[]));
        map.insert(Opcode::ShiftLeft, Definition::new("OpShiftLeft", &[]));
        map.insert(Opcode::ShiftRight, Definition::new("OpShiftRight", &[]));
        map.insert(Opcode::Dup, Definition::new("OpDup", &[]));
        map.insert(Opcode::GetProp, Definition::new("OpGetProp", &[1]));
        map.insert(Opcode::SetProp, Definition::new("OpSetProp", &[1]));
        map.insert(Opcode::Dollar, Definition::new("OpDollar", &[]));
        map
    };
}

pub fn lookup(op: u8) -> Result<&'static Definition, String> {
    match DEFINITIONS.get(&Opcode::from(op)) {
        Some(def) => Ok(def),
        None => Err(format!("opcode {} undefined", op)),
    }
}

/*
 * Helper function to build up bytecode instructions
 * After calculating the final value of instruction_len, allocate the
 * 'instruction' vector with a fixed capacity and add the opcode as the
 * first byte. Then, iterate over the 'operand_widths', take the matching
 * element from the operands and add it to the instruction. Make sure
 * that the operands are encoded in BigEndian format. After encoding the
 * operand, increment the offset by its width.
 */
pub fn make(op: Opcode, operands: &[usize], line: usize) -> Instructions {
    if let Some(def) = DEFINITIONS.get(&op) {
        let mut instruction_len = 1;
        for &w in def.operand_widths {
            instruction_len += w;
        }

        let mut instruction = Vec::with_capacity(instruction_len);
        instruction.push(op.into());

        // Iterate through operands and its widths
        for (&o, width) in operands.iter().zip(def.operand_widths) {
            // Generate bytecode depending on the width of the operands
            match width {
                2 => {
                    instruction.write_u16::<BigEndian>(o as u16).unwrap();
                }
                1 => {
                    instruction.write_u8(o as u8).unwrap();
                }
                _ => panic!("Unsupported operand width: {}", width),
            }
        }

        Instructions::new(instruction, vec![line; instruction_len])
    } else {
        Instructions::new(Vec::new(), Vec::new())
    }
}

/*
 * Helper function to decode the the operands of a bytecode instruction.
 * It is a counterpart of 'make'
 */
pub fn read_operands(def: &Definition, ins: &[u8]) -> (Vec<usize>, usize) {
    let mut operands = vec![0; def.operand_widths.len()];
    let mut offset = 0;

    for (i, &width) in def.operand_widths.iter().enumerate() {
        operands[i] = match width {
            2 => u16::from_be_bytes([ins[offset], ins[offset + 1]]) as usize,
            1 => ins[offset] as usize,
            _ => panic!("Unsupported operand width: {}", width),
        };
        offset += width;
    }

    (operands, offset)
}

#[derive(Default, Debug, Clone)]
pub struct Instructions {
    pub code: Vec<u8>,
    pub lines: Vec<usize>,
}

impl Instructions {
    pub fn new(data: Vec<u8>, lines: Vec<usize>) -> Instructions {
        Instructions { code: data, lines }
    }
    pub fn len(&self) -> usize {
        self.code.len()
    }
    #[allow(dead_code)]
    pub fn get(&self, index: usize) -> u8 {
        self.code[index]
    }

    fn fmt_instruction(&self, def: &Definition, operands: &[usize]) -> String {
        let operand_count = def.operand_widths.len();
        if operands.len() != operand_count {
            return format!(
                "ERROR: operand len {} does not match defined {}",
                operands.len(),
                operand_count
            );
        }

        match operand_count {
            0 => def.name.to_string(),
            1 => format!("{} {}", def.name, operands[0]),
            2 => format!("{} {} {}", def.name, operands[0], operands[1]),
            _ => format!(
                "ERROR: unhandled operand count for {} ['{}']",
                def.name, operand_count
            ),
        }
    }

    // Disassemble instructions after compilation
    #[allow(dead_code)]
    pub fn disassemble(&self) {
        eprintln!(
            "--------- Instructions [len: {:<4}] -------------------",
            self.len(),
        );
        eprint!("{}", self);
        eprintln!("------------------------------------------------------");
    }

    // Print current instructions during execution
    #[allow(dead_code)]
    pub fn print(&self, ip: usize) {
        eprintln!(
            "--------- Instructions [len: {:<4}, ip: {:<4}] ---------",
            self.len(),
            ip
        );
        eprint!("{}", self);
        eprintln!("------------------------------------------------------");
    }
}

impl fmt::Display for Instructions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut out = String::new();
        let mut i = 0;

        while i < self.code.len() {
            let def = match lookup(self.code[i]) {
                Ok(d) => d,
                Err(err) => {
                    out.push_str(&format!("ERROR: {}\n", err));
                    i += 1;
                    continue;
                }
            };
            let (operands, read) = read_operands(def, &self.code[i + 1..]);
            out.push_str(&format!(
                "{:04} {}\n",
                i,
                self.fmt_instruction(def, &operands)
            ));
            i += 1 + read;
        }

        write!(f, "{}", out)
    }
}

impl PartialEq for Instructions {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code && self.lines == other.lines
    }
}

impl Eq for Instructions {}
