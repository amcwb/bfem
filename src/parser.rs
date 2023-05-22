use crate::{program::Instruction, DisableFlags};
use miette::SourceSpan;
use std::collections::HashSet;

pub struct Parser {
    src: String,
    flag: DisableFlags,
    index: usize,

    // Get names
    aliases: HashSet<String>,
}

impl Parser {
    pub fn new(src: String, flag: DisableFlags) -> Self {
        Self {
            src,
            flag,
            index: 0 as usize,
            aliases: HashSet::new(),
        }
    }

    pub fn get_aliases(&self) -> &HashSet<String> {
        &self.aliases
    }

    fn parse_one(&mut self) -> (SourceSpan, Instruction) {
        let mut character = self.src.chars().nth(self.index).unwrap();
        // Skip whitespaces
        while character.is_whitespace() {
            self.index += 1;
            character = self.src.chars().nth(self.index).unwrap();
        }

        let start_index = self.index;
        let instruction = match character {
            '+' => {
                self.index += 1;
                Instruction::Add(1)
            }
            '-' => {
                self.index += 1;
                Instruction::Subtract(1)
            }
            '>' => {
                self.index += 1;
                Instruction::Right(1)
            }
            '<' => {
                self.index += 1;
                Instruction::Left(1)
            }
            '[' => {
                self.index += 1;
                let mut instructions: Vec<(SourceSpan, Instruction)> = vec![];
                let mut character = self.src.chars().nth(self.index).unwrap();

                // Keep going until we encounter close brackets
                while character != ']' {
                    let instruction = self.parse_one();
                    instructions.push(instruction);

                    character = self.src.chars().nth(self.index).unwrap();
                }

                // Skip over end loop
                self.index += 1;

                Instruction::Loop(instructions)
            }
            '.' => {
                self.index += 1;
                Instruction::Output
            }
            ',' => {
                self.index += 1;
                Instruction::Input
            }
            '{' if !self.flag.disable_aliases => {
                self.index += 1;
                let mut name = String::new();
                let mut character = self.src.chars().nth(self.index).unwrap();

                // Keep going until we encounter close brackets
                while character != '}' {
                    name.push(character);
                    self.index += 1;
                    character = self.src.chars().nth(self.index).unwrap();
                }

                // Skip over end loop
                self.index += 1;
                self.aliases.insert(name.clone());
                Instruction::Goto(name)
            }
            _ => panic!("Unrecognised character: {}", character),
        };

        ((start_index, self.index - start_index).into(), instruction)
    }

    fn is_instruction_consecutive(instruction: &Instruction) -> bool {
        match instruction {
            Instruction::Goto(_)
            | Instruction::Input
            | Instruction::Output
            | Instruction::Loop(_) => false,
            _ => true,
        }
    }

    fn set_count(instruction: &Instruction, count: usize) -> Instruction {
        match instruction {
            Instruction::Add(_) => Instruction::Add(count as u8),
            Instruction::Subtract(_) => Instruction::Subtract(count as u8),
            Instruction::Left(_) => Instruction::Left(count as u128),
            Instruction::Right(_) => Instruction::Right(count as u128),
            _ => instruction.clone(),
        }
    }

    pub fn is_consecutive_okay(left: &Instruction, right: &Instruction) -> bool {
        Parser::is_instruction_consecutive(left)
            && Parser::is_instruction_consecutive(right)
            && std::mem::discriminant(left) == std::mem::discriminant(right)
    }

    pub fn optimise_consecutive(
        instructions: &mut Vec<(SourceSpan, Instruction)>,
    ) -> Vec<(SourceSpan, Instruction)> {
        let mut index = 0 as usize;
        let mut optimised: Vec<(SourceSpan, Instruction)> = vec![];

        // Must be -1 as we need to not attempt to stretch past the last one
        while index < instructions.len() {
            let mut count = 1;

            let (start_span, start_instruction) = instructions[index].clone();
            if let Instruction::Loop(mut inner_instructions) = start_instruction {
                optimised.push((
                    (start_span.offset(), inner_instructions.len()).into(),
                    Instruction::Loop(Parser::optimise_consecutive(&mut inner_instructions)),
                ));

                index += count;
            } else if let Instruction::Goto(key) = start_instruction {
                optimised.push((
                    (start_span.offset(), key.len() + 2).into(),
                    Instruction::Goto(key),
                ));

                index += count;
            } else {
                while (index + count) < instructions.len() -1
                {
                    let (_end_span, mut end_instruction) = instructions[index + count].clone();
                    if !Parser::is_consecutive_okay(&start_instruction, &end_instruction) {
                        break;
                    }
                    count += 1;
                    let (_new_end_span, new_end_instruction) = instructions[index + count].clone();

                    end_instruction = new_end_instruction;
                }

                optimised.push((
                    (start_span.offset(), count).into(),
                    Parser::set_count(&start_instruction, count),
                ));

                index += count;
            }
        }

        optimised
    }

    pub fn parse(&mut self) -> Vec<(SourceSpan, Instruction)> {
        let mut instructions: Vec<(SourceSpan, Instruction)> = vec![];

        while self.index < self.src.len() {
            let instruction = self.parse_one();
            instructions.push(instruction);
        }

        if !self.flag.disable_optimise {
            instructions = Parser::optimise_consecutive(&mut instructions);
        }

        instructions
    }
}
