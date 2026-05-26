use anchor_lang::prelude::*;
use anchor_spl::token::{burn, transfer, Burn, Mint, Token, TokenAccount, Transfer};
use crate::state::Config;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
        constraint = !config.locked @ WithdrawError::PoolLocked,
    )]
    pub config: Account<'info, Config>,

    #[account(address = config.mint_x)]
    pub mint_x: Account<'info, Mint>,

    #[account(address = config.mint_y)]
    pub mint_y: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"lp", config.key().as_ref()],
        bump = config.lp_bump,
    )]
    pub mint_lp: Account<'info, Mint>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
    )]
    pub vault_x: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
    )]
    pub vault_y: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_x.mint == config.mint_x @ WithdrawError::WrongMint,
        constraint = user_x.owner == user.key() @ WithdrawError::WrongOwner,
    )]
    pub user_x: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_y.mint == config.mint_y @ WithdrawError::WrongMint,
        constraint = user_y.owner == user.key() @ WithdrawError::WrongOwner,
    )]
    pub user_y: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_lp.mint == mint_lp.key() @ WithdrawError::WrongMint,
        constraint = user_lp.owner == user.key() @ WithdrawError::WrongOwner,
    )]
    pub user_lp: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(
    ctx: Context<Withdraw>,
    amount: u64,
    minimum_amount_x: u64,
    minimum_amount_y: u64,
) -> Result<()> {
    require!(amount > 0, WithdrawError::ZeroAmount);

    let lp_supply = ctx.accounts.mint_lp.supply;
    require!(lp_supply > 0, WithdrawError::NoLiquidity);
    require!(amount <= lp_supply, WithdrawError::BurnExceedsSupply);

    let x_amount = proportional(amount, ctx.accounts.vault_x.amount, lp_supply)?;
    let y_amount = proportional(amount, ctx.accounts.vault_y.amount, lp_supply)?;

    require!(x_amount >= minimum_amount_x, WithdrawError::SlippageExceeded);
    require!(y_amount >= minimum_amount_y, WithdrawError::SlippageExceeded);

    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Burn {
            mint: ctx.accounts.mint_lp.to_account_info(),
            from: ctx.accounts.user_lp.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    burn(cpi_ctx, amount)?;

    let seed_bytes = ctx.accounts.config.seed.to_le_bytes();
    let bump = ctx.accounts.config.config_bump;
    let seeds: &[&[u8]] = &[b"config", &seed_bytes, &[bump]];
    let signer = &[seeds];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.vault_x.to_account_info(),
            to: ctx.accounts.user_x.to_account_info(),
            authority: ctx.accounts.config.to_account_info(),
        },
        signer,
    );
    transfer(cpi_ctx, x_amount)?;

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.vault_y.to_account_info(),
            to: ctx.accounts.user_y.to_account_info(),
            authority: ctx.accounts.config.to_account_info(),
        },
        signer,
    );
    transfer(cpi_ctx, y_amount)?;

    msg!("withdraw: lp_burned={} x={} y={}", amount, x_amount, y_amount);
    Ok(())
}

fn proportional(lp_amount: u64, vault_amount: u64, lp_supply: u64) -> Result<u64> {
    let result = (vault_amount as u128)
        .checked_mul(lp_amount as u128)
        .ok_or(WithdrawError::Overflow)?
        .checked_div(lp_supply as u128)
        .ok_or(WithdrawError::Overflow)?;

    require!(result <= u64::MAX as u128, WithdrawError::Overflow);
    Ok(result as u64)
}

#[error_code]
pub enum WithdrawError {
    #[msg("Amount must be > 0")]
    ZeroAmount,
    #[msg("Slippage limit exceeded")]
    SlippageExceeded,
    #[msg("Math overflow")]
    Overflow,
    #[msg("Pool is locked")]
    PoolLocked,
    #[msg("No liquidity in pool")]
    NoLiquidity,
    #[msg("Burn amount exceeds LP supply")]
    BurnExceedsSupply,
    #[msg("Wrong token mint")]
    WrongMint,
    #[msg("Wrong token account owner")]
    WrongOwner,
}
