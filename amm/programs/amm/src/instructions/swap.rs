use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};
use crate::state::Config;

// Protocol receives 1/5 (20%) of the LP fee; the rest stays in the vault for LPs.
// Example: fee = 30 bps → 6 bps to treasury, 24 bps to LPs.
const PROTOCOL_SHARE: u128 = 5;

#[derive(Accounts)]
pub struct Swap<'info> {
    // config comes first — vault constraints reference config.mint_x / config.mint_y
    #[account(
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
        constraint = !config.locked @ SwapError::PoolLocked,
    )]
    pub config: Account<'info, Config>,

    // mints before vaults that reference them
    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,

    #[account(mut)]
    pub user: Signer<'info>,

    // pool vaults — owned by config PDA
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

    // user's source and destination token accounts
    #[account(
        mut,
        constraint = user_source.owner == user.key() @ SwapError::WrongOwner,
    )]
    pub user_source: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_destination.owner == user.key() @ SwapError::WrongOwner,
    )]
    pub user_destination: Account<'info, TokenAccount>,

    // Treasury ATA for the source token.
    // Pass vault_source here when no treasury is configured — the handler checks config.treasury.
    #[account(mut)]
    pub treasury_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<Swap>, amount_in: u64, minimum_amount_out: u64) -> Result<()> {
    require!(amount_in > 0, SwapError::ZeroAmount);
    require!(minimum_amount_out > 0, SwapError::ZeroAmount);

    let is_x_to_y = ctx.accounts.user_source.mint == ctx.accounts.config.mint_x;

    // Validate direction
    if is_x_to_y {
        require!(
            ctx.accounts.user_source.mint == ctx.accounts.mint_x.key(),
            SwapError::WrongMint
        );
        require!(
            ctx.accounts.user_destination.mint == ctx.accounts.mint_y.key(),
            SwapError::WrongMint
        );
    } else {
        require!(
            ctx.accounts.user_source.mint == ctx.accounts.mint_y.key(),
            SwapError::WrongMint
        );
        require!(
            ctx.accounts.user_destination.mint == ctx.accounts.mint_x.key(),
            SwapError::WrongMint
        );
    }

    let (vault_in, vault_out) = if is_x_to_y {
        (ctx.accounts.vault_x.amount, ctx.accounts.vault_y.amount)
    } else {
        (ctx.accounts.vault_y.amount, ctx.accounts.vault_x.amount)
    };

    require!(vault_in > 0 && vault_out > 0, SwapError::InsufficientLiquidity);

    let fee_bps = ctx.accounts.config.fee as u128;
    let amount_in_u128 = amount_in as u128;

    // Split fee: 20% to treasury, 80% stays in vault as LP fee
    let protocol_fee = amount_in_u128
        .checked_mul(fee_bps)
        .ok_or(SwapError::Overflow)?
        .checked_div(10000 * PROTOCOL_SHARE)
        .ok_or(SwapError::Overflow)?;

    let lp_fee_bps = fee_bps
        .checked_sub(fee_bps / PROTOCOL_SHARE)
        .ok_or(SwapError::Overflow)?;

    // Amount fed into constant-product after protocol fee is taken out
    let amount_after_protocol = amount_in_u128
        .checked_sub(protocol_fee)
        .ok_or(SwapError::Overflow)?;

    // Apply LP fee to what remains — constant product formula
    let amount_in_cp = amount_after_protocol
        .checked_mul(10000 - lp_fee_bps)
        .ok_or(SwapError::Overflow)?
        .checked_div(10000)
        .ok_or(SwapError::Overflow)?;

    // amount_out = vault_out * amount_in_cp / (vault_in + amount_in_cp)
    let numerator = (vault_out as u128)
        .checked_mul(amount_in_cp)
        .ok_or(SwapError::Overflow)?;
    let denominator = (vault_in as u128)
        .checked_add(amount_in_cp)
        .ok_or(SwapError::Overflow)?;
    let amount_out_u128 = numerator
        .checked_div(denominator)
        .ok_or(SwapError::Overflow)?;

    require!(amount_out_u128 <= u64::MAX as u128, SwapError::Overflow);
    let amount_out = amount_out_u128 as u64;

    require!(amount_out >= minimum_amount_out, SwapError::SlippageExceeded);

    // 1. User → source vault (full amount_in; user signs)
    {
        let vault_source = if is_x_to_y {
            ctx.accounts.vault_x.to_account_info()
        } else {
            ctx.accounts.vault_y.to_account_info()
        };
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_source.to_account_info(),
                to: vault_source,
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        transfer(cpi_ctx, amount_in)?;
    }

    // 2. Source vault → treasury ATA (protocol fee); config PDA signs
    let has_treasury = ctx.accounts.config.treasury.is_some();
    if has_treasury && protocol_fee > 0 {
        require!(protocol_fee <= u64::MAX as u128, SwapError::Overflow);

        let seed_bytes = ctx.accounts.config.seed.to_le_bytes();
        let bump = ctx.accounts.config.config_bump;
        let seeds: &[&[u8]] = &[b"config", &seed_bytes, &[bump]];
        let signer = &[seeds];

        let vault_source = if is_x_to_y {
            ctx.accounts.vault_x.to_account_info()
        } else {
            ctx.accounts.vault_y.to_account_info()
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: vault_source,
                to: ctx.accounts.treasury_ata.to_account_info(),
                authority: ctx.accounts.config.to_account_info(),
            },
            signer,
        );
        transfer(cpi_ctx, protocol_fee as u64)?;
    }

    // 3. Destination vault → user; config PDA signs
    {
        let seed_bytes = ctx.accounts.config.seed.to_le_bytes();
        let bump = ctx.accounts.config.config_bump;
        let seeds: &[&[u8]] = &[b"config", &seed_bytes, &[bump]];
        let signer = &[seeds];

        let vault_dest = if is_x_to_y {
            ctx.accounts.vault_y.to_account_info()
        } else {
            ctx.accounts.vault_x.to_account_info()
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: vault_dest,
                to: ctx.accounts.user_destination.to_account_info(),
                authority: ctx.accounts.config.to_account_info(),
            },
            signer,
        );
        transfer(cpi_ctx, amount_out)?;
    }

    msg!(
        "swap: in={} protocol_fee={} out={}",
        amount_in,
        if has_treasury { protocol_fee } else { 0 },
        amount_out
    );
    Ok(())
}

#[error_code]
pub enum SwapError {
    #[msg("Amount must be > 0")]
    ZeroAmount,
    #[msg("Slippage limit exceeded")]
    SlippageExceeded,
    #[msg("Math overflow")]
    Overflow,
    #[msg("Pool is locked")]
    PoolLocked,
    #[msg("Insufficient liquidity in pool")]
    InsufficientLiquidity,
    #[msg("Wrong token mint for this direction")]
    WrongMint,
    #[msg("Wrong token account owner")]
    WrongOwner,
}
