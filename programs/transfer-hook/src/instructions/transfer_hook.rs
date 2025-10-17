use std::cell::RefMut;
use anchor_lang::prelude::Pubkey;
use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::spl_token_2022::{
        extension::{
            transfer_hook::TransferHookAccount,
            BaseStateWithExtensionsMut,
            PodStateWithExtensionsMut,
        },
        pod::PodAccount
    },
    token_interface::{
        Mint,
        TokenAccount,
        TokenInterface
    }
};
use anchor_spl::token_2022::spl_token_2022::extension::transfer_hook::get_program_id;
use anchor_lang::accounts::interface_account::InterfaceAccount;
use anchor_spl::token_2022::spl_token_2022::extension::transfer_hook::instruction::TransferHookInstruction;
use crate::state::Whitelist;
use crate::errors::WhitelistError;

// use crate::instructions::is_whitelist_account;

#[derive(Accounts)]
pub struct TransferHook<'info> {
    #[account(
        token::mint = mint, 
        token::authority = owner,
    )]
    pub source_token: InterfaceAccount<'info, TokenAccount>,
    
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        token::mint = mint,
    )]
    pub destination_token: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: source token account owner, can be SystemAccount or PDA owned by another program
    pub owner: AccountInfo<'info>,
    /// CHECK: ExtraAccountMetaList Account,
    #[account(
        seeds = [b"extra-account-metas", mint.key().as_ref()], 
        bump
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,
    #[account(
        seeds = [
        b"whitelist",
        owner.key().as_ref(),
        ], 
        bump,
    )]
    pub user_whitelist_account: Account<'info, Whitelist>,
}

impl<'info> TransferHook<'info> {
    /// This function is called when the transfer hook is executed.
    pub fn transfer_hook(&mut self, _amount: u64) -> Result<()> {
        // Fail this instruction if it is not called from within a transfer hook
        self.check_is_transferring()?;

       // let user_whitelist_account_info = &self.user_whitelist_account.to_account_info();

        // let owner_key = self.owner.key();;
        // let seeds = &["whitelist".as_bytes(), owner_key.as_ref()];
        // let(whitelist_pda, u_bump) = Pubkey::find_program_address(seeds, &crate::ID);
        // if whitelist_pda != self.user_whitelist_account.key() {
        //     return Err(WhitelistError::AccountDoesNotMatch.into());
        // }
        // 
        // if self.user_whitelist_account.to_account_info().data_len() > 8 {
        //     if self.user_whitelist_account.address != self.owner.key() {
        //         return Err(WhitelistError::AccountDoesNotMatch.into());
        //     }
        //     if self.user_whitelist_account.bump != u_bump {
        //         return Err(WhitelistError::AccountDoesNotMatch.into());
        //     }
        //     msg!("transfer_hook : PDA is already initialized. User already on whitelist");
        // } else {
        //     msg!("PDA was not initialized.");
        //     return Err(WhitelistError::AccountNotWhitelisted.into());
        // }
        

        Ok(())
    }

    /// Checks if the transfer hook is being executed during a transfer operation.
    fn check_is_transferring(&mut self) -> Result<()> {
        // Ensure that the source token account has the transfer hook extension enabled

        // Get the account info of the source token account
        let source_token_info = self.source_token.to_account_info();
        // Borrow the account data mutably
        let mut account_data_ref: RefMut<&mut [u8]> = source_token_info.try_borrow_mut_data()?;

        // Unpack the account data as a PodStateWithExtensionsMut
        // This will allow us to access the extensions of the token account
        // We use PodStateWithExtensionsMut because TokenAccount is a POD (Plain Old Data) type
        let mut account = PodStateWithExtensionsMut::<PodAccount>::unpack(*account_data_ref)?;
        // Get the TransferHookAccount extension
        // Search for the TransferHookAccount extension in the token account
        // The returning struct has a `transferring` field that indicates if the account is in the middle of a transfer operation
        let account_extension = account.get_extension_mut::<TransferHookAccount>()?;

        // Check if the account is in the middle of a transfer operation
        if !bool::from(account_extension.transferring) {
            panic!("TransferHook: Not transferring");
        }

        Ok(())
    }

    // fallback instruction handler as workaround to anchor instruction discriminator check
    // pub fn fallback(
    //     &mut self,
    //     program_id: &Pubkey,
    // ) -> Result<()> {
    //     msg!("Transferred amount of an amount tokens" );
    //     Ok(())
    //     // let instruction = TransferHookInstruction::unpack(data)?;
    // 
    //     // match instruction discriminator to transfer hook interface execute instruction
    //     // token2022 program CPIs this instruction on token transfer
    //     // match instruction {
    //     //     TransferHookInstruction::Execute { amount } => {
    //     //         let amount_bytes = amount.to_le_bytes();
    //     // 
    //     //         msg!("Transferred amount is {}", amount);
    //     //         Ok(())
    //     //     }
    //     //     _ => return Err(ProgramError::InvalidInstructionData.into()),
    //     // }
    // }
}