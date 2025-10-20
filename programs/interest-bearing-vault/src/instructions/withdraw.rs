use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        Mint, TokenAccount, TokenInterface
    }
};

use crate::state::*;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub withdrawer: Signer<'info>,

    /// The vault account - only the vault authority can withdraw
    #[account(
        mut,
        seeds = [b"vault", vault.vault_authority.as_ref()],
        bump = vault.bump,
        constraint = vault.vault_authority == withdrawer.key() @ crate::errors::VaultError::Unauthorized
    )]
    pub vault: Account<'info, Vault>,

    /// The mint associated with the vault
    pub mint: InterfaceAccount<'info, Mint>,

    /// The withdrawer's token account (must be vault authority)
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = withdrawer,
        associated_token::token_program = token_program,
    )]
    pub withdrawer_token_account: InterfaceAccount<'info, TokenAccount>,

    /// The vault's token reserve account
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program,
    )]
    pub vault_token_reserve: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: ExtraAccountMetaList account for transfer hook
    #[account(
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump,
        seeds::program = transfer_hook_program.key()
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,

    /// CHECK: Transfer hook program
    pub transfer_hook_program: UncheckedAccount<'info>,

    /// CHECK: Whitelist account for withdrawer (resolved via transfer hook)
    #[account(
        seeds = [b"whitelist", vault.key().as_ref()],
        bump,
        seeds::program = transfer_hook_program.key()
    )]
    pub vault_whitelist: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64, bump: u8) -> Result<()> {
        require!(amount > 0, crate::errors::VaultError::InvalidAmount);
        require!(
            self.vault.token_reserve_amount >= amount,
            crate::errors::VaultError::InsufficientFunds
        );

        // Transfer tokens from vault reserve to withdrawer using SPL Token-2022 onchain helper
        // This properly handles transfer hooks by including additional accounts
        let vault_authority = self.vault.vault_authority;
        let seeds = &[
            b"vault",
            vault_authority.as_ref(),
            &[bump],
        ];
        let signer_seeds = &[&seeds[..]];

        // IMPORTANT: Only pass EXTRA accounts (not source/mint/dest/authority) in additional_accounts
        let additional_accounts = vec![
            self.extra_account_meta_list.to_account_info(),
            self.vault_whitelist.to_account_info(),
            self.transfer_hook_program.to_account_info(),
        ];

        spl_token_2022::onchain::invoke_transfer_checked(
            &self.token_program.key(),
            self.vault_token_reserve.to_account_info(),
            self.mint.to_account_info(),
            self.withdrawer_token_account.to_account_info(),
            self.vault.to_account_info(),
            &additional_accounts,
            amount,
            self.mint.decimals,
            signer_seeds,
        )?;

        // Update vault state
        self.vault.token_reserve_amount = self.vault.token_reserve_amount
            .checked_sub(amount)
            .ok_or(crate::errors::VaultError::Underflow)?;

        msg!("Withdrew {} tokens from vault", amount);
        msg!("Remaining vault balance: {}", self.vault.token_reserve_amount);

        Ok(())
    }
}
