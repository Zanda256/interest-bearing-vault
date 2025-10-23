use std::error::Error;
use anchor_lang::{prelude::*, solana_program::program::{invoke, invoke_signed},};
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


    #[account(
        mut,
        // constraint =  withdrawer,
        seeds = [b"vault_registry", vault.key().as_ref(), withdrawer.key().as_ref()],
        bump,
    )]
    pub vault_registry_entry: Account<'info, VaultRegistryEntry>,

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
        seeds = [b"whitelist", mint.key().as_ref(), withdrawer.key().as_ref()],
        bump,
        seeds::program = transfer_hook_program.key()
    )]
    pub withdrawer_whitelist_PDA: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        require!(amount > 0, crate::errors::VaultError::InvalidAmount);
        require!(
            self.vault_registry_entry.token_balance >= amount,
            crate::errors::VaultError::InsufficientFunds
        );
        require!(
            self.vault.token_reserve_amount >= amount,
            crate::errors::VaultError::InsufficientFunds
        );


        // Transfer tokens from vault reserve to withdrawer using SPL Token-2022 onchain helper
        // This properly handles transfer hooks by including additional accounts
        let vault_authority = self.vault.vault_authority;
        let bump = &[self.vault.bump];
        let seeds = &[
            b"vault".as_ref(),
            vault_authority.as_ref(),
            bump,
        ];

        // IMPORTANT: Only pass EXTRA accounts (not source/mint/dest/authority) in additional_accounts
        // let additional_accounts = vec![
        //     self.extra_account_meta_list.to_account_info(),
        //     self.withdrawer_whitelist_PDA.to_account_info(),
        //     self.transfer_hook_program.to_account_info(),
        // ];
        //

        let v_signer_meta = AccountMeta::new(self.vault.key(), true);

        let mut transfer_ix =  spl_token_2022::instruction::transfer_checked(
            &self.token_program.key(),
            &self.vault_token_reserve.key(),
            &self.mint.key(),
            &self.withdrawer_token_account.key(),
            &self.vault.key(),
            &[&self.withdrawer_token_account.key()],
            amount,
            self.mint.decimals,
        ).map_err(|e:ProgramError|{msg!("Unable to invoke instruction: {:?}", e); e } )?;

        transfer_ix.accounts.push(AccountMeta::new_readonly(self.extra_account_meta_list.key(), false));
        transfer_ix.accounts.push(AccountMeta::new(self.withdrawer_whitelist_PDA.key(), false));

        let account_infos = &[
            self.vault_token_reserve.to_account_info(),
            self.mint.to_account_info(),
            self.withdrawer_token_account.to_account_info(),
            self.vault.to_account_info(),
            self.token_program.to_account_info(), // The Token Program must be in this list for `invoke`
            self.extra_account_meta_list.to_account_info(),
            self.withdrawer_whitelist_PDA.to_account_info(),
            self.transfer_hook_program.to_account_info(),
        ];

        msg!("On to invoke instruction.");

        invoke_signed(&transfer_ix, account_infos, &[seeds])?;

        // spl_token_2022::onchain::invoke_transfer_checked(
        //     &self.token_program.key(),
        //     self.vault_token_reserve.to_account_info(),
        //     self.mint.to_account_info(),
        //     self.withdrawer_token_account.to_account_info(),
        //     self.vault.to_account_info(),
        //     &[
        //         self.extra_account_meta_list.to_account_info(),
        //         self.withdrawer_whitelist_PDA.to_account_info(),
        //         self.transfer_hook_program.to_account_info(),
        //     ],
        //     amount,
        //     self.mint.decimals,
        //     &[],
        // ).map_err(|e:ProgramError|{msg!("Unable to invoke instruction: {:?}", e); e } )?;

        // Update vault state
        self.vault.token_reserve_amount = self.vault.token_reserve_amount
            .checked_sub(amount)
            .ok_or(crate::errors::VaultError::Underflow)?;

        msg!("Withdrew {} tokens from vault", amount);
        msg!("Remaining vault balance: {}", self.vault.token_reserve_amount);

        // Update vault registry
        self.vault_registry_entry.token_balance = self.vault_registry_entry.token_balance
                .checked_sub(amount)
                .ok_or(crate::errors::VaultError::Overflow)?;
        self.vault_registry_entry.num_withdraws = self.vault_registry_entry.num_withdraws
                .checked_add(1)
                .ok_or(crate::errors::VaultError::Overflow)?;

        Ok(())
    }
}
