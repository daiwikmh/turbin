use anchor_lang::prelude::*;

#[error_code]
pub enum DiceError {
    InvalidRoll,
    BetTooSmall,
    InvalidMemoProgram,
    InvalidHouseRoll,
    SeedMismatch,
    TimeoutNotReached,
    Overflow,
}
