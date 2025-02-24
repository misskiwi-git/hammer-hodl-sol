use anchor_lang::prelude::*;

declare_id!("6pj96wwKMC7U4SpNuzhQHyECtbEdKqQ75DZ8Dg1BSFFh");

#[program]
pub mod hammer_hodl_sol {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
