use anchor_lang::prelude::*;
use anchor_spl:: {
    associated_token::AssociatedToken,
    token::{transfer, Mint, MintTo, Token, TokenAccount, Transfer},
    metadata::{
        Metadata,
        CreateMetadataAccountsV3,
        create_metadata_accounts_v3,
        mpl_token_metadata::types::DataV2
    }
};

declare_id!("FuZswV8nqxeRQikhwdEtdu1SkrMSW53UwggHkuMrqsxa");

#[program]
pub mod test_vault {
    use super::*;

    pub fn initialize_vault(ctx: Context<InitializeVault>, deposit_amount: u64) -> Result<()> {

        if deposit_amount <= 0 {
            return err!(ErrorCode::InvalidDepositAmount);
        }

        msg!("depositing {} to vault", deposit_amount);
        // msg!("vault token account {:?}", ctx.accounts.vault_token_account);
        // msg!("mint token account {:?}", ctx.accounts.mint);
        // msg!("vault {:?}", ctx.accounts.vault);
        // msg!("vault authority {:?}", ctx.accounts.vault_authority);
        // msg!("shares account {:?}", ctx.accounts.shares_account);
        // msg!("metadata account {:?}", ctx.accounts.metadata_account);
        // msg!("token program {:?}", ctx.accounts.token_program.to_account_info());
        // msg!("system program {:?}", ctx.accounts.system_program);
        // msg!("token metadata program {:?}", ctx.accounts.token_metadata_program.to_account_info());
        // msg!("rent {:?}", ctx.accounts.rent);
        msg!("owner token account {:?}", ctx.accounts.owner_token_account);

        // // Transfer token from the vault owner to the vault token account
        // let context = ctx.accounts.token_program_context( 
        //     Transfer {
        //     from: ctx.accounts.owner_token_account.to_account_info(),
        //     to: ctx.accounts.vault_token_account.to_account_info(),
        //     authority: ctx.accounts.owner.to_account_info(),
        // });

        // let context = CpiContext::new(
        //     ctx.accounts.token_program.to_account_info(), 
        //     Transfer {
        //         // from: ctx.accounts.owner_token_account.to_account_info(),
        //         from: ctx.accounts.owner.to_account_info(),
        //         to: ctx.accounts.vault_token_account.to_account_info(),
        //         authority: ctx.accounts.owner.to_account_info(),
        // });

        // transfer(context, deposit_amount)?;

        let bumps = Bumps {
            vault: ctx.bumps.vault,
            vault_authority: ctx.bumps.vault_authority,
            vault_token_account: ctx.bumps.vault_token_account,
            // metadata_account: ctx.bumps.metadata_account,
        };

        msg!("initializing shares");

        // let data = DataV2 {
        //     name: "Shares".to_string(),
        //     symbol: "SHRS".to_string(),
        //     uri: "https://example.com/metadata".to_string(),
        //     seller_fee_basis_points: 0,
        //     creators: None,
        //     collection: None,
        //     uses: None,
        // };

        // let metadata_program_context = ctx.accounts.metadata_program_context(
        //     CreateMetadataAccountsV3 {
        //         metadata: ctx.accounts.metadata_account.to_account_info(),
        //         mint: ctx.accounts.shares_account.to_account_info(),
        //         mint_authority: ctx.accounts.owner.to_account_info(),
        //         update_authority: ctx.accounts.owner.to_account_info(),
        //         payer: ctx.accounts.owner.to_account_info(),
        //         system_program: ctx.accounts.system_program.to_account_info(),
        //         rent: ctx.accounts.rent.to_account_info(),
        //     }
        // );

        // create_metadata_accounts_v3(
        //     metadata_program_context,
        //     data,
        //     false,
        //     true,
        //     None,
        // )?;

        msg!("Token created successfully.");
    

        // ctx.accounts.vault.set_inner(Vault {
        //     deposited_amount: deposit_amount,
        //     withdrawn_amount: 0,
        //     interest_earned: None,
        //     initialized: true,
        //     owner: ctx.accounts.owner.key(),
        //     mint: ctx.accounts.mint.key(),
        //     bumps,
        // });

        Ok(())

    }

    pub fn deposit(ctx: Context<Deposit>, deposit_amount: u64) -> Result<()> {

        if deposit_amount <= 0 {
            return err!(ErrorCode::InvalidDepositAmount);
        }
        
        msg!("depositing {} to vault", deposit_amount);

        // Transfer token from the vault owner to the vault token account
        let context = ctx.accounts.token_program_context( Transfer {
            from: ctx.accounts.owner_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        });

        transfer(context, deposit_amount)?;

        let vault_data = &mut ctx.accounts.vault;
        let updated_deposited_amount = vault_data
            .deposited_amount
            .checked_add(deposit_amount)
            .unwrap();

        vault_data.deposited_amount = updated_deposited_amount;

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, withdraw_amount: u64) -> Result<()> {
        let vault_token_balance = &ctx.accounts.vault_token_account.amount;
        if withdraw_amount <= 0 || withdraw_amount > *vault_token_balance {
            return err!(ErrorCode::InvalidWithdrawAmount);
        }

        msg!("withdrawing {} from vault", withdraw_amount);

        // Transfer token from the vault token account to the owner token account
        let context = ctx.accounts.token_program_context( Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.owner_token_account.to_account_info(),
            authority: ctx.accounts.vault_authority.to_account_info(),
        });

        transfer(context, withdraw_amount)?;

        let vault_data = &mut ctx.accounts.vault;
        let updated_withdraw_amount = vault_data
            .deposited_amount
            .checked_sub(withdraw_amount)
            .unwrap();

        vault_data.withdrawn_amount = updated_withdraw_amount;
        // reduce deposited amount

        Ok(())

    }
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
        token::mint=mint,
        token::authority=vault_authority,
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
        seeds = [b"shares".as_ref(), vault.key().as_ref()], bump
    )]
    shares_account: Box<Account<'info, Mint>>,
    /// CHECK: Validate address by deriving pda
    #[account(
        mut,
        seeds = [b"metadata".as_ref(), token_metadata_program.key().as_ref(), shares_account.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key()
    )]
    metadata_account: UncheckedAccount<'info>,

    // Programs
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    token_metadata_program: Program<'info, Metadata>,
    rent: Sysvar<'info, Rent>,
}


impl<'info> InitializeVault<'info> {
    pub fn token_program_context<T: ToAccountMetas + ToAccountInfos<'info>>(
        &self,
        data: T,
    ) -> CpiContext<'_, '_, '_, 'info, T> {
        CpiContext::new(self.token_program.to_account_info(), data)
    }

    // pub fn metadata_program_context<T: ToAccountMetas + ToAccountInfos<'info>>(
    //     &self,
    //     data: T,
    // ) -> CpiContext<'_, '_, '_, 'info, T> {
    //     CpiContext::new(self.token_metadata_program.to_account_info(), data)
    // }
}

#[account]
#[derive(Debug)]
pub struct Vault {
    pub deposited_amount: u64,
    pub withdrawn_amount: u64,
    pub interest_earned: Option<u64>,
    pub initialized: bool,
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub bumps: Bumps,
}

// impl Vault {
//     pub const LEN: usize = {
//         let discriminator = 8;
//         let amounts = 3 * 8;
//         let option = 1;
//         let initialized = 1;
//         let pubkeys = 2 * 32;
//         let vault_bumps = 3 * 1;
//         discriminator + amounts + option + initialized + pubkeys + vault_bumps
//     };
// }

#[derive(AnchorDeserialize, AnchorSerialize, Debug, Clone)]
pub struct Bumps {
    pub vault: u8,
    pub vault_authority: u8,
    pub vault_token_account: u8,
    // pub metadata_account: u8,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    // External accounts
    #[account(address = vault.owner)]
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
    vault: Account<'info, Vault>,
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
    vault_token_account: Account<'info, TokenAccount>,

    // Programs section
    token_program: Program<'info, Token>,
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

    // Programs section
    token_program: Program<'info, Token>,
}

impl<'info> Withdraw<'info> {
    fn token_program_context<T: ToAccountMetas + ToAccountInfos<'info>>(
        &self,
        data: T,
    ) -> CpiContext<'_, '_, '_, 'info, T> {
        CpiContext::new(self.token_program.to_account_info(), data)
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Deposit amount must be greater than 0")]
    InvalidDepositAmount,
    #[msg("Withdraw amount must be")]
    InvalidWithdrawAmount,
}