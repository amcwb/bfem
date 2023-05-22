use std::{fs, path::PathBuf, process};

use crate::{
    errors::{fmt_report, BFError, BFErrors},
    parser::Parser,
    tape::Tape,
    DisableFlags,
};
use bimap::BiMap;
use getch::Getch;
use miette::{miette, LabeledSpan, NamedSource, SourceSpan};

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

    // For aliases
    Goto(String),
}

/// A core program. This contains no special features, and is the result of
/// BFEM code being parsed.
pub struct Program {
    path: PathBuf,
    /// Source.
    src: String,
    /// Instructions.
    instructions: Vec<(SourceSpan, Instruction)>,
    /// Tape. Can only be 0-255
    pub tape: Tape,
    /// Disabled flags
    flag: DisableFlags,
    getch: Getch,
    /// Aliases
    aliases: BiMap<String, u128>,
    /// Parser
    parser: Option<Parser>,
}

impl Program {
    pub fn new(
        path: PathBuf,
        src: String,
        instructions: Vec<(SourceSpan, Instruction)>,
        tape: Tape,
        flag: DisableFlags,
        parser: Option<Parser>,
    ) -> Self {
        let getch = Getch::new();
        Self {
            path,
            src,
            instructions,
            tape,
            flag,
            getch,
            aliases: BiMap::new(),
            parser,
        }
    }

    pub fn read_file(path: PathBuf, tape: Tape, flag: DisableFlags) -> Self {
        let file = fs::read_to_string(path.clone()).expect("File not found");

        Program::parse(path, file, tape, flag)
    }

    pub fn parse(path: PathBuf, src: String, tape: Tape, flag: DisableFlags) -> Self {
        // Use parser to parse it
        let mut parser = Parser::new(src.clone(), flag);
        let instructions = parser.parse();
        Self::new(path, src, instructions, tape, flag, Some(parser))
    }

    pub fn get_instructions(&self) -> &Vec<(SourceSpan, Instruction)> {
        &self.instructions
    }

    pub fn setup(&mut self) {
        if let Some(parser) = &self.parser {
            if !self.flag.disable_alloc {
                self.run_prealloc(
                    parser
                        .get_aliases()
                        .iter()
                        .map(|f| f.to_owned())
                        .collect::<Vec<_>>(),
                )
            }
        }
    }

    pub fn run_prealloc(&mut self, aliases: Vec<String>) {
        for alias in aliases {
            self.assign_alias_address(alias);
        }
    }

    fn assign_alias_address(&mut self, key: String) -> u128 {
        // Work backwards until we find an empty spot
        let mut index = self.tape.size() - 1;
        while self.tape.get_value_at_index(index) != 0 || self.aliases.contains_right(&index) {
            index -= 1;
        }

        self.aliases.insert(key.clone(), index);
        index
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
                let mut character: Option<u8> = None;
                while character.is_none() {
                    match self.getch.getch() {
                        Ok(c) => character = Some(c),
                        _ => (),
                    }
                }

                self.tape.set_value(character.unwrap())
            }
            Instruction::Output => {
                print!("{}", self.tape.get_value() as char);
            }
            Instruction::Goto(key) => {
                let address = self.aliases.get_by_left(&key);
                if let Some(address) = address {
                    self.tape.set_pointer(*address);
                } else if self.flag.disable_alloc {
                    // Alloc was disabled so we need to assign at runtime
                    let index = self.assign_alias_address(key);
                    self.tape.set_pointer(index);
                } else {
                    return Err(BFError::new(
                        BFErrors::RuntimeError,
                        format!("Alias {} was not found and pre-alloc was not disabled. This may indicate an error in the compiler", key),
                    ));
                }
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
                            source_span
                        )],
                        "{}",
                        error.message
                    );
                    println!(
                        "{}",
                        fmt_report(
                            (report).with_source_code(NamedSource::new(
                                self.path.to_str().unwrap(),
                                self.src.clone()
                            )),
                            Some(&instruction)
                        )
                    );
                    process::exit(1);
                }
            }
        }
    }

    fn produce_labeled_spans(instructions: &Vec<(SourceSpan, Instruction)>) -> Vec<LabeledSpan> {
        let mut labeled_spans: Vec<LabeledSpan> = vec![];
        for (source_span, instruction) in instructions {
            let info = match instruction {
                Instruction::Add(value) => Some(format!("Add {}", value)),
                Instruction::Subtract(value) => Some(format!("Subtract {}", value)),
                Instruction::Loop(layer_instructions) => {
                    labeled_spans.append(&mut Program::produce_labeled_spans(layer_instructions));

                    None
                }
                Instruction::Left(value) => Some(format!("Move left {} spaces", value)),
                Instruction::Right(value) => Some(format!("Move right {} spaces", value)),
                Instruction::Input => Some("Take input".to_string()),
                Instruction::Output => Some("Write output".to_string()),
                Instruction::Goto(name) => Some(format!("Go to alias {}", name)),
            };

            if let Some(info) = info {
                labeled_spans.push(LabeledSpan::new_with_span(Some(info), source_span.clone()));
            }
        }

        labeled_spans
    }

    pub fn info(&mut self) {
        let mut labeled_spans: Vec<LabeledSpan> =
            Program::produce_labeled_spans(&self.instructions);

        let report = miette!(labels = labeled_spans, "{}", "Your info sheet");
        println!(
            "{}",
            fmt_report(
                (report).with_source_code(NamedSource::new(
                    self.path.to_str().unwrap(),
                    self.src.clone()
                )),
                None
            )
        );

        process::exit(0);
    }
}
