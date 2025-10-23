use anchor_lang::{prelude::*, solana_program::program::{invoke}};
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        Mint, TokenAccount, TokenInterface,
        transfer_checked, TransferChecked,
    }
};
use spl_tlv_account_resolution::state::ExtraAccountMetaList;
use spl_token_2022::{onchain, instruction as spl_2022_instruction};
use spl_transfer_hook_interface::instruction::ExecuteInstruction;
use crate::state::*;

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    /// The vault account
    #[account(
        mut,
        seeds = [b"vault", vault.vault_authority.as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        init_if_needed,
        payer = depositor,
        space = 8 + VaultRegistryEntry::INIT_SPACE,
        seeds = [b"vault_registry", vault.key().as_ref(), depositor.key().as_ref()],
        bump,
    )]
    pub vault_registry_entry: Account<'info, VaultRegistryEntry>,

    /// The mint associated with the vault
    #[account(
        mut,
        extensions::transfer_hook::program_id = transfer_hook_program.key(),
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    /// The depositor's token account
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = depositor,
        associated_token::token_program = token_program,
    )]
    pub depositor_token_account: InterfaceAccount<'info, TokenAccount>,

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
    pub extra_account_meta_list: AccountInfo<'info>,

    /// CHECK: Transfer hook program
    pub transfer_hook_program: UncheckedAccount<'info>,

    /// CHECK: Whitelist account for depositor (resolved via transfer hook)
    #[account(
        mut,
        seeds = [b"whitelist", mint.key().as_ref(), depositor.key().as_ref()],
        bump,
        seeds::program = transfer_hook_program.key()
    )]
    pub depositor_whitelist_PDA: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64, registry_bump:u8) -> Result<()> {
        require!(amount > 0, crate::errors::VaultError::InvalidAmount);
        
        msg!("Deposit has been initiated");
       
        onchain::invoke_transfer_checked(
            &self.token_program.key(),
            self.depositor_token_account.to_account_info(),
            self.mint.to_account_info(),
            self.vault_token_reserve.to_account_info(),
            self.depositor.to_account_info(),
            &[
                self.extra_account_meta_list.to_account_info(),
                self.depositor_whitelist_PDA.to_account_info(),
                self.transfer_hook_program.to_account_info(),
            ],
            amount,
            self.mint.decimals,
            &[], // No signer seeds needed - depositor is already a signer
        )?;
       
       // msg!("Deposited {} tokens to PDA vault", amount);
        
        // Update vault state
        self.vault.token_reserve_amount = self.vault.token_reserve_amount
            .checked_add(amount)
            .ok_or(crate::errors::VaultError::Overflow)?;
        
        // Increment depositor count for deposit
        self.vault.num_depositors = self.vault.num_depositors
            .checked_add(1)
            .ok_or(crate::errors::VaultError::Overflow)?;
        
        // msg!("Deposited {} tokens to vault reserve {:?}", amount, &self.vault_token_reserve.key());
        // msg!("Total vault balance: {}", self.vault.token_reserve_amount);
        // msg!("Total depositors: {}", self.vault.num_depositors);
        
        // Update vault registry
        let v = VaultRegistryEntry{
            user: self.depositor.key(),
            user_ata: self.depositor_token_account.key(),
            vault: self.vault.key(),
            mint: self.mint.key(),
            token_balance: self.vault_registry_entry.token_balance
                .checked_add(amount)
                .ok_or(crate::errors::VaultError::Overflow)?,
            num_withdraws: self.vault_registry_entry.num_withdraws,
            num_deposits: self.vault_registry_entry.num_deposits
                .checked_add(1)
                .ok_or(crate::errors::VaultError::Overflow)?,
            bump: registry_bump
        };
        self.vault_registry_entry.set_inner(v);
        // 
        // self.vault_registry_entry.vault = self.vault.key();
        // self.vault_registry_entry.mint = self.mint.key();
        // self.vault_registry_entry.token_balance = self.vault_registry_entry.token_balance
        //         .checked_add(amount)
        //         .ok_or(crate::errors::VaultError::Overflow)?;
        // self.vault_registry_entry.deposits = self.vault_registry_entry.num_deposits
        //         .checked_add(1)
        //         .ok_or(crate::errors::VaultError::Overflow)?;
        
        Ok(())
    }
}


// ====================================================================================================================

