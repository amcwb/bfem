use clap::ValueEnum;

use crate::{
    errors::{BFError, BFErrors},
    TapeFlags,
};

fn zeros(size: u128) -> Vec<u8> {
    vec![0; size as usize]
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum TapeMode {
    /// Loop round to the start
    Circular,
    /// Create new cells at the end
    Append,
    /// Panic
    Panic,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum CellMode {
    /// -1 becomes 255. 256 becomes 0.
    Circular,
    /// Nothing. -1 becomes 0. 256 becomes 255.
    Nothing,
    /// Panic
    Panic,
}

pub struct Tape {
    size: u128,
    cells: Vec<u8>,
    tape_behaviour: TapeMode,
    cell_behaviour: CellMode,
    /// Pointer
    pointer: u128,

    /// The amount indexes should be shifted. This only applies
    /// when we add cells to the _start_ but we have named cells.
    pub shift: u128,
}

impl Default for Tape {
    fn default() -> Self {
        Self {
            size: 30000,
            cells: zeros(30000),
            tape_behaviour: TapeMode::Circular,
            cell_behaviour: CellMode::Circular,
            pointer: 0,
            shift: 0,
        }
    }
}

impl Tape {
    pub fn new(flags: TapeFlags) -> Self {
        Self {
            size: flags.tape_size,
            cells: zeros(flags.tape_size),
            tape_behaviour: flags.tape_mode,
            cell_behaviour: flags.cell_mode,
            pointer: 0,
            shift: 0,
        }
    }

    pub fn realign(&mut self) {
        self.pointer = 0;
    }

    pub fn clear(&mut self) {
        self.cells = zeros(self.size);
    }

    pub fn get_value(&self) -> u8 {
        self.cells[self.pointer as usize]
    }

    pub fn get_value_at_index(&self, address: u128) -> u8 {
        self.cells[address as usize]
    }

    pub fn set_value_at_index(&mut self, address: u128, value: u8) {
        self.cells[address as usize] = value;
    }

    pub fn set_value(&mut self, value: u8) {
        self.cells[self.pointer as usize] = value;
    }

    pub fn get_pointer(&self) -> u128 {
        self.pointer
    }

    pub fn set_pointer(&mut self, value: u128) {
        self.pointer = value;
    }

    pub fn size(&self) -> u128 {
        self.cells.len() as u128
    }

    pub fn add(&mut self, count: u8) -> Result<(), BFError> {
        match self.cell_behaviour {
            CellMode::Circular => {
                let value = self.cells[self.pointer as usize];
                self.cells[self.pointer as usize] = value.overflowing_add(count).0;
                Ok(())
            }
            CellMode::Nothing => {
                self.cells[self.pointer as usize] = self.cells[self.pointer as usize]
                    .checked_add(count)
                    .unwrap_or(u8::MAX);
                Ok(())
            }
            CellMode::Panic => {
                let (pointer, overflow) = self.cells[self.pointer as usize].overflowing_add(count);
                if overflow {
                    Err(BFError::new(
                        BFErrors::RuntimeError,
                        format!(
                            "Cell {} (value {}) would go above {} if {} were added",
                            self.pointer,
                            self.cells[self.pointer as usize],
                            0,
                            count
                        ),
                    ))
                } else {
                    self.cells[self.pointer as usize] = pointer;
                    Ok(())
                }
            }
        }
    }

    pub fn sub(&mut self, count: u8) -> Result<(), BFError> {
        match self.cell_behaviour {
            CellMode::Circular => {
                let value = self.cells[self.pointer as usize];
                self.cells[self.pointer as usize] = value.overflowing_sub(count).0;
                Ok(())
            }
            CellMode::Nothing => {
                self.cells[self.pointer as usize] = self.cells[self.pointer as usize]
                    .checked_sub(count)
                    .unwrap_or(0);
                Ok(())
            }
            CellMode::Panic => {
                let (pointer, overflow) = self.cells[self.pointer as usize].overflowing_sub(count);
                if overflow {
                    Err(BFError::new(
                        BFErrors::RuntimeError,
                        format!(
                            "Cell {} (value {}) would go below {} if {} were subtracted",
                            self.pointer,
                            self.cells[self.pointer as usize],
                            u8::MAX,
                            count
                        ),
                    ))
                } else {
                    self.cells[self.pointer as usize] = pointer;
                    Ok(())
                }
            },
        }
    }

    pub fn left(&mut self, count: u128) -> Result<(), BFError> {
        match self.tape_behaviour {
            TapeMode::Circular => {
                if self.pointer >= count {
                    self.pointer -= count;
                } else {
                    self.pointer = self.cells.len() as u128 - (count - self.pointer)
                }

                Ok(())
            }
            TapeMode::Append => {
                if self.pointer >= count {
                    self.pointer -= count;
                } else {
                    self.pointer = 0;
                    // Create more cells
                    self.cells.splice(0..0, zeros(count).iter().cloned());
                }

                Ok(())
            }
            TapeMode::Panic => {
                let (pointer, overflow) = self.pointer.overflowing_sub(count);
                if overflow {
                    Err(BFError::new(
                        BFErrors::RuntimeError,
                        format!(
                            "Tape pointer would be below {} if moved left {} spaces from {}",
                            0, count, self.pointer
                        ),
                    ))
                } else {
                    self.pointer = pointer;
                    Ok(())
                }
            }
        }
    }

    pub fn right(&mut self, count: u128) -> Result<(), BFError> {
        match self.tape_behaviour {
            TapeMode::Circular => {
                if self.pointer >= count {
                    self.pointer -= count;
                } else {
                    self.pointer = self.cells.len() as u128 - (count - self.pointer)
                }

                Ok(())
            }
            TapeMode::Append => {
                self.pointer += count;

                // Create more cells
                let mut data = zeros(count);
                self.cells.append(&mut data);

                Ok(())
            }
            TapeMode::Panic => {
                let (pointer, overflow) = self.pointer.overflowing_add(count);
                if overflow || pointer > self.size {
                    Err(BFError::new(
                        BFErrors::RuntimeError,
                        format!(
                            "Tape pointer would be above {} if moved right {} spaces from {}",
                            self.cells.len(),
                            count,
                            self.pointer
                        ),
                    ))
                } else {
                    self.pointer = pointer;
                    Ok(())
                }
            }
        }
    }
}
