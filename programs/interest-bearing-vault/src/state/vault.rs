use anchor_lang::prelude::*;

pub const VAULT_SEED: &str = "vault";

#[account]
#[derive(InitSpace)]
pub struct Vault {
    pub vault_authority: Pubkey,
    pub mint : Pubkey,
    pub token_reserve: Pubkey,
    pub token_reserve_amount: u64,
    pub num_depositors: u64,
    pub bump:u8,
}

#[account]
#[derive(InitSpace)]
pub struct VaultRegistryEntry {
    pub vault: Pubkey,
    pub mint: Pubkey,
    pub token_balance: u64,
    pub num_withdraws: u64,
    pub num_deposits: u64,
  //  pub bump:u8,
}