use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use crate::state::Whitelist;

#[derive(Accounts)]
pub struct WhitelistOperations<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    /// CHECK: The wallet address to whitelist
    pub address: AccountInfo<'info>,

    /// CHECK: The Mint account
    pub mint: AccountInfo<'info>,

    // The whitelist PDA account
    #[account(
        init_if_needed,
        payer = admin,
        space = 8 + Whitelist::INIT_SPACE, // discriminator + pubkey + u8
        seeds = [b"whitelist", mint.key().as_ref(), address.key().as_ref()],
        bump
    )]
    pub whitelist_PDA: Account<'info, Whitelist>,

    pub system_program: Program<'info, System>,
}

impl<'info> WhitelistOperations<'info> {
    pub fn add_to_whitelist(&mut self, bump:u8) -> Result<()> {
        self.whitelist_PDA.address = self.address.key();
        self.whitelist_PDA.mint = self.mint.key();
        self.whitelist_PDA.bump = bump;

        msg!("Whitelist initialized for address: {}", self.address.key());
        Ok(())
    }

    pub fn remove_from_whitelist(&mut self) -> Result<()> {
        self.whitelist_PDA.close(self.admin.to_account_info())
    }
}
