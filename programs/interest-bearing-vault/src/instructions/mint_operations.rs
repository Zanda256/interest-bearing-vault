use anchor_lang::prelude::*;
use anchor_lang::system_program::{create_account, CreateAccount};
use anchor_spl::token_interface::{Mint, TokenInterface};
use anchor_spl::token_2022::spl_token_2022::{
    pod::{PodMint},
    extension::{
        ExtensionType,
        interest_bearing_mint::{
            InterestBearingConfig,
            instruction::{
                InterestBearingMintInstruction,
                InitializeInstructionData,
                initialize as initialize_interest_bearing_mint_instruction,
            }
        },
        transfer_hook::{
            TransferHook,
            instruction::{
                initialize as initialize_transfer_hook_instruction,
                TransferHookInstruction
            }
        },
    },
    instruction::{initialize_mint2},
};

#[derive(Accounts)]
pub struct TokenFactory<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: We're initializing this manually with extensions
    #[account(
        mut,
        signer,
    )]
    pub mint: AccountInfo<'info>,

    /// CHECK: ExtraAccountMetaList Account, will be checked by the transfer hook
    #[account(mut)]
    pub extra_account_meta_list: UncheckedAccount<'info>,

    /// CHECK: The transfer hook program ID
    pub hook_program_id: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> TokenFactory<'info> {
    pub fn init_mint(&mut self, interest_rate: i16) -> Result<()> {
        let decimals = 9;

        // Calculate space needed for mint with extensions
        let space = ExtensionType::try_calculate_account_len::<PodMint>(&[
            ExtensionType::TransferHook,
            ExtensionType::InterestBearingConfig,
        ])?;

        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(space);

        // 1. Create the mint account
        create_account(
            CpiContext::new(
                self.system_program.to_account_info(),
                CreateAccount {
                    from: self.user.to_account_info(),
                    to: self.mint.to_account_info(),
                },
            ),
            lamports,
            space as u64,
            &self.token_program.key(),
        )?;

        // Could use this
        // transfer_hook_initialize<'info>(
        //     ctx: CpiContext<'_, '_, '_, 'info, TransferHookInitialize<'info>>,
        //     authority: Option<Pubkey>,
        //     transfer_hook_program_id: Option<Pubkey>,
        // ) -> Result<()>

        // 2. Initialize Transfer Hook Extension
        let init_transfer_hook_ix = initialize_transfer_hook_instruction(
            &self.token_program.key(),
            &self.mint.key(),
            Some(self.user.key()),
            Some(self.hook_program_id.key()),
        )?;

        anchor_lang::solana_program::program::invoke(
            &init_transfer_hook_ix,
            &[
                self.mint.to_account_info(),
                self.token_program.to_account_info(),
                self.user.to_account_info(),
                self.hook_program_id.to_account_info(),
            ],
        )?;

        // 3. Initialize Interest Bearing Extension
        let init_interest_ix = initialize_interest_bearing_mint_instruction(
            &self.token_program.key(),
            &self.mint.key(),
            Some(self.user.key()),
            interest_rate,
        )?;
        
        anchor_lang::solana_program::program::invoke(
            &init_interest_ix,
            &[
                self.token_program.to_account_info(),
                self.mint.to_account_info(),
                self.user.to_account_info(),
            ],
        )?;

        // 4. Initialize the mint itself
        let init_mint_ix = initialize_mint2(
            &self.token_program.key(),
            &self.mint.key(),
            &self.user.key(),
            Some( &self.user.key()),
            decimals,
        )?;

        anchor_lang::solana_program::program::invoke(
            &init_mint_ix,
            &[
                self.mint.to_account_info(),
            ],
        )?;

        msg!("Mint initialized with transfer hook and interest bearing extensions");
        Ok(())
    }
}
