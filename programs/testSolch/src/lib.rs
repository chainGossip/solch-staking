use anchor_lang::prelude::*;
use anchor_spl::token::{ self, TokenAccount, Token };
use anchor_lang::solana_program::{clock};
use crate::constants::*;


declare_id!("6L5dTUpZE8ZSjm8S8h65xtsHJBeJ96LmUwh7QCWV2qX2");

mod constants {
    pub const DAY_TIME: u32 = 60;
    pub const LIVE_TIME: u32 = 7 * DAY_TIME;
    pub const DECIMAL: u64 = 1000000000;
    #[warn(dead_code)]
    pub const DEPOSITE_FEE: u64 = 10 * DECIMAL;
    pub const APY: u32 = 5;
}

#[program]
pub mod solch {
    use super::*;
    pub fn create_vault(_ctx: Context<VaultAccount>, _bump_vault: u8) -> Result<()> {
        Ok(())
    }
    pub fn create_pool(_ctx: Context<PoolAccount>, _bump_pool: u8) -> Result<()> {
        let pool = &mut _ctx.accounts.pool;
        pool.owner = _ctx.accounts.user.key();
        Ok(())
    }
    pub fn stake(_ctx: Context<StakeAccount>, amount: u32) -> Result<()> {
        let pool = &mut _ctx.accounts.pool;
        if pool.owner != _ctx.accounts.user.key() {
            return Err(ErrorCode::AuthorityInvalid.into());
        }
        let clock = clock::Clock::get().unwrap();
        pool.last_time = clock.unix_timestamp as u32;
        pool.start_time = clock.unix_timestamp as u32;
        pool.amount = amount * DECIMAL as u32 - DEPOSITE_FEE as u32;
        pool.reward = (amount * DECIMAL as u32 - DEPOSITE_FEE as u32) * APY / 100 / LIVE_TIME;
        pool.is_stake = true;
        let cpi_ctx = CpiContext::new(
            _ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: _ctx.accounts.from.to_account_info(),
                to: _ctx.accounts.to.to_account_info(),
                authority: _ctx.accounts.user.to_account_info(),
            }
        );
        token::transfer(cpi_ctx, pool.amount.into());
        Ok(())
    }
    pub fn claim(_ctx: Context<ClaimAccount>, bump_vault: u8) -> Result<()> {
        let pool = &mut _ctx.accounts.pool;
        if pool.owner != _ctx.accounts.user.key() {
            return Err(ErrorCode::AuthorityInvalid.into());
        }
        let clock = clock::Clock::get().unwrap();
        let vault_seeds = &[
            b"SOLCH_STAKING_ACCOUNT".as_ref(),
            &[bump_vault]
        ];
        let vault_signer = &[&vault_seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            _ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: _ctx.accounts.from.to_account_info(),
                to: _ctx.accounts.to.to_account_info(),
                authority: _ctx.accounts.vault.to_account_info()
            },
            vault_signer
        );
        let time_stamp: u32 = clock.unix_timestamp as u32 - pool.last_time;
        let claim_amount: u32;
        if time_stamp > LIVE_TIME {
            claim_amount = pool.amount * APY / 100;
        } else {
            claim_amount = time_stamp / DAY_TIME * pool.reward;
        }
        
        pool.last_time = clock.unix_timestamp as u32;
        token::transfer(cpi_ctx, claim_amount.into());
        Ok(())
    }
    pub fn unstake(_ctx: Context<UnstakeAccount>, bump: u8) -> Result<()> {
        let pool = &mut _ctx.accounts.pool;
        if pool.owner != _ctx.accounts.user.key() {
            return Err(ErrorCode::AuthorityInvalid.into());
        }
        let clock = clock::Clock::get().unwrap();
        let time_stamp: u32 = clock.unix_timestamp as u32 - pool.start_time;
        let claim_amount: u32;
        if time_stamp > LIVE_TIME {
            claim_amount = pool.amount * APY / 100;
        } else {
            return Err(ErrorCode::UnStakeTimingInvalid.into());
        }
       
        let vault_seeds = &[
            b"SOLCH_STAKING_ACCOUNT".as_ref(),
            &[bump]
        ];
        let vault_signer = &[&vault_seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            _ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: _ctx.accounts.from.to_account_info(),
                to: _ctx.accounts.to.to_account_info(),
                authority: _ctx.accounts.vault.to_account_info()
            },
            vault_signer
        );
        pool.is_stake = false;
        token::transfer(cpi_ctx, claim_amount.into());
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct VaultAccount<'info> {
    #[account(init, seeds=[b"SOLCH_STAKING_ACCOUNT".as_ref()], bump, payer = admin, space = 8 + 1)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct PoolAccount<'info> {
    #[account(init, seeds=[b"SOLCH_STAKING_POOL".as_ref(), user.key().as_ref()], bump, payer = user, space = 8 + 32 + 32 + 4 + 1)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>
}
#[derive(Accounts)]
pub struct StakeAccount<'info> {
    pub user: Signer<'info>,
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
#[derive(Accounts)]
pub struct ClaimAccount<'info> {
    pub vault: Account<'info, Vault>,
    pub pool: Account<'info, Pool>,
    pub user: Signer<'info>,
    pub from: Account<'info, TokenAccount>,
    pub to: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
#[derive(Accounts)]
pub struct UnstakeAccount<'info> {
    pub vault: Account<'info, Vault>,
    pub pool: Account<'info, Pool>,
    pub user: Signer<'info>,
    pub from: Account<'info, TokenAccount>,
    pub to: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>
}
#[account]
pub struct Vault {
    pub bump_vault: u8
}
#[account]
pub struct Pool {
    pub owner: Pubkey,
    pub amount: u32,
    pub last_time: u32,
    pub start_time: u32,
    pub reward: u32,
    pub is_stake: bool
}

#[error_code]
pub enum ErrorCode {
    #[msg("Authority is invalid")]
    AuthorityInvalid,
    #[msg("Unstake is not available")]
    UnStakeTimingInvalid
}