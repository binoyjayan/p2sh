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

        let mut instruction = Vec::with_capacity(instruction_len as usize);
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
        match width {
            2 => {
                let bytes: &[u8; 2] = ins[offset..offset + width]
                    .try_into()
                    .expect("Invalid slice size");
                operands[i] = u16::from_be_bytes(*bytes) as usize;
            }
            _ => panic!("Unsupported operand width: {}", width),
        }
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
            _ => format!("ERROR: unhandled operandCount for {}", def.name),
        }
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
