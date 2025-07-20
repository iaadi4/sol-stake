use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer};

declare_id!("E8E9zijhTNpPPuexRFYjiyqstFwHoJN7mNPNLvPpv6F1");

const PRECISION: u128 = 1_000_000_000_000;

#[program]
pub mod sol_stake {
    use anchor_spl::token;

    use super::*;

    pub fn initialize(ctx: Context<Initialize>, stake_cap: u64) -> Result<()> {
        let reward_token_mint = &mut ctx.accounts.reward_token_mint;
        let stake_token_mint = &mut ctx.accounts.stake_token_mint;
        let stake_token_vault = &mut ctx.accounts.stake_token_vault;
        let reward_token_vault = &mut ctx.accounts.reward_token_vault;

        let clock = &ctx.accounts.clock;

        let authority = &mut ctx.accounts.authority;
        let pool: &mut Account<'_, Pool> = &mut ctx.accounts.pool;

        pool.authority = authority.key();
        pool.last_reward_balance = reward_token_vault.amount;
        pool.reward_token_mint = reward_token_mint.key();
        pool.stake_token_mint = stake_token_mint.key();
        pool.stake_token_vault = stake_token_vault.key();
        pool.reward_token_vault = reward_token_vault.key();
        pool.stake_cap = stake_cap;
        pool.total_staked = 0;
        pool.acc_reward_per_share = 0;
        pool.bump = ctx.bumps.pool;

        pool.created_at_ts = clock.unix_timestamp;
        pool.created_at_epoch = clock.epoch;
        pool.updated_at_ts = clock.unix_timestamp;
        pool.updated_at_epoch = clock.epoch;

        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, stake_amount: u64) -> Result<()> {
        let user = &mut ctx.accounts.user;
        let pool = &mut ctx.accounts.pool;
        let user_stake = &mut ctx.accounts.user_stake;

        require!(
            pool.total_staked + (stake_amount as u128) <= pool.stake_cap as u128,
            CustomError::StakeCapExceeded
        );

        let pool_key = pool.key();

        let authority_seed = &[b"pool_authority", pool_key.as_ref(), &[ctx.bumps.authority]];
        let signer = &[&authority_seed[..]];

        let pending = (user_stake.stake_amount as u128)
            .checked_mul(pool.acc_reward_per_share)
            .unwrap()
            .checked_div(PRECISION)
            .unwrap()
            .checked_sub(user_stake.reward_debt)
            .unwrap();

        if pending > 0 {
            require!(
                ctx.accounts.reward_token_vault.mint == ctx.accounts.user_reward_account.mint,
                CustomError::InvalidRewardTokenMint
            );

            let reward_tranfer_cpi_accounts = Transfer {
                from: ctx.accounts.reward_token_vault.to_account_info(),
                to: ctx.accounts.user_reward_account.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            };

            let reward_transfer_cpi_tx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                reward_tranfer_cpi_accounts,
                signer,
            );

            token::transfer(reward_transfer_cpi_tx, pending as u64)?;
        }

        require!(
            ctx.accounts.stake_token_vault.mint == ctx.accounts.user_stake_account.mint,
            CustomError::InvalidStakeTokenMint
        );

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_stake_account.to_account_info(),
            to: ctx.accounts.stake_token_vault.to_account_info(),
            authority: user.to_account_info(),
        };

        let cpi_tx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);

        token::transfer(cpi_tx, stake_amount)?;

        user_stake.stake_amount = user_stake.stake_amount.checked_add(stake_amount).unwrap();
        pool.total_staked = pool.total_staked.checked_add(stake_amount as u128).unwrap();

        user_stake.reward_debt = (user_stake.stake_amount as u128)
            .checked_mul(pool.acc_reward_per_share)
            .unwrap()
            .checked_div(PRECISION)
            .unwrap();

        user_stake.pool = ctx.accounts.pool.key();
        user_stake.last_stake_time = ctx.accounts.clock.unix_timestamp;
        user_stake.bump = ctx.bumps.user_stake;

        Ok(())
    }

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let user_stake = &mut ctx.accounts.user_stake;
        let clock = Clock::get().unwrap();

        let pending = (user_stake.stake_amount as u128)
            .checked_mul(pool.acc_reward_per_share)
            .unwrap()
            .checked_div(PRECISION)
            .unwrap()
            .checked_sub(user_stake.reward_debt)
            .unwrap();

        if pending > 0 {
            let pool_key = pool.key();
            let authority_seed = &[b"pool_authority", pool_key.as_ref(), &[ctx.bumps.authority]];
            let signer = &[&authority_seed[..]];

            let reward_accounts = Transfer {
                from: ctx.accounts.reward_token_vault.to_account_info(),
                to: ctx.accounts.user_reward_account.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            };
            let cpi = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                reward_accounts,
                signer,
            );

            token::transfer(cpi, pending as u64)?;
            pool.last_reward_balance = ctx.accounts.reward_token_vault.amount;
        }

        user_stake.reward_debt = (user_stake.stake_amount as u128)
            .checked_mul(pool.acc_reward_per_share)
            .unwrap()
            .checked_div(PRECISION)
            .unwrap();
        user_stake.last_stake_time = clock.unix_timestamp;

        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let user_stake = &mut ctx.accounts.user_stake;
        let clock = Clock::get().unwrap();

        let pending = (user_stake.stake_amount as u128)
            .checked_mul(pool.acc_reward_per_share)
            .unwrap()
            .checked_div(PRECISION)
            .unwrap()
            .checked_sub(user_stake.reward_debt)
            .unwrap();

        if pending > 0 {
            let pool_key = pool.key();
            let authority_seed = &[b"pool_authority", pool_key.as_ref(), &[ctx.bumps.authority]];
            let signer = &[&authority_seed[..]];

            let reward_accounts = Transfer {
                from: ctx.accounts.reward_token_vault.to_account_info(),
                to: ctx.accounts.user_reward_account.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            };
            let cpi = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                reward_accounts,
                signer,
            );

            token::transfer(cpi, pending as u64)?;
            pool.last_reward_balance = ctx.accounts.reward_token_vault.amount;
        }

        require!(
            amount <= user_stake.stake_amount,
            CustomError::InsufficientStake
        );

        let cpi_accounts = Transfer {
            from: ctx.accounts.stake_token_vault.to_account_info(),
            to: ctx.accounts.user_stake_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let pool_key = pool.key();
        let authority_seed = &[b"pool_authority", pool_key.as_ref(), &[ctx.bumps.authority]];
        let signer = &[&authority_seed[..]];
        let cpi = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer,
        );

        token::transfer(cpi, amount)?;

        user_stake.stake_amount = user_stake.stake_amount.checked_sub(amount).unwrap();
        pool.total_staked = pool.total_staked.checked_sub(amount as u128).unwrap();

        user_stake.reward_debt = (user_stake.stake_amount as u128)
            .checked_mul(pool.acc_reward_per_share)
            .unwrap()
            .checked_div(PRECISION)
            .unwrap();
        user_stake.last_stake_time = clock.unix_timestamp;

        Ok(())
    }

    pub fn distribute(ctx: Context<Distribute>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let current_balance = ctx.accounts.reward_token_vault.amount;

        let new_rewards = current_balance
            .checked_sub(pool.last_reward_balance)
            .unwrap();

        if pool.total_staked == 0 || new_rewards == 0 {
            return Ok(());
        }

        pool.acc_reward_per_share = pool
            .acc_reward_per_share
            .checked_add(
                (new_rewards as u128)
                    .checked_mul(PRECISION)
                    .unwrap()
                    .checked_div(pool.total_staked as u128)
                    .unwrap(),
            )
            .unwrap();

        pool.last_reward_balance = current_balance;

        Ok(())
    }
}

#[error_code]
pub enum CustomError {
    #[msg("Stake cap exceeded")]
    StakeCapExceeded,
    #[msg("Invalid reward token mint")]
    InvalidRewardTokenMint,
    #[msg("Invalid stake token mint")]
    InvalidStakeTokenMint,
    #[msg("Insufficient stake to unstake")]
    InsufficientStake,
}

#[derive(Accounts)]
#[instruction(stake_cap: u64)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
#[account(mut)]
    pub reward_token_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub stake_token_mint: Box<Account<'info, Mint>>,

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
    pub reward_token_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = user,
        seeds = [b"stake_vault", pool.key().as_ref()],
        bump,
        token::mint = stake_token_mint,
        token::authority = authority
    )]
    pub stake_token_vault: Box<Account<'info, TokenAccount>>,

/// CHECK: This PDA is used as the authority for token vaults and doesn't need additional checks
    #[account(
        seeds = [b"pool_authority", pool.key().as_ref()],
        bump,
    )]
    pub authority: UncheckedAccount<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(stake_amount: u64)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, has_one = stake_token_vault, has_one = reward_token_vault)]
    pub pool: Account<'info, Pool>,
    #[account(mut, constraint = user_stake_account.mint == pool.stake_token_mint)]
    pub user_stake_account: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = user_reward_account.mint == pool.reward_token_mint)]
    pub user_reward_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub stake_token_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub reward_token_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = user,
        seeds = [b"user_stake", pool.key().as_ref(), user.key().as_ref()],
        bump,
        space = 8 + UserStake::INIT_SPACE
    )]
    pub user_stake: Account<'info, UserStake>,

    /// CHECK: This PDA is used as the authority for token vaults and doesn't need additional checks
    #[account(
        seeds = [b"pool_authority", pool.key().as_ref()],
        bump,
    )]
    pub authority: UncheckedAccount<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Distribute<'info> {
    #[account(mut, has_one = reward_token_vault)]
    pub pool: Account<'info, Pool>,
    #[account()]
    pub reward_token_vault: Box<Account<'info, TokenAccount>>,
}

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)] pub user: Signer<'info>,
    #[account(mut, has_one = stake_token_vault, has_one = reward_token_vault)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub stake_token_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub reward_token_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = user_stake_account.mint == pool.stake_token_mint)]
    pub user_stake_account: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = user_reward_account.mint == pool.reward_token_mint)]
    pub user_reward_account: Box<Account<'info, TokenAccount>>,
    #[account(mut,
        seeds = [b"user_stake", pool.key().as_ref(), user.key().as_ref()],
        bump = user_stake.bump,
        has_one = pool
    )]
    pub user_stake: Account<'info, UserStake>,

    /// CHECK: This PDA is used as the authority for token vaults and doesn't need additional checks
    #[account(
        seeds = [b"pool_authority", pool.key().as_ref()],
        bump,
    )]
    pub authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct Unstake<'info> {
    #[account(mut, has_one = stake_token_vault, has_one = reward_token_vault)]
    pub pool: Account<'info, Pool>,
    #[account(mut)] pub stake_token_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut)] pub reward_token_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = user_stake_account.mint == pool.stake_token_mint)]
    pub user_stake_account: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = user_reward_account.mint == pool.reward_token_mint)]
    pub user_reward_account: Box<Account<'info, TokenAccount>>,
    #[account(mut,
        seeds = [b"user_stake", pool.key().as_ref(), user.key().as_ref()],
        bump = user_stake.bump,
        has_one = pool
    )]
    pub user_stake: Account<'info, UserStake>,
    #[account(mut)] pub user: Signer<'info>,
    /// CHECK: This PDA is used as the authority for token vaults and doesn't need additional checks
    #[account(
        seeds = [b"pool_authority", pool.key().as_ref()],
        bump,
    )]
    pub authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}


#[account]
#[derive(InitSpace)]
pub struct UserStake {
    /// The staking pool this user stake belongs to
    pub pool: Pubkey,
    /// The mint of the token being staked
    pub stake_token_mint: Pubkey,
    pub stake_amount: u64,
    pub last_stake_time: i64,
    pub reward_debt: u128,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub authority: Pubkey,
    pub stake_token_mint: Pubkey,
    pub reward_token_mint: Pubkey,
    pub stake_token_vault: Pubkey,
    pub reward_token_vault: Pubkey,
    pub acc_reward_per_share: u128,
    pub total_staked: u128,
    pub last_reward_balance: u64,
    pub created_at_epoch: u64,
    pub updated_at_epoch: u64,
    pub created_at_ts: i64,
    pub updated_at_ts: i64,
    pub stake_cap: u64,
    pub bump: u8,
}
