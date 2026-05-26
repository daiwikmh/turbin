use anchor_lang::prelude::*;
use crate::state::Config;

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    // Only the stored authority may call this instruction
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
        // Reject if pool has no authority (fully permissionless) or signer is wrong
        constraint = config.authority == Some(authority.key()) @ UpdateConfigError::Unauthorized,
    )]
    pub config: Account<'info, Config>,
}

pub fn handler(
    ctx: Context<UpdateConfig>,
    new_fee: Option<u16>,
    new_locked: Option<bool>,
    new_authority: Option<Pubkey>,
) -> Result<()> {
    let config = &mut ctx.accounts.config;

    if let Some(fee) = new_fee {
        require!(fee <= 1000, UpdateConfigError::InvalidFee);
        config.fee = fee;
    }

    if let Some(locked) = new_locked {
        config.locked = locked;
    }

    // Passing Some(new_authority) updates it.
    // Passing None leaves it unchanged — use update_config with new_authority = None
    // and explicitly set it to Some(Pubkey::default()) to revoke (make permissionless).
    if let Some(authority) = new_authority {
        config.authority = Some(authority);
    }

    msg!(
        "config updated: fee={:?} locked={:?} authority={:?}",
        new_fee,
        new_locked,
        new_authority
    );
    Ok(())
}

#[error_code]
pub enum UpdateConfigError {
    #[msg("Only the pool authority can update config")]
    Unauthorized,
    #[msg("Fee must be <= 1000 basis points (10%)")]
    InvalidFee,
}
