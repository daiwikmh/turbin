use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{
    error::DiceError,
    state::{Bet, MAX_ROLL, MIN_BET_LAMPORTS, MIN_ROLL},
};

#[derive(Accounts)]
#[instruction(seed: u128)]
pub struct PlaceBet<'info> {
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
        init,
        payer = player,
        space = 8 + Bet::INIT_SPACE,
        seeds = [b"bet", vault.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub bet: Account<'info, Bet>,

    pub system_program: Program<'info, System>,
}

impl<'info> PlaceBet<'info> {
    pub fn create_bet(
        &mut self,
        seed: u128,
        roll: u8,
        amount: u64,
        bumps: &PlaceBetBumps,
    ) -> Result<()> {
        require!(roll >= MIN_ROLL && roll <= MAX_ROLL, DiceError::InvalidRoll);
        require!(amount >= MIN_BET_LAMPORTS, DiceError::BetTooSmall);

        self.bet.set_inner(Bet {
            slot: Clock::get()?.slot,
            player: self.player.key(),
            seed,
            amount,
            roll,
            bump: bumps.bet,
        });

        Ok(())
    }

    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        let accounts = Transfer {
            from: self.player.to_account_info(),
            to: self.vault.to_account_info(),
        };
        let ctx = CpiContext::new(self.system_program.to_account_info(), accounts);
        transfer(ctx, amount)
    }
}
