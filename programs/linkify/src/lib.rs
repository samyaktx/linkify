use anchor_lang::prelude::*;

declare_id!("53Yz1VtkvMmp3ASE6gcL1dYBQAvXHTAvUJzHEFL8icgM");

#[program]
pub mod linkify {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
