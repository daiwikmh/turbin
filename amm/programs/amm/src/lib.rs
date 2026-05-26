use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("5ZgBaT6F6T5eA86EAX5q8bT368YWSUr3EfdaRUa4DDta");

#[program]
pub mod amm {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        seed: u64,
        fee: u16,
        authority: Option<Pubkey>,
        treasury: Option<Pubkey>,
    ) -> Result<()> {
        instructions::initialize::handler(ctx, seed, fee, authority, treasury)
    }

    pub fn deposit(
        ctx: Context<Deposit>,
        amount: u64,
        max_x: u64,
        max_y: u64,
    ) -> Result<()> {
        instructions::deposit::handler(ctx, amount, max_x, max_y)
    }

    pub fn swap(
        ctx: Context<Swap>,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> Result<()> {
        instructions::swap::handler(ctx, amount_in, minimum_amount_out)
    }

    pub fn withdraw(
        ctx: Context<Withdraw>,
        amount: u64,
        minimum_amount_x: u64,
        minimum_amount_y: u64,
    ) -> Result<()> {
        instructions::withdraw::handler(ctx, amount, minimum_amount_x, minimum_amount_y)
    }

    pub fn update_config(
        ctx: Context<UpdateConfig>,
        new_fee: Option<u16>,
        new_locked: Option<bool>,
        new_authority: Option<Pubkey>,
    ) -> Result<()> {
        instructions::update_config::handler(ctx, new_fee, new_locked, new_authority)
    }
}
