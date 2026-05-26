use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use crate::state::Config;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,

    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,

    #[account(
        init,
        payer = initializer,
        seeds = [b"config", seed.to_le_bytes().as_ref()],
        bump,
        space = 8 + Config::INIT_SPACE,
    )]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = initializer,
        seeds = [b"lp", config.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = config,
    )]
    pub mint_lp: Account<'info, Mint>,

    #[account(
        init,
        payer = initializer,
        associated_token::mint = mint_x,
        associated_token::authority = config,
    )]
    pub vault_x: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = initializer,
        associated_token::mint = mint_y,
        associated_token::authority = config,
    )]
    pub vault_y: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<Initialize>,
    seed: u64,
    fee: u16,
    authority: Option<Pubkey>,
    treasury: Option<Pubkey>,
) -> Result<()> {
    require!(fee <= 1000, AmmError::InvalidFee);
    require!(
        ctx.accounts.mint_x.key() != ctx.accounts.mint_y.key(),
        AmmError::SameTokenMints
    );

    let config = &mut ctx.accounts.config;
    config.seed = seed;
    config.authority = authority;
    config.mint_x = ctx.accounts.mint_x.key();
    config.mint_y = ctx.accounts.mint_y.key();
    config.fee = fee;
    config.locked = false;
    config.config_bump = ctx.bumps.config;
    config.lp_bump = ctx.bumps.mint_lp;
    config.treasury = treasury;

    Ok(())
}

#[error_code]
pub enum AmmError {
    #[msg("Fee must be <= 1000 basis points (10%)")]
    InvalidFee,
    #[msg("Token X and Y mints must differ")]
    SameTokenMints,
}
