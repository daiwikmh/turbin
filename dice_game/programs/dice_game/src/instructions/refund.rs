use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{
    constants::REFUND_COOLDOWN_SLOTS,
    error::DiceError,
    state::Bet,
};

#[derive(Accounts)]
pub struct RefundBet<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    /// CHECK: house pubkey used to derive the vault
    pub house: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"vault", house.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        mut,
        close = player,
        has_one = player,
        seeds = [b"bet", vault.key().as_ref(), bet.seed.to_le_bytes().as_ref()],
        bump = bet.bump,
    )]
    pub bet: Account<'info, Bet>,

    pub system_program: Program<'info, System>,
}

impl<'info> RefundBet<'info> {
    pub fn refund_bet(&mut self, bumps: &RefundBetBumps) -> Result<()> {
        let elapsed = Clock::get()?.slot.saturating_sub(self.bet.slot);
        require!(elapsed >= REFUND_COOLDOWN_SLOTS, DiceError::TimeoutNotReached);

        let house_key = self.house.key();
        let seeds: &[&[u8]] = &[b"vault", house_key.as_ref(), &[bumps.vault]];
        let signer_seeds = &[seeds];

        let accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.player.to_account_info(),
        };
        let ctx = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            accounts,
            signer_seeds,
        );
        transfer(ctx, self.bet.amount)
    }
}
