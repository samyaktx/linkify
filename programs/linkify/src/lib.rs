use anchor_lang::prelude::*;
use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

declare_id!("GZWBJp4oydN6d17NuHaDYVmJhQwhAAbdKStgPh64vr1r");

#[program]
pub mod linkify {
    use super::*;

    // region:   --- Create User Function

    pub fn create_user(ctx: Context<CreateUser>, username: String) -> Result<()> {
        let user_info = &mut ctx.accounts.user_info;
        user_info.username = username;
        user_info.user_pubkey = ctx.accounts.signer.key();
        user_info.req_received_count = 0;
        user_info.req_checked_count = 0;
        user_info.staked_count = 0;
        user_info.unstaked_count = 0;

        Ok(())
    }

    // endregion:   --- Create User Function

    // region:   --- Update Username Function

    pub fn update_username(
        ctx: Context<UpdateUsername>,
        user_pubkey: Pubkey,
        username: String,
    ) -> Result<()> {
        let user_key = &ctx.accounts.user_info.user_pubkey;

        require!(user_key == &user_pubkey, Error::InvaildUserPubkey);
        let change_username = &mut ctx.accounts.user_info;
        change_username.username = username;
        Ok(())
    }

    // endregion:   --- Update Username Function

    // region:   --- Request Connection Function

    pub fn request_connection(
        ctx: Context<RequestConnection>,
        acceptor_pubkey: Pubkey,
    ) -> Result<()> {
        let requester_key = &ctx.accounts.requester_acc.user_pubkey;
        let acceptor_key = &ctx.accounts.acceptor_acc.user_pubkey;

        require!(acceptor_key == &acceptor_pubkey, Error::InvaildAcceptorPubkey);
        require!(acceptor_key != requester_key, Error::SameAccountNotAllowed);

        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.signer.key(),
            &ctx.accounts.connection.key(),
            LAMPORTS_PER_SOL / 10 * 2,
        );

        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.signer.to_account_info(),
                ctx.accounts.connection.to_account_info(),
            ],
        )?;

        let connection = &mut ctx.accounts.connection;
        connection.requester = *requester_key;
        connection.acceptor = *acceptor_key;
        connection.are_connected = false;

        let acceptor = &mut ctx.accounts.acceptor_acc;
        acceptor.req_received_count += 1;

        Ok(())
    }

    // endregion:   --- Request Connection Function

    // region:   --- Accept Connection Function

    pub fn accept_connection(
        ctx: Context<AcceptConnection>,
        requester_pubkey: Pubkey,
    ) -> Result<()> {
        let acceptor_connect = &ctx.accounts.connection.acceptor;
        let requester_connect = &ctx.accounts.connection.requester;
        let acceptor_key = &ctx.accounts.acceptor_acc.user_pubkey;
        let requester_key = &ctx.accounts.requester_acc.user_pubkey;

        require!(acceptor_connect == acceptor_key, Error::IncorrectAcceptorPubkey);
        require!(requester_connect == requester_key, Error::IncorrectRequesterPubkey);
        require!(requester_key == &requester_pubkey, Error::IncorrectRequesterPubkey);
        require!(acceptor_key != requester_key, Error::SameAccountNotAllowed);

        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.signer.key(),
            &ctx.accounts.connection.key(),
            LAMPORTS_PER_SOL / 10 * 2,
        );

        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.signer.to_account_info(),
                ctx.accounts.connection.to_account_info(),
            ],
        )?;

        let connecion = &mut ctx.accounts.connection;
        connecion.are_connected = true;

        let acceptor = &mut ctx.accounts.acceptor_acc;
        acceptor.req_checked_count += 1;

        let requester = &mut ctx.accounts.requester_acc;
        requester.staked_count += 1;

        Ok(())
    }

    // endregion:   --- Accept Connection Function
  
    // region:   --- Reject Connection Function

    pub fn reject_connection(ctx: Context<RejectConnection>, denialist_pubkey: Pubkey) -> Result<()> {
        let denialist_connection = &ctx.accounts.connection.acceptor;
        let requester_connection = &ctx.accounts.connection.requester;
        let denialist_key = &ctx.accounts.denialist_acc.user_pubkey;
        let requester_key = &ctx.accounts.requester_acc.user_pubkey;

        require!(denialist_connection == denialist_key, Error::IncorrectAcceptorPubkey);
        require!(requester_connection == requester_key, Error::IncorrectRequesterPubkey);
        require!(denialist_key == &denialist_pubkey, Error::IncorrectRejectorPubkey);
        require!(denialist_key != requester_key, Error::SameAccountNotAllowed);

        let connection_acc = &ctx.accounts.connection;

        connection_acc.sub_lamports(LAMPORTS_PER_SOL / 10 * 2)?;
        ctx.accounts.requester_pubkey.add_lamports(LAMPORTS_PER_SOL / 10 * 2)?;

        let denialist = &mut ctx.accounts.denialist_acc;
        denialist.req_checked_count += 1;

        let requester = &mut ctx.accounts.requester_acc;
        requester.unstaked_count += 1;

        Ok(())
    }


    // endregion:   --- Reject Connection Function

    // region:   --- Withdraw Stake Function

    pub fn withdraw_stake(ctx: Context<WithdrawStake>, signer: Pubkey) -> Result<()> {
        let requester_unstaking = &ctx.accounts.requester_acc.user_pubkey;
        let acceptor_unstaking = &ctx.accounts.acceptor_acc.user_pubkey;
        let are_connected = &ctx.accounts.connection.are_connected;

        require!(*are_connected != false, Error::AcceptorRequesterAreNotConnected);
        require!(acceptor_unstaking == &signer || requester_unstaking == &signer, Error::InvaildUserPubkey);

        let connection_acc = &ctx.accounts.connection;
        connection_acc.sub_lamports(LAMPORTS_PER_SOL / 10 * 4)?;
        ctx.accounts.requester_pubkey.add_lamports(LAMPORTS_PER_SOL / 10 * 2)?;
        ctx.accounts.acceptor_pubkey.add_lamports(LAMPORTS_PER_SOL / 10 * 2)?;

        let unstaking_count = &mut ctx.accounts.requester_acc.unstaked_count;
        *unstaking_count += 1;
        Ok(())
    }

    // endregion:   --- Withdraw Stake Function
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
#[instruction(user_pubkey: Pubkey)]
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
#[instruction(acceptor_pubkey: Pubkey)]
pub struct RequestConnection<'info> {
    #[account(
        init,
        payer = signer,
        space = 8 + 32 + 32 + 1,
        seeds = [b"connect", acceptor_acc.user_pubkey.key().as_ref(), &acceptor_acc.req_received_count.to_le_bytes()],
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
    pub system_program: Program<'info, System>,
}

// endregion:   --- Request Connection Instruction


// region:   --- Accept Connection Instruction

#[derive(Accounts)]
#[instruction(requester_pubkey: Pubkey)]
pub struct AcceptConnection<'info> {
    #[account(
        mut,
        seeds = [b"connect", signer.key().as_ref(), &acceptor_acc.req_checked_count.to_le_bytes()],
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
    pub acceptor_acc: Account<'info, UserInfo>,
    #[account(
        mut,
        seeds = [b"user", connection.requester.key().as_ref()],
        bump
    )]
    pub requester_acc: Account<'info, UserInfo>,
    pub system_program: Program<'info, System>,
}

// endregion:   --- Accept Connection Instruction


// region:   --- Reject Connection Instruction

#[derive(Accounts)]
#[instruction(denialist_pubkey: Pubkey)]
pub struct RejectConnection<'info> {
    #[account(
        mut,
        close = requester_pubkey,
        seeds = [b"connect", signer.key().as_ref(), &denialist_acc.req_checked_count.to_le_bytes()],
        bump
    )]
    pub connection: Account<'info, Connection>,
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"user", signer.key().as_ref()],
        bump
    )]
    pub denialist_acc: Account<'info, UserInfo>,
    #[account(
        mut,
        seeds = [b"user", connection.requester.key().as_ref()],
        bump
    )]
    pub requester_acc: Account<'info, UserInfo>,
    /// CHECK: This is safe because we only use this account for transferring SOL back
    #[account(mut)]
    pub requester_pubkey: UncheckedAccount<'info>,
}

// endregion:   --- Reject Connection Instruction


// region:   --- Withdraw Stake Instruction

// #[derive(Accounts)]
// #[instruction(signer: Pubkey)]
// pub struct WithdrawStake<'info> {
//     #[account(
//         mut,
//         close = signer,
//         constraint =  acceptor_acc.user_pubkey.key() == signer.key() || requester_acc.user_pubkey.key() == signer.key(),
//     )]
//     pub connection: Account<'info, Connection>,
//     #[account(
//         mut,
//         seeds = [b"user", signer.key().as_ref()],
//         bump
//     )]
//     pub acceptor_acc: Account<'info, UserInfo>,
//     #[account(
//         mut,
//         seeds = [b"user", requester_acc.user_pubkey.key().as_ref()],
//         bump
//     )]
//     pub requester_acc: Account<'info, UserInfo>,
//     #[account(mut)]
//     pub signer: Signer<'info>,
//     /// CHECK: This is safe because we only use this account for transferring SOL
//     #[account(mut)]
//     pub requester_key: UncheckedAccount<'info>,
//     /// CHECK: This is safe because we only use this account for transferring SOL
//     #[account(mut)]
//     pub acceptor_key: UncheckedAccount<'info>
// }


#[derive(Accounts)]
#[instruction(signer: Pubkey)]
pub struct WithdrawStake<'info> {
    #[account(
        mut,
        close = signer,
        // constraint =  acceptor_acc.user_pubkey.key() == signer.key() || requester_acc.user_pubkey.key() == signer.key(),
        seeds = [b"connect", acceptor_acc.user_pubkey.key().as_ref(), &requester_acc.unstaked_count.to_le_bytes()],
        bump
    )]
    pub connection: Account<'info, Connection>,
    #[account(
        mut,
        seeds = [b"user", acceptor_acc.user_pubkey.key().as_ref()],
        bump
    )]
    pub acceptor_acc: Account<'info, UserInfo>,
    #[account(
        mut,
        seeds = [b"user", requester_acc.user_pubkey.key().as_ref()],
        bump
    )]
    pub requester_acc: Account<'info, UserInfo>,
    #[account(mut)]
    pub signer: Signer<'info>,
    /// CHECK: This is safe because we only use this account for transferring SOL
    #[account(mut)]
    pub requester_pubkey: UncheckedAccount<'info>,
    /// CHECK: This is safe because we only use this account for transferring SOL
    #[account(mut)]
    pub acceptor_pubkey: UncheckedAccount<'info>,
}

// endregion:   --- Withdraw Stake Instruction


// region:   --- UserInfo Account

#[account]
#[derive(InitSpace)]
pub struct UserInfo {
    pub user_pubkey: Pubkey,
    #[max_len(20)]
    pub username: String,
    pub req_received_count: u32,
    pub req_checked_count: u32,
    pub staked_count: u32,
    pub unstaked_count: u32,
}

// endregion:   --- UserInfo Account


// region:   --- Connection Account

#[account]
pub struct Connection {
    pub requester: Pubkey,
    pub acceptor: Pubkey,
    pub are_connected: bool,
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
    #[msg("Rejector pubkey is not matched")]
    IncorrectRejectorPubkey,
    #[msg("Acceptor and Requester are not connected")]
    AcceptorRequesterAreNotConnected,
}

// endregion:   --- Error Handling