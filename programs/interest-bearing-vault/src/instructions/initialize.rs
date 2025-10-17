use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        Mint, TokenAccount, TokenInterface,
    }
};

use crate::state::*;

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    pub vault_authority: Signer<'info>,

    // IMPORTANT: Make sure this is read as an InterfaceAccount
    pub mint: InterfaceAccount<'info, Mint>,

    /// CHECK: Program id of the tf hook
    pub hook_program_id: UncheckedAccount<'info>,

    #[account(
        init, 
        payer = vault_authority, 
        space = 8 + Vault::INIT_SPACE,
        seeds = [b"vault", vault_authority.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        init,
        payer = vault_authority,
        associated_token::mint = mint,
        associated_token::authority = vault,
        associated_token::token_program = associated_token_program,
    )]
    pub token_reserve: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeVault<'info> {
    pub fn initialize_vault(&mut self, bump: u8) -> Result<()> {
        let vault = &mut self.vault;

        vault.vault_authority = self.vault_authority.key();
        vault.mint = self.mint.key();
        vault.token_reserve = self.token_reserve.key();
        vault.token_reserve_amount = 0;
        vault.num_depositors = 0;
        vault.bump = bump;

        Ok(())
    }
}