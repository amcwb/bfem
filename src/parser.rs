use crate::{program::Instruction, DisableFlags};
use miette::SourceSpan;

pub struct Parser {}

impl Parser {
    fn parse_one(
        src: String,
        mut index: usize,
        flag: DisableFlags,
    ) -> (usize, (SourceSpan, Instruction)) {
        let character = src.chars().nth(index).unwrap();
        let start_index = index;
        let instruction = match character {
            '+' => {
                index += 1;
                Instruction::Add(1)
            }
            '-' => {
                index += 1;
                Instruction::Subtract(1)
            }
            '>' => {
                index += 1;
                Instruction::Right(1)
            }
            '<' => {
                index += 1;
                Instruction::Left(1)
            }
            '[' => {
                index += 1;
                let mut instructions: Vec<(SourceSpan, Instruction)> = vec![];
                let mut character = src.chars().nth(index).unwrap();

                // Keep going until we encounter close brackets
                while character != ']' {
                    let (new_index, instruction) = Parser::parse_one(src.clone(), index, flag);
                    index = new_index;
                    instructions.push(instruction);

                    character = src.chars().nth(index).unwrap();
                }

                // Skip over end loop
                index += 1;

                Instruction::Loop(instructions)
            }
            '.' => {
                index += 1;
                Instruction::Output
            }
            ',' => {
                index += 1;
                Instruction::Input
            }
            '{' if !flag.disable_aliases => todo!(),
            _ => panic!("Unrecognised character: {}", character),
        };

        (
            index,
            ((start_index, index - start_index).into(), instruction),
        )
    }

    pub fn parse(src: String, flag: DisableFlags) -> Vec<(SourceSpan, Instruction)> {
        let mut index: usize = 0;
        let mut instructions: Vec<(SourceSpan, Instruction)> = vec![];

        while index < src.len() {
            let (new_index, instruction) = Parser::parse_one(src.clone(), index, flag);
            index = new_index;
            instructions.push(instruction);
        }

        instructions
    }
}
