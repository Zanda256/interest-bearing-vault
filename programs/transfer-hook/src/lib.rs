#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;
use spl_discriminator::SplDiscriminate;
use spl_transfer_hook_interface::{
    instruction::{
        ExecuteInstruction,
        InitializeExtraAccountMetaListInstruction,
    },
};
use spl_tlv_account_resolution::state::ExtraAccountMetaList;

use instructions::*;

mod instructions;
mod state;
mod errors;

use errors::*;

mod tests;

declare_id!("6cAZiTnevHt88rM8WyzaMTaUXQ7vB2hXnpRZW65Jrg2Z");


#[program]
pub mod transfer_hook {
    use super::*;
    
    pub fn add_to_whitelist(ctx: Context<WhitelistOperations>) -> Result<()> {
        ctx.accounts.add_to_whitelist(ctx.bumps.whitelist_PDA)
    }

    pub fn remove_from_whitelist(ctx: Context<WhitelistOperations>) -> Result<()> {
        ctx.accounts.remove_from_whitelist()
    }
    
    #[instruction(discriminator = InitializeExtraAccountMetaListInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn initialize_extra_accounts(ctx: Context<InitializeExtraAccountMetaList>) -> Result<()> { 
        msg!("Initializing Transfer Hook...");

        // Get the extra account metas for the transfer hook
        let extra_account_metas = InitializeExtraAccountMetaList::extra_account_metas()
            .map_err(|_| ExtraAccountMetaError::InvalidExtraAccountMeta)?;

        msg!("Extra Account Metas: {:?}", extra_account_metas);
        msg!("Extra Account Metas Length: {}", extra_account_metas.len());

        // initialize ExtraAccountMetaList account with extra accounts
        ExtraAccountMetaList::init::<ExecuteInstruction>(
            &mut ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?,
            &extra_account_metas
        ).map_err(|_| ExtraAccountMetaError::InvalidExtraAccountMeta)?;

        Ok(())
    }

    #[instruction(discriminator = ExecuteInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn transfer_hook(ctx: Context<TransferHook>, amount: u64) -> Result<()> {
        // Call the transfer hook logic
        ctx.accounts.transfer_hook(amount)
    }

    // #[instruction(discriminator = FallbackInstruction::SPL_DISCRIMINATOR_SLICE)]
    // pub fn fallback(ctx: Context<TransferHook>) -> Result<()> {
    //     ctx.accounts.fallback(&crate::ID)
    // }
}
