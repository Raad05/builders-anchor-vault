use crate::state::VaultState;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    // the user is the signer of this transaction, so we use the `Signer` type here
    #[account(mut)]
    pub user: Signer<'info>,

    // the state account is a PDA that will hold the vault's state, so we use the `init` attribute here to create it
    #[account(
        init,
        payer = user,
        seeds = [b"state", user.key().as_ref()],
        bump,
        space = 64 + VaultState::INIT_SPACE
    )]
    pub vault_state: Account<'info, VaultState>,

    // the vault account is a system account that will hold the funds, so we don't use the `init` attribute here
    #[account(
        seeds = [b"vault", vault_state.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    // the system program is required to create accounts (9-15 lines of code)
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: &InitializeBumps) -> Result<()> {
        // save the data to state or state account
        self.vault_state.vault_bump = bumps.vault;
        self.vault_state.state_bump = bumps.vault_state;

        Ok(())
    }
}
