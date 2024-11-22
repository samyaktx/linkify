use anchor_lang::prelude::*;

declare_id!("gmMaMu6yguyX8HMWmfDRkkXe8iDSiRNvXv75QvwyBRR");

#[program]
pub mod linkify {
    use super::*;

    // region:   --- Create User Function

    pub fn create_user(ctx: Context<CreateUser>, username: String) -> Result<()> {
        let user_info = &mut ctx.accounts.user_info;
        user_info.username = username;
        user_info.user_pubkey = ctx.accounts.signer.key();
        user_info.req_sent_count = 0;
        user_info.req_checked_count = 0;
        
        Ok(())
    }

    // endregion:   --- Create User

    // region:   --- Request Connection Function

    pub fn request_connection(ctx: Context<RequestConnection>, acceptor_pubkey: Pubkey) -> Result<()> {
        let requester_key = &ctx.accounts.requester_acc.user_pubkey;
        let acceptor_key = &ctx.accounts.acceptor_acc.user_pubkey;

        require!(acceptor_key != &acceptor_pubkey, Error::InvaildAcceptorPubkey);
        require!(acceptor_key == requester_key, Error::SameAccountNotAllowed);

        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.signer.key(), 
            &ctx.accounts.program_account.key(), 
            2_000_000
        );

        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.signer.to_account_info(),
                ctx.accounts.program_account.to_account_info()
            ]
        )?;

        let connection = &mut ctx.accounts.connection;
        connection.requester = *requester_key;
        connection.acceptor = *acceptor_key;
        connection.connected = false;

        let requester = &mut ctx.accounts.requester_acc;
        requester.req_sent_count += 1;

        Ok(())
    }

    // endregion:   --- Request Connection

}

// region:   --- Create User Instruction

#[derive(Accounts)]
pub struct CreateUser<'info> {
   #[account(
        init,
        payer = signer,
        space = 8 + 32 + UserInfo::INIT_SPACE + 4 + 4,
        seeds = [b"user", signer.key().as_ref()],
        bump
    )]
    pub user_info: Account<'info, UserInfo>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// endregion:   --- Create User Instruction


// region:   --- Request Connection Instruction

#[derive(Accounts)]
pub struct RequestConnection<'info> {
    #[account(
        init,
        payer = signer,
        space = 8 + 32 + 32 + 1,
        seeds = [b"connect", acceptor_acc.user_pubkey.key().as_ref(), &requester_acc.req_sent_count.to_le_bytes()],
        bump
    )]
    pub connection: Account<'info, Connection>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"user", signer.key().as_ref()],
        bump
    )]
    pub requester_acc: Account<'info, UserInfo>,
    #[account(
        mut,
        seeds = [b"user", acceptor_acc.user_pubkey.key().as_ref()],
        bump
    )]
    pub acceptor_acc: Account<'info, UserInfo>,
    #[account(mut)]
    pub program_account: AccountInfo<'info>,
    pub system_program: Program<'info, System>
}

// endregion:   --- Request Connection Instruction


// region:   --- UserInfo Account

#[account]
#[derive(InitSpace)]
pub struct UserInfo {
    pub user_pubkey: Pubkey,
    #[max_len(20)]
    pub username: String,
    pub req_sent_count: u32,
    pub req_checked_count: u32,
}

// endregion:   --- UserInfo Account


// region:   --- Connection Account

#[account]
pub struct Connection {
    pub requester: Pubkey,
    pub acceptor: Pubkey,
    pub connected: bool
}

// endregion:   --- Connection Account


// region:   --- Error Handling

#[error_code]
pub enum Error {
    #[msg("Invaild or Wrong Acceptor pubkey")]
    InvaildAcceptorPubkey,
    #[msg("Requester and Acceptor accounts cannot be the same.")]
    SameAccountNotAllowed
}

// endregion:   --- Error Handling