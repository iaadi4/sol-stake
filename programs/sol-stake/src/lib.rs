use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, Token};

declare_id!("E8E9zijhTNpPPuexRFYjiyqstFwHoJN7mNPNLvPpv6F1");

#[program]
pub mod sol_stake {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, reward_rate: u64, stake_cap: u64) -> Result<()> {
        let reward_token_mint = &mut ctx.accounts.reward_token_mint;
        let stake_token_mint = &mut ctx.accounts.stake_token_mint;
        let stake_token_vault = &mut ctx.accounts.stake_token_vault;
        let reward_token_vault = &mut ctx.accounts.reward_token_vault;
        
        let clock = &ctx.accounts.clock;

        let authority = &mut ctx.accounts.authority;
        let pool = &mut ctx.accounts.pool;

        pool.authority = authority.key();
        pool.reward_rate = reward_rate;
        pool.reward_token_mint = reward_token_mint.key();
        pool.stake_token_mint = stake_token_mint.key();
        pool.stake_token_vault = stake_token_vault.key();
        pool.reward_token_vault = reward_token_vault.key();
        pool.stake_cap = stake_cap;
        pool.total_staked = 0;
        pool.bump = ctx.bumps.pool;

        pool.created_at_ts = clock.unix_timestamp;
        pool.created_at_epoch = clock.epoch;
        pool.updated_at_ts = clock.unix_timestamp;
        pool.updated_at_epoch = clock.epoch;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)] pub user: Signer<'info>,
    #[account(mut)] pub reward_token_mint: Account<'info, Mint>,
    #[account(mut)] pub stake_token_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = user,
        seeds = [b"stake_pool", user.key().as_ref(), stake_token_mint.key().as_ref()],
        bump,
        space = 8 + Pool::INIT_SPACE
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        init,
        payer = user,
        seeds = [b"reward_vault", pool.key().as_ref()],
        bump,
        token::mint = reward_token_mint,
        token::authority = authority
    )]
    pub reward_token_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = user,
        seeds = [b"stake_vault", pool.key().as_ref()],
        bump,
        token::mint = stake_token_mint,
        token::authority = authority
    )]
    pub stake_token_vault: Account<'info, TokenAccount>,

    /// CHECK: This PDA is used as the authority for vaults
    #[account(
        seeds = [b"pool_authority", pool.key().as_ref()],
        bump,
    )]
    pub authority: UncheckedAccount<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)] pub user: Signer<'info>,
    #[account(mut)] pub pool: Account<'info, Pool>,
    #[account(mut)] pub stake_token_mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = user,
        seeds = [b"user_stake", pool.key().as_ref(), user.key().as_ref()],
        bump,
        space = 8 + UserStake::INIT_SPACE
    )]
    pub user_stake: Account<'info, UserStake>,

    pub system_program: Program<'info, System>
}

#[account]
#[derive(InitSpace)]
pub struct UserStake {
    pub pool: Pubkey,
    pub stake_token_mint: Pubkey,
    pub stake_amount: u64,
    pub last_stake_time: u64,
    pub reward_debt: u128,
    pub bump: u8
}

#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub authority: Pubkey,
    pub stake_token_mint: Pubkey,
    pub reward_token_mint: Pubkey,
    pub stake_token_vault: Pubkey,
    pub reward_token_vault: Pubkey,
    pub total_staked: u64,
    pub reward_rate: u64,
    pub created_at_epoch: u64,
    pub updated_at_epoch: u64,
    pub created_at_ts: i64,
    pub updated_at_ts: i64,
    pub stake_cap: u64,
    pub bump: u8,
}