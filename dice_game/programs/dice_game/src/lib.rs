pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("4wqabXsr1RXm8zcng5mGA4RFa3HVuCB8UsBZkL7MGfRZ");

#[program]
pub mod dice_game {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, amount: u64) -> Result<()> {
        ctx.accounts.init(amount)
    }

    pub fn place_bet(ctx: Context<PlaceBet>, seed: u128, roll: u8, amount: u64) -> Result<()> {
        ctx.accounts.create_bet(seed, roll, amount, &ctx.bumps)?;
        ctx.accounts.deposit(amount)
    }

    pub fn resolve_bet(ctx: Context<ResolveBet>) -> Result<()> {
        let house_roll = ctx.accounts.read_house_roll()?;
        ctx.accounts.resolve_bet(&ctx.bumps, house_roll)
    }

    pub fn refund_bet(ctx: Context<RefundBet>) -> Result<()> {
        ctx.accounts.refund_bet(&ctx.bumps)
    }
}
