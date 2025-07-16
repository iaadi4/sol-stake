use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, Token};

declare_id!("E8E9zijhTNpPPuexRFYjiyqstFwHoJN7mNPNLvPpv6F1");

#[program]
pub mod sol_stake {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        
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

    pub authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>
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
    pub created_at: u64,
    pub updated_at: u64,
    pub stake_cap: u64,
    pub bump: u8,
}