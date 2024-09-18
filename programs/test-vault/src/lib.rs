use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{
        Mint, Token, TokenAccount, Transfer, MintTo, Burn,
        mint_to, transfer, burn,
    },
};

declare_id!("FuZswV8nqxeRQikhwdEtdu1SkrMSW53UwggHkuMrqsxa");

#[program]
pub mod test_vault {
    use super::*;

    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        let bumps = Bumps {
            vault: ctx.bumps.vault,
            vault_authority: ctx.bumps.vault_authority,
            vault_token_account: ctx.bumps.vault_token_account,
            shares_mint: ctx.bumps.shares_mint,
        };
 
        ctx.accounts.vault.set_inner(Vault {
            deposit: 0,
            interest_earned: None,
            initialized: true,
            owner: ctx.accounts.owner.key(),
            mint: ctx.accounts.mint.key(),
            bumps,
        });

        msg!("Vault initialized successfully.");
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, deposit_amount: u64) -> Result<()> {
        if deposit_amount == 0 {
            return err!(ErrorCode::InvalidDepositAmount);
        }
        
        msg!("Depositing {} to vault", deposit_amount);

        transfer_tokens_to_vault(&ctx, deposit_amount)?;
        mint_shares_to_owner(&ctx, deposit_amount)?;
        update_vault_deposit(&mut ctx.accounts.vault, deposit_amount, true)?;

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, withdraw_amount: u64) -> Result<()> {
        let vault_token_balance = ctx.accounts.vault_token_account.amount;
        if withdraw_amount == 0 || withdraw_amount > vault_token_balance {
            return err!(ErrorCode::InvalidWithdrawAmount);
        }

        msg!("Withdrawing {} from vault", withdraw_amount);

        transfer_tokens_from_vault(&ctx, withdraw_amount)?;
        burn_shares(&ctx, withdraw_amount)?;
        update_vault_deposit(&mut ctx.accounts.vault, withdraw_amount, false)?;

        Ok(())
    }
}

// Helper functions
fn transfer_tokens_to_vault<'info>(ctx: &Context<Deposit<'info>>, amount: u64) -> Result<()> {
    transfer(
        ctx.accounts.token_program_context(Transfer {
            from: ctx.accounts.owner_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        }),
        amount,
    )
}

fn mint_shares_to_owner<'info>(ctx: &Context<Deposit<'info>>, amount: u64) -> Result<()> {
    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.shares_mint.to_account_info(),
                to: ctx.accounts.owner_shares_account.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            &[&[
                b"authority",
                ctx.accounts.vault.to_account_info().key.as_ref(),
                &[ctx.accounts.vault.bumps.vault_authority],
            ]],
        ),
        amount,
    )
}

fn transfer_tokens_from_vault<'info>(ctx: &Context<Withdraw<'info>>, amount: u64) -> Result<()> {
    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_token_account.to_account_info(),
                to: ctx.accounts.owner_token_account.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            &[&[
                b"authority",
                ctx.accounts.vault.to_account_info().key.as_ref(),
                &[ctx.accounts.vault.bumps.vault_authority],
            ]],
        ),
        amount,
    )
}

fn burn_shares<'info>(ctx: &Context<Withdraw<'info>>, amount: u64) -> Result<()> {
    burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.shares_mint.to_account_info(),
                from: ctx.accounts.owner_shares_account.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        ),
        amount,
    )
}

fn update_vault_deposit(vault: &mut Account<Vault>, amount: u64, is_deposit: bool) -> Result<()> {
    vault.deposit = if is_deposit {
        vault.deposit.checked_add(amount)
    } else {
        vault.deposit.checked_sub(amount)
    }.ok_or(ErrorCode::ArithmeticError)?;
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    // External accounts
    #[account(mut)]
    owner: Signer<'info>,
    #[account(constraint = mint.is_initialized == true)]
    mint: Box<Account<'info, Mint>>,
    #[account(mut, token::mint=mint, token::authority=owner)]
    owner_token_account: Box<Account<'info, TokenAccount>>,

    // PDAs
    #[account(
        init,
        payer = owner,
        space = 8 + std::mem::size_of::<Vault>(),
        seeds = [b"vault".as_ref(), owner.key().as_ref(), mint.key().as_ref()], bump
    )]
    vault: Box<Account<'info, Vault>>,
    #[account(
        seeds = [b"authority".as_ref(), vault.key().as_ref()], bump
    )]
    vault_authority: SystemAccount<'info>,
    #[account(
        init,
        payer = owner,
        token::mint = mint,
        token::authority = vault_authority,
        seeds = [b"tokens".as_ref(), vault.key().as_ref()], bump
    )]
    vault_token_account: Box<Account<'info, TokenAccount>>,

    // InitShares
    #[account(
        init,
        payer = owner,
        mint::decimals = 9,
        mint::authority = vault_authority,
        mint::freeze_authority = vault_authority,
        seeds = [b"shares_mint".as_ref(), vault.key().as_ref()], bump
    )]
    shares_mint: Box<Account<'info, Mint>>,

    // Programs
    token_program: Program<'info, Token>,
    associated_token_program: Program<'info, AssociatedToken>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}


impl<'info> InitializeVault<'info> {
    pub fn token_program_context<T: ToAccountMetas + ToAccountInfos<'info>>(
        &self,
        data: T,
    ) -> CpiContext<'_, '_, '_, 'info, T> {
        CpiContext::new(self.token_program.to_account_info(), data)
    }
}

#[account]
#[derive(Debug)]
pub struct Vault {
    pub deposit: u64,
    pub interest_earned: Option<u64>,
    pub initialized: bool,
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub bumps: Bumps,
}

#[derive(AnchorDeserialize, AnchorSerialize, Debug, Clone)]
pub struct Bumps {
    pub vault: u8,
    pub vault_authority: u8,
    pub vault_token_account: u8,
    pub shares_mint: u8
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    // External accounts
    #[account(mut,address = vault.owner)]
    owner: Signer<'info>,
    #[account(mut, token::mint=vault.mint, token::authority=owner)]
    owner_token_account: Account<'info, TokenAccount>,
    #[account(constraint = mint.is_initialized == true)]
    mint: Account<'info, Mint>,

    // PDAs
    #[account(
        mut,
        seeds = [b"vault".as_ref(), owner.key().as_ref(), mint.key().as_ref()],
        bump = vault.bumps.vault,
        constraint = vault.initialized == true,
    )]
    vault: Box<Account<'info, Vault>>,
    #[account(
        seeds = [b"authority".as_ref(), vault.key().as_ref()],
        bump = vault.bumps.vault_authority
    )]
    vault_authority: SystemAccount<'info>,
    #[account(
        mut,
        token::mint=vault.mint,
        token::authority=vault_authority,
        seeds = [b"tokens".as_ref(), vault.key().as_ref()],
        bump = vault.bumps.vault_token_account
    )]
    vault_token_account: Box<Account<'info, TokenAccount>>,

    // InitShares
    #[account(
        mut,
        seeds = [b"shares_mint".as_ref(), vault.key().as_ref()],
        bump = vault.bumps.shares_mint,
        constraint = mint.is_initialized == true
    )]
    shares_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = shares_mint,
        associated_token::authority = owner,
    )]
    owner_shares_account: Box<Account<'info, TokenAccount>>,

    // Programs
    token_program: Program<'info, Token>,
    associated_token_program: Program<'info, AssociatedToken>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}

impl<'info> Deposit<'info> {
    fn token_program_context<T: ToAccountMetas + ToAccountInfos<'info>>(
        &self,
        data: T,
    ) -> CpiContext<'_, '_, '_, 'info, T> {
        CpiContext::new(self.token_program.to_account_info(), data)
    }
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    // External accounts
    #[account(address = vault.owner)]
    owner: Signer<'info>,
    #[account(mut, token::mint=vault.mint, token::authority=owner)]
    owner_token_account: Account<'info, TokenAccount>,
    #[account(constraint = mint.is_initialized == true)]
    mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"vault".as_ref(), owner.key().as_ref(), mint.key().as_ref()],
        bump = vault.bumps.vault,
        constraint = vault.initialized == true,
    )]
    vault: Account<'info, Vault>,
    #[account(
        mut,
        token::mint=vault.mint,
        token::authority=vault_authority,
        seeds = [b"tokens".as_ref(), vault.key().as_ref()],
        bump = vault.bumps.vault_token_account
    )]
    vault_token_account: Account<'info, TokenAccount>,
    #[account(
        seeds = [b"authority".as_ref(), vault.key().as_ref()],
        bump = vault.bumps.vault_authority
    )]
    vault_authority: SystemAccount<'info>, 

    // InitShares
    #[account(
        mut,
        seeds = [b"shares_mint".as_ref(), vault.key().as_ref()],
        bump = vault.bumps.shares_mint,
        constraint = mint.is_initialized == true
    )]
    shares_mint: Account<'info, Mint>,

    #[account(
        mut,
        token::mint = shares_mint,
        token::authority = owner,
    )]
    owner_shares_account: Box<Account<'info, TokenAccount>>,

    // Programs section
    token_program: Program<'info, Token>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Deposit amount must be greater than 0")]
    InvalidDepositAmount,
    #[msg("Withdraw amount must be")]
    InvalidWithdrawAmount,
    #[msg("Arithmetic error")]
    ArithmeticError,
}