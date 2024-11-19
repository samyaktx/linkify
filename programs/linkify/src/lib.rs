use anchor_lang::prelude::*;

declare_id!("53Yz1VtkvMmp3ASE6gcL1dYBQAvXHTAvUJzHEFL8icgM");

#[program]
pub mod linkify {
    use super::*;

    pub fn create_user(ctx: Context<CreateUser>, name: String) -> Result<()> {
        let user_info = &mut ctx.accounts.user;
        user_info.name = name;
        user_info.user_pubkey = ctx.accounts.signer.key();
        user_info.req_sent_count = 0;
        user_info.req_received_count = 0;
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateUser<'info> {
   #[account(
        init,
        payer = signer,
        space = 8 + 32 + 4 + 4,
        seeds = [b"user", signer.key().as_ref()],
        bump
    )]
    pub user: Account<'info, UserInfo>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct UserInfo {
    pub user_pubkey: Pubkey,
    #[max_len(20)]
    pub name: String,
    pub req_sent_count: u32,
    pub req_received_count: u32,
}

#[account]
pub struct Connection {
    pub req_sender: Pubkey,
    pub req_receiver: Pubkey,
    pub connected: bool
}