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

    // endregion:   --- Create User Function


    // region:   --- Update Username Function

    pub fn update_username(ctx: Context<UpdateUsername>, user_pubkey: Pubkey, username: String) -> Result<()> {
        let user_key = &ctx.accounts.user_info.user_pubkey;

        require!(user_key != &user_pubkey, Error::InvaildUserPubkey);
        let change_username = &mut ctx.accounts.user_info;
        change_username.username = username;
        Ok(())
    }

    // endregion:   --- Update Username Function


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
        connection.are_connected = false;

        let requester = &mut ctx.accounts.requester_acc;
        requester.req_sent_count += 1;

        Ok(())
    }

    // endregion:   --- Request Connection Function


    // region:   --- Accept Connection Function

    pub fn accept_connection(ctx: Context<AcceptConnection>) -> Result<()> {
        let acceptor_connection = &ctx.accounts.connection.acceptor; 
        let requester_connection = &ctx.accounts.connection.requester; 
        let acceptor_key = &ctx.accounts.acceptor.user_pubkey;
        let requester_key = &ctx.accounts.requester.user_pubkey;

        require!(acceptor_connection != acceptor_key, Error::IncorrectAcceptorPubkey);
        require!(requester_connection != requester_key, Error::IncorrectRequesterPubkey);
        require!(acceptor_key == requester_key, Error::SameAccountNotAllowed);

        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &acceptor_key.key(), 
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

        let connecion = &mut ctx.accounts.connection;
        connecion.are_connected = true;

        let acceptor = &mut ctx.accounts.acceptor;
        acceptor.req_checked_count += 1;

        Ok(())
    }

    // endregion:   --- Accept Connection Function


    // region:   --- Reject Connection Function

    pub fn reject_connection(ctx: Context<RejectConnection>) -> Result<()> {
        let acceptor_connection = &ctx.accounts.connection.acceptor; 
        let requester_connection = &ctx.accounts.connection.requester; 
        let acceptor_key = &ctx.accounts.acceptor.user_pubkey;
        let requester_key = &ctx.accounts.requester.user_pubkey;

        require!(acceptor_connection != acceptor_key, Error::IncorrectAcceptorPubkey);
        require!(requester_connection != requester_key, Error::IncorrectRequesterPubkey);
        require!(acceptor_key == requester_key, Error::SameAccountNotAllowed);

        let program_acc = &ctx.accounts.program_account;

        program_acc.sub_lamports(200_000_000)?;
        ctx.accounts.requester_pubkey.add_lamports(200_000_000)?;

        let acceptor = &mut ctx.accounts.acceptor;
        acceptor.req_checked_count += 1;

        Ok(())
    }

    // endregion:   --- Reject Connection Function

    // @TODO
    // withdraw stake
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


// region:   --- Update Username Instruction

#[derive(Accounts)]
pub struct UpdateUsername<'info> {
   #[account(
        mut,
        seeds = [b"user", signer.key().as_ref()],
        bump
    )]
    pub user_info: Account<'info, UserInfo>,
    #[account(mut)]
    pub signer: Signer<'info>,
}

// endregion:   --- Update Username Instruction



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


// region:   --- Accept Connection Instruction

#[derive(Accounts)]
pub struct AcceptConnection<'info> {
    #[account(
        mut,
        seeds = [b"connect", signer.key().as_ref(), &acceptor.req_checked_count.to_le_bytes()],
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
    pub acceptor: Account<'info, UserInfo>,
    #[account(
        mut,
        seeds = [b"user", connection.requester.key().as_ref()],
        bump
    )]
    pub requester: Account<'info, UserInfo>,
    #[account(mut)]
    pub program_account: AccountInfo<'info>,
    pub system_program: Program<'info, System>
}

// endregion:   --- Accept Connection Instruction


// region:   --- Reject Connection Instruction

#[derive(Accounts)]
pub struct RejectConnection<'info> {
    #[account(
        mut,
        close = requester_pubkey,
        seeds = [b"matched", signer.key().as_ref(), &acceptor.req_checked_count.to_le_bytes()],
        bump
    )]
    pub connection: Account<'info, Connection>,
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"user", signer.key().as_ref()],
        bump
    )]
    pub acceptor: Account<'info, UserInfo>, 
    #[account(
        mut,
        seeds = [b"user", connection.requester.key().as_ref()],
        bump
    )]
    pub requester: Account<'info, UserInfo>,
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account, we just need requester wallet address
    pub requester_pubkey: UncheckedAccount<'info>,
    #[account(mut)]
    pub program_account: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

// endregion:   --- Reject Connection Instruction


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
    pub are_connected: bool
}

// endregion:   --- Connection Account


// region:   --- Error Handling

#[error_code]
pub enum Error {
    #[msg("Invaild or Wrong user pubkey")]
    InvaildUserPubkey,
    #[msg("Invaild or Wrong Acceptor pubkey")]
    InvaildAcceptorPubkey,
    #[msg("Requester and Acceptor accounts cannot be the same.")]
    SameAccountNotAllowed,
    #[msg("Requester pubkey in UserInfo account is not equal in Connection account")]
    IncorrectRequesterPubkey,
    #[msg("Acceptor pubkey in UserInfo account is not equal in Connection account")]
    IncorrectAcceptorPubkey,
}

// endregion:   --- Error Handling