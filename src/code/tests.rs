#![allow(unused_imports)]
use super::opcode::*;
use crate::code::definitions;
use crate::code::definitions::Instructions;

#[test]
fn test_make() {
    /*
     * For opcode 'Constant', pass OpConstant and the operand 65534 to Make.
     * Expect to get back a `Vec<u8>` holding three bytes. Of these three, the
     * first one has to be the opcode, Constant, and the other two should be
     * the big-endian encoding of 65534. That’s also why '65534' is used instead
     * of the maximum value 65535. This way the endianness can be validated.
     * 65534 is encoded in big endian as the byte sequence 0xFF 0xFE.
     */
    let tests = vec![
        (
            Opcode::Constant,
            &[65534],
            vec![Opcode::Constant as u8, 255, 254],
        ),
        (Opcode::Add, &[0], vec![Opcode::Add as u8]),
    ];

    for (op, operands, expected) in tests {
        let instruction = definitions::make(op, operands, 1).code;
        assert_eq!(
            instruction.len(),
            expected.len(),
            "instruction has wrong length. want={}, got={}",
            expected.len(),
            instruction.len()
        );

        for (i, &b) in expected.iter().enumerate() {
            assert_eq!(
                instruction[i], b,
                "wrong byte at pos {}. want={}, got={}",
                i, b, instruction[i]
            );
        }
    }
}

#[cfg(test)]
fn concat_instructions(s: &[Instructions]) -> Instructions {
    let mut out = Instructions::default();
    for ins in s {
        out.code.extend_from_slice(&ins.code);
    }
    out
}

#[test]
fn test_instructions_string() {
    let instructions = vec![
        definitions::make(Opcode::Add, &[0], 1),
        definitions::make(Opcode::Constant, &[2], 1),
        definitions::make(Opcode::Constant, &[65535], 1),
    ];
    // The '\' at the end of the lines escapes indentation in the next line
    let expected = "\
        0000 OpAdd\n\
        0001 OpConstant 2\n\
        0004 OpConstant 65535\n";
    let concatted = concat_instructions(&instructions);

    assert_eq!(concatted.to_string(), expected);
}

#[test]
fn test_read_operands() {
    let tests = vec![(Opcode::Constant, vec![65535], 2)];

    for (op, operands, bytes_read) in tests {
        let instruction = definitions::make(op, &operands, 1);
        let def = definitions::lookup(instruction.code[0] as u8).unwrap();
        let (operands_read, n) = definitions::read_operands(def, &instruction.code[1..]);

        assert_eq!(n, bytes_read, "n is wrong");

        for (i, want) in operands.iter().enumerate() {
            assert_eq!(operands_read[i], *want);
        }
    }
}
