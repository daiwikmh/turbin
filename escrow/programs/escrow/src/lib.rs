pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("9VPfkPig6vxgWa4qyGXiuQwebMtRLtFqNayyaUN2f4gD");

#[program]
pub mod escrow {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    pub fn make(ctx: Context<Make>, seed: u64, deposit: u64, receive: u64, duration: i64) -> Result<()> {
        ctx.accounts.init_escrow(seed, receive, duration, &ctx.bumps)?;
        ctx.accounts.deposit_to_vault(deposit)
    }

    pub fn take(ctx: Context<Take>) -> Result<()> {
        ctx.accounts.exchange()
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund_and_close_vault()
    }
}
