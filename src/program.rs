use std::{fs, path::PathBuf, process};

use crate::{
    errors::{fmt_report, BFError},
    parser::Parser,
    tape::Tape,
    DisableFlags,
};
use miette::{miette, LabeledSpan, Report, SourceSpan};
use pancurses::{endwin, initscr, noecho, Input, Window};

/// All instructions with optimisations for count
#[derive(Clone, Debug)]
pub enum Instruction {
    Add(u8),
    Subtract(u8),
    Loop(Vec<(SourceSpan, Instruction)>),
    Left(u128),
    Right(u128),
    Input,
    Output,
}

/// A core program. This contains no special features, and is the result of
/// BFEM code being parsed.
pub struct Program {
    /// Source.
    src: String,
    /// Instructions.
    instructions: Vec<(SourceSpan, Instruction)>,
    /// Tape. Can only be 0-255
    pub tape: Tape,
    /// Disabled flags
    flag: DisableFlags,
    window: Window,
}

impl Program {
    pub fn new(
        src: String,
        instructions: Vec<(SourceSpan, Instruction)>,
        tape: Tape,
        flag: DisableFlags,
    ) -> Self {
        let window = initscr();
        Self {
            src,
            instructions,
            tape,
            flag,
            window,
        }
    }

    pub fn read_file(path: PathBuf, tape: Tape, flag: DisableFlags) -> Self {
        let file = fs::read_to_string(path).expect("File not found");

        Program::parse(file, tape, flag)
    }

    pub fn parse(src: String, tape: Tape, flag: DisableFlags) -> Self {
        // Use parser to parse it
        let instructions = Parser::parse(src.clone(), flag);
        Self::new(src, instructions, tape, flag)
    }

    pub fn get_instructions(&self) -> &Vec<(SourceSpan, Instruction)> {
        &self.instructions
    }

    fn run_one(&mut self, instruction: &Instruction) -> Result<(), BFError> {
        match instruction.clone() {
            Instruction::Add(count) => {
                self.tape.add(count)?;
            }
            Instruction::Subtract(count) => {
                self.tape.sub(count)?;
            }
            Instruction::Loop(instructions) => {
                while self.tape.get_value() != 0 {
                    for (_span, instruction) in &instructions {
                        self.run_one(instruction)?;
                    }
                }
            }
            Instruction::Left(count) => {
                self.tape.left(count)?;
            }
            Instruction::Right(count) => {
                self.tape.right(count)?;
            }
            Instruction::Input => {
                let mut character: Option<char> = None;
                while character.is_none() {
                    match self.window.getch() {
                        Some(Input::Character(c)) => character = Some(c),
                        _ => (),
                    }
                }

                self.tape.set_value(character.unwrap() as u8)
            }
            Instruction::Output => {
                self.window.addch(self.tape.get_value() as char);
            }
        }

        Ok(())
    }

    pub fn run(&mut self) {
        // Iterate through instructions, catch error if possible
        self.tape.clear();
        self.tape.realign();
        for (source_span, instruction) in self.instructions.clone() {
            let instruction = instruction.clone();
            let source_span = source_span.clone();

            match self.run_one(&instruction) {
                Ok(()) => continue,
                Err(error) => {
                    let report = miette!(
                        labels = vec![LabeledSpan::new_with_span(
                            Some("error occurs here".to_string()),
                            source_span.offset()
                        )],
                        "{}",
                        error.message
                    );
                    println!(
                        "{}",
                        fmt_report(
                            // (Into::<Report>::into(error.into_detailed(source_span)))
                            //     .with_source_code(self.src.clone())
                            (report).with_source_code(self.src.clone())
                        )
                    );
                    process::exit(1);
                }
            }
        }
    }
}
