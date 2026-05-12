use anchor_lang::prelude::*;

#[derive(InitSpace)]
#[account]
pub struct VaultState {
    pub vault_bump: u8, // access this data to derive the vault account PDA
    pub state_bump: u8, // access this data to derive the state account PDA
}
