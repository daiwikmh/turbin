#![allow(unexpected_cfgs)]

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("AKWnQtCbpRfeifA6F34SjS9kRFJEaNvG6wtyhnEEXmSR");

#[derive(Accounts)]
pub struct Initialize {}

#[program]
pub mod nft_staking {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn program_id_matches_submission() {
        assert_eq!(id().to_string(), "AKWnQtCbpRfeifA6F34SjS9kRFJEaNvG6wtyhnEEXmSR");
    }

    #[test]
    fn account_space_constants_are_correct() {
        assert_eq!(StakeConfig::INIT_SPACE, 8);
        assert_eq!(UserAccount::INIT_SPACE, 6);
        assert_eq!(StakeAccount::INIT_SPACE, 73);
    }

    #[test]
    fn seed_derivations_are_deterministic() {
        let program_id = id();
        let (config_pda, _) = Pubkey::find_program_address(&[b"config"], &program_id);
        let user = Pubkey::new_unique();
        let (user_pda, _) = Pubkey::find_program_address(&[b"user", user.as_ref()], &program_id);
        let (rewards_pda, _) = Pubkey::find_program_address(&[b"rewards", config_pda.as_ref()], &program_id);

        assert_ne!(config_pda, user_pda);
        assert_ne!(rewards_pda, config_pda);
        assert_ne!(rewards_pda, user_pda);
    }
}
