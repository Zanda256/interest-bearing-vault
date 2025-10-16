use anchor_lang::prelude::*;

declare_id!("GaHcCA1SB8gjXCBG6ZDDo9d8j8F8fz5cRsSYgfhkuDzp");

#[program]
pub mod interest_bearing_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
