mod state;
mod instructions;
mod errors;

#[cfg(test)]
mod tests;

use anchor_lang::prelude::*;

declare_id!("GaHcCA1SB8gjXCBG6ZDDo9d8j8F8fz5cRsSYgfhkuDzp");

use instructions::*;
use errors::*;

#[program]
pub mod interest_bearing_vault {
    use super::*;

    pub fn create_mint_with_extensions(
        ctx: Context<TokenFactory>,
        interest_rate: i16,
    ) -> Result<()> {
        ctx.accounts.init_mint(interest_rate)
    }

    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        ctx.accounts.initialize_vault(ctx.bumps.vault)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let bump = ctx.accounts.vault.bump;
        ctx.accounts.withdraw(amount, bump)
    }
}
