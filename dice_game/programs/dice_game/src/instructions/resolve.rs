use anchor_lang::{
    prelude::*,
    solana_program::sysvar::instructions::load_instruction_at_checked,
    system_program::{transfer, Transfer},
};

use crate::{
    constants::MEMO_PROGRAM_ID,
    error::DiceError,
    state::{Bet, HouseRoll, HOUSE_EDGE_BASIS_POINTS},
};

#[derive(Accounts)]
pub struct ResolveBet<'info> {
    #[account(mut)]
    pub house: Signer<'info>,

    #[account(mut)]
    pub player: SystemAccount<'info>,

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

    /// CHECK: instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instruction_sysvar: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> ResolveBet<'info> {
    pub fn read_house_roll(&self) -> Result<HouseRoll> {
        let note = load_instruction_at_checked(0, &self.instruction_sysvar)?;

        require_keys_eq!(note.program_id, MEMO_PROGRAM_ID, DiceError::InvalidMemoProgram);

        let text = std::str::from_utf8(&note.data).map_err(|_| DiceError::InvalidHouseRoll)?;
        let mut parts = text.split(',');
        let seed: u128 = parts
            .next()
            .and_then(|s| s.parse().ok())
            .ok_or(DiceError::InvalidHouseRoll)?;
        let randomness: u64 = parts
            .next()
            .and_then(|s| s.parse().ok())
            .ok_or(DiceError::InvalidHouseRoll)?;

        require!(seed == self.bet.seed, DiceError::SeedMismatch);

        Ok(HouseRoll { seed, randomness })
    }

    pub fn resolve_bet(&mut self, bumps: &ResolveBetBumps, house_roll: HouseRoll) -> Result<()> {
        let roll = (house_roll.randomness % 100) as u8 + 1;

        if self.bet.roll > roll {
            let payout = (self.bet.amount as u128)
                .checked_mul(10_000 - HOUSE_EDGE_BASIS_POINTS as u128)
                .ok_or(DiceError::Overflow)?
                .checked_div((self.bet.roll - 1) as u128)
                .ok_or(DiceError::Overflow)?
                .checked_div(100)
                .ok_or(DiceError::Overflow)? as u64;

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
            transfer(ctx, payout)?;
        }

        Ok(())
    }
}
