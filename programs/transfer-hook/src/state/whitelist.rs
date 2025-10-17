use anchor_lang::prelude::*;

#[account]
pub struct Whitelist {
    pub address: Pubkey,
    pub bump: u8,
}