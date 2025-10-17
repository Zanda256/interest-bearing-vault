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