use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, transfer, Mint, MintTo, Token, TokenAccount, Transfer},
};
use crate::state::Config;

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
        constraint = !config.locked @ DepositError::PoolLocked,
    )]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub user: Signer<'info>,

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

    // pool vaults — reference mint_x/mint_y and config
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

    // user ATAs
    #[account(
        mut,
        constraint = user_x.mint == config.mint_x @ DepositError::WrongMint,
        constraint = user_x.owner == user.key() @ DepositError::WrongOwner,
    )]
    pub user_x: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_y.mint == config.mint_y @ DepositError::WrongMint,
        constraint = user_y.owner == user.key() @ DepositError::WrongOwner,
    )]
    pub user_y: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_lp,
        associated_token::authority = user,
    )]
    pub user_lp: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Deposit>, amount: u64, max_x: u64, max_y: u64) -> Result<()> {
    require!(amount > 0, DepositError::ZeroAmount);
    require!(max_x > 0, DepositError::ZeroAmount);
    require!(max_y > 0, DepositError::ZeroAmount);

    let (x_amount, y_amount) = calculate_deposit(
        ctx.accounts.vault_x.amount,
        ctx.accounts.vault_y.amount,
        ctx.accounts.mint_lp.supply,
        amount,
        max_x,
        max_y,
    )?;

    require!(x_amount <= max_x, DepositError::SlippageExceeded);
    require!(y_amount <= max_y, DepositError::SlippageExceeded);

    cpi_transfer(
        ctx.accounts.user.to_account_info(),
        ctx.accounts.user_x.to_account_info(),
        ctx.accounts.vault_x.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        x_amount,
    )?;

    cpi_transfer(
        ctx.accounts.user.to_account_info(),
        ctx.accounts.user_y.to_account_info(),
        ctx.accounts.vault_y.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        y_amount,
    )?;

    let seed_bytes = ctx.accounts.config.seed.to_le_bytes();
    let bump = ctx.accounts.config.config_bump;
    let seeds: &[&[u8]] = &[b"config", &seed_bytes, &[bump]];
    let signer = &[seeds];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.mint_lp.to_account_info(),
            to: ctx.accounts.user_lp.to_account_info(),
            authority: ctx.accounts.config.to_account_info(),
        },
        signer,
    );
    mint_to(cpi_ctx, amount)?;

    msg!("deposit: x={} y={} lp_minted={}", x_amount, y_amount, amount);
    Ok(())
}

fn calculate_deposit(
    vault_x: u64,
    vault_y: u64,
    lp_supply: u64,
    lp_amount: u64,
    max_x: u64,
    max_y: u64,
) -> Result<(u64, u64)> {
    if vault_x == 0 && vault_y == 0 {
        return Ok((max_x, max_y));
    }

    let x = (lp_amount as u128)
        .checked_mul(vault_x as u128)
        .ok_or(DepositError::Overflow)?
        .checked_div(lp_supply as u128)
        .ok_or(DepositError::Overflow)? as u64;

    let y = (lp_amount as u128)
        .checked_mul(vault_y as u128)
        .ok_or(DepositError::Overflow)?
        .checked_div(lp_supply as u128)
        .ok_or(DepositError::Overflow)? as u64;

    Ok((x, y))
}

fn cpi_transfer<'a>(
    authority: AccountInfo<'a>,
    from: AccountInfo<'a>,
    to: AccountInfo<'a>,
    token_program: AccountInfo<'a>,
    amount: u64,
) -> Result<()> {
    let cpi_ctx = CpiContext::new(token_program, Transfer { from, to, authority });
    transfer(cpi_ctx, amount)
}

#[error_code]
pub enum DepositError {
    #[msg("Amount must be > 0")]
    ZeroAmount,
    #[msg("Slippage limit exceeded")]
    SlippageExceeded,
    #[msg("Math overflow")]
    Overflow,
    #[msg("Pool is locked")]
    PoolLocked,
    #[msg("Wrong token mint")]
    WrongMint,
    #[msg("Wrong token account owner")]
    WrongOwner,
}
