use anchor_lang::prelude::*;

declare_id!("6cAZiTnevHt88rM8WyzaMTaUXQ7vB2hXnpRZW65Jrg2Z");

#[program]
pub mod transfer_hook {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
