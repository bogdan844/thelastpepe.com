use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("11111111111111111111111111111111");

/**
 * The Last Pepe Presale Program for Solana
 * Allows users to buy PEPE tokens using SOL or USDC
 */

#[program]
pub mod pepe_presale {
    use super::*;

    // ============ INITIALIZE PRESALE ============
    pub fn initialize_presale(
        ctx: Context<InitializePresale>,
        presale_price: u64,
        bonus_percentage: u8,
        presale_cap: u64,
    ) -> Result<()> {
        let presale = &mut ctx.accounts.presale_config;
        presale.authority = ctx.accounts.authority.key();
        presale.pepe_mint = ctx.accounts.pepe_mint.key();
        presale.usdc_mint = ctx.accounts.usdc_mint.key();
        presale.treasury_wallet = ctx.accounts.treasury_wallet.key();
        presale.presale_price = presale_price;
        presale.bonus_percentage = bonus_percentage;
        presale.presale_cap = presale_cap;
        presale.total_tokens_sold = 0;
        presale.is_paused = false;
        presale.require_whitelist = false;

        emit PresaleInitialized {
            presale_price,
            bonus_percentage,
            presale_cap,
        };

        Ok(())
    }

    // ============ BUY WITH SOL ============
    pub fn buy_with_sol(
        ctx: Context<BuyWithSol>,
        amount: u64,
    ) -> Result<()> {
        let presale = &mut ctx.accounts.presale_config;

        // Check presale is active
        require!(!presale.is_paused, PresaleError::PresalePaused);
        require!(
            presale.total_tokens_sold < presale.presale_cap,
            PresaleError::PresaleCapReached
        );

        // Check whitelist if required
        if presale.require_whitelist {
            require!(
                ctx.accounts.whitelist_config.is_some(),
                PresaleError::NotWhitelisted
            );
        }

        // Calculate PEPE amount (PEPE uses 6 decimals on Solana)
        let pepe_amount = (amount as u128)
            .checked_mul(1_000_000u128)
            .ok_or(PresaleError::CalculationError)?
            .checked_div(presale.presale_price as u128)
            .ok_or(PresaleError::CalculationError)? as u64;

        // Calculate bonus tokens
        let bonus_tokens = pepe_amount
            .checked_mul(presale.bonus_percentage as u64)
            .ok_or(PresaleError::CalculationError)?
            .checked_div(100)
            .ok_or(PresaleError::CalculationError)?;

        let total_tokens = pepe_amount
            .checked_add(bonus_tokens)
            .ok_or(PresaleError::CalculationError)?;

        // Check presale cap
        require!(
            presale.total_tokens_sold.checked_add(pepe_amount).unwrap() <= presale.presale_cap,
            PresaleError::ExceedsPresaleCap
        );

        // Transfer SOL to treasury
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.buyer.key(),
            &ctx.accounts.treasury_wallet.key(),
            amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.buyer.to_account_info(),
                ctx.accounts.treasury_wallet.to_account_info(),
            ],
        )?;

        // Transfer PEPE tokens to buyer
        let cpi_accounts = Transfer {
            from: ctx.accounts.pepe_vault.to_account_info(),
            to: ctx.accounts.buyer_pepe_account.to_account_info(),
            authority: ctx.accounts.presale_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, total_tokens)?;

        // Update presale state
        presale.total_tokens_sold = presale
            .total_tokens_sold
            .checked_add(pepe_amount)
            .ok_or(PresaleError::CalculationError)?;

        // Update buyer info
        let buyer_info = &mut ctx.accounts.buyer_info;
        buyer_info.buyer = ctx.accounts.buyer.key();
        buyer_info.tokens_purchased = buyer_info
            .tokens_purchased
            .checked_add(total_tokens)
            .ok_or(PresaleError::CalculationError)?;
        buyer_info.amount_spent = buyer_info
            .amount_spent
            .checked_add(amount)
            .ok_or(PresaleError::CalculationError)?;

        emit TokensPurchased {
            buyer: ctx.accounts.buyer.key(),
            payment_token: "SOL".to_string(),
            payment_amount: amount,
            tokens_received: pepe_amount,
            bonus_tokens,
        };

        Ok(())
    }

    // ============ BUY WITH USDC ============
    pub fn buy_with_usdc(
        ctx: Context<BuyWithUsdc>,
        amount: u64,
    ) -> Result<()> {
        let presale = &mut ctx.accounts.presale_config;

        // Check presale is active
        require!(!presale.is_paused, PresaleError::PresalePaused);
        require!(
            presale.total_tokens_sold < presale.presale_cap,
            PresaleError::PresaleCapReached
        );

        // Check whitelist if required
        if presale.require_whitelist {
            require!(
                ctx.accounts.whitelist_config.is_some(),
                PresaleError::NotWhitelisted
            );
        }

        // Calculate PEPE amount
        let pepe_amount = (amount as u128)
            .checked_mul(1_000_000u128)
            .ok_or(PresaleError::CalculationError)?
            .checked_div(presale.presale_price as u128)
            .ok_or(PresaleError::CalculationError)? as u64;

        // Calculate bonus tokens
        let bonus_tokens = pepe_amount
            .checked_mul(presale.bonus_percentage as u64)
            .ok_or(PresaleError::CalculationError)?
            .checked_div(100)
            .ok_or(PresaleError::CalculationError)?;

        let total_tokens = pepe_amount
            .checked_add(bonus_tokens)
            .ok_or(PresaleError::CalculationError)?;

        // Check presale cap
        require!(
            presale.total_tokens_sold.checked_add(pepe_amount).unwrap() <= presale.presale_cap,
            PresaleError::ExceedsPresaleCap
        );

        // Transfer USDC from buyer to treasury
        let cpi_accounts = Transfer {
            from: ctx.accounts.buyer_usdc_account.to_account_info(),
            to: ctx.accounts.treasury_usdc_account.to_account_info(),
            authority: ctx.accounts.buyer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        // Transfer PEPE tokens to buyer
        let cpi_accounts = Transfer {
            from: ctx.accounts.pepe_vault.to_account_info(),
            to: ctx.accounts.buyer_pepe_account.to_account_info(),
            authority: ctx.accounts.presale_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, total_tokens)?;

        // Update presale state
        presale.total_tokens_sold = presale
            .total_tokens_sold
            .checked_add(pepe_amount)
            .ok_or(PresaleError::CalculationError)?;

        // Update buyer info
        let buyer_info = &mut ctx.accounts.buyer_info;
        buyer_info.buyer = ctx.accounts.buyer.key();
        buyer_info.tokens_purchased = buyer_info
            .tokens_purchased
            .checked_add(total_tokens)
            .ok_or(PresaleError::CalculationError)?;
        buyer_info.amount_spent = buyer_info
            .amount_spent
            .checked_add(amount)
            .ok_or(PresaleError::CalculationError)?;

        emit TokensPurchased {
            buyer: ctx.accounts.buyer.key(),
            payment_token: "USDC".to_string(),
            payment_amount: amount,
            tokens_received: pepe_amount,
            bonus_tokens,
        };

        Ok(())
    }

    // ============ ADMIN FUNCTIONS ============

    pub fn pause_presale(ctx: Context<AdminFunction>) -> Result<()> {
        ctx.accounts.presale_config.is_paused = true;
        Ok(())
    }

    pub fn unpause_presale(ctx: Context<AdminFunction>) -> Result<()> {
        ctx.accounts.presale_config.is_paused = false;
        Ok(())
    }

    pub fn set_bonus_percentage(
        ctx: Context<AdminFunction>,
        new_percentage: u8,
    ) -> Result<()> {
        require!(new_percentage <= 100, PresaleError::InvalidPercentage);
        ctx.accounts.presale_config.bonus_percentage = new_percentage;
        emit BonusPercentageUpdated { new_percentage };
        Ok(())
    }

    pub fn set_presale_price(ctx: Context<AdminFunction>, new_price: u64) -> Result<()> {
        require!(new_price > 0, PresaleError::InvalidPrice);
        ctx.accounts.presale_config.presale_price = new_price;
        Ok(())
    }

    pub fn set_presale_cap(ctx: Context<AdminFunction>, new_cap: u64) -> Result<()> {
        ctx.accounts.presale_config.presale_cap = new_cap;
        Ok(())
    }

    pub fn set_require_whitelist(
        ctx: Context<AdminFunction>,
        require_whitelist: bool,
    ) -> Result<()> {
        ctx.accounts.presale_config.require_whitelist = require_whitelist;
        Ok(())
    }

    pub fn add_to_whitelist(ctx: Context<AddToWhitelist>) -> Result<()> {
        let whitelist = &mut ctx.accounts.whitelist_config;
        whitelist.user = ctx.accounts.user.key();
        whitelist.is_whitelisted = true;
        emit WhitelistUpdated {
            user: ctx.accounts.user.key(),
            status: true,
        };
        Ok(())
    }

    pub fn remove_from_whitelist(ctx: Context<RemoveFromWhitelist>) -> Result<()> {
        let whitelist = &mut ctx.accounts.whitelist_config;
        whitelist.is_whitelisted = false;
        emit WhitelistUpdated {
            user: ctx.accounts.user.key(),
            status: false,
        };
        Ok(())
    }

    pub fn withdraw_remaining_tokens(ctx: Context<WithdrawTokens>) -> Result<()> {
        let pepe_vault_balance = ctx.accounts.pepe_vault.amount;
        require!(pepe_vault_balance > 0, PresaleError::NoTokensToWithdraw);

        let cpi_accounts = Transfer {
            from: ctx.accounts.pepe_vault.to_account_info(),
            to: ctx.accounts.admin_pepe_account.to_account_info(),
            authority: ctx.accounts.presale_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, pepe_vault_balance)?;

        Ok(())
    }
}

// ============ ACCOUNT STRUCTURES ============

#[account]
pub struct PresaleConfig {
    pub authority: Pubkey,
    pub pepe_mint: Pubkey,
    pub usdc_mint: Pubkey,
    pub treasury_wallet: Pubkey,
    pub presale_price: u64,
    pub bonus_percentage: u8,
    pub presale_cap: u64,
    pub total_tokens_sold: u64,
    pub is_paused: bool,
    pub require_whitelist: bool,
}

#[account]
pub struct BuyerInfo {
    pub buyer: Pubkey,
    pub tokens_purchased: u64,
    pub amount_spent: u64,
}

#[account]
pub struct WhitelistConfig {
    pub user: Pubkey,
    pub is_whitelisted: bool,
}

// ============ CONTEXT STRUCTURES ============

#[derive(Accounts)]
pub struct InitializePresale<'info> {
    #[account(init, payer = authority, space = 8 + 32 + 32 + 32 + 32 + 8 + 1 + 8 + 8 + 1 + 1)]
    pub presale_config: Account<'info, PresaleConfig>,
    pub pepe_mint: Account<'info, anchor_spl::token::Mint>,
    pub usdc_mint: Account<'info, anchor_spl::token::Mint>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: Treasury wallet
    pub treasury_wallet: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BuyWithSol<'info> {
    #[account(mut)]
    pub presale_config: Account<'info, PresaleConfig>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub buyer_pepe_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pepe_vault: Account<'info, TokenAccount>,
    /// CHECK: Presale authority
    pub presale_authority: UncheckedAccount<'info>,
    /// CHECK: Treasury wallet
    #[account(mut)]
    pub treasury_wallet: UncheckedAccount<'info>,
    #[account(init_if_needed, payer = buyer, space = 8 + 32 + 8 + 8)]
    pub buyer_info: Account<'info, BuyerInfo>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub whitelist_config: Option<Account<'info, WhitelistConfig>>,
}

#[derive(Accounts)]
pub struct BuyWithUsdc<'info> {
    #[account(mut)]
    pub presale_config: Account<'info, PresaleConfig>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub buyer_pepe_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub buyer_usdc_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pepe_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub treasury_usdc_account: Account<'info, TokenAccount>,
    /// CHECK: Presale authority
    pub presale_authority: UncheckedAccount<'info>,
    #[account(init_if_needed, payer = buyer, space = 8 + 32 + 8 + 8)]
    pub buyer_info: Account<'info, BuyerInfo>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub whitelist_config: Option<Account<'info, WhitelistConfig>>,
}

#[derive(Accounts)]
pub struct AdminFunction<'info> {
    #[account(mut, has_one = authority)]
    pub presale_config: Account<'info, PresaleConfig>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct AddToWhitelist<'info> {
    pub presale_config: Account<'info, PresaleConfig>,
    #[account(init, payer = authority, space = 8 + 32 + 1)]
    pub whitelist_config: Account<'info, WhitelistConfig>,
    /// CHECK: User to whitelist
    pub user: UncheckedAccount<'info>,
    #[account(mut, address = presale_config.authority)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RemoveFromWhitelist<'info> {
    pub presale_config: Account<'info, PresaleConfig>,
    #[account(mut, has_one = user)]
    pub whitelist_config: Account<'info, WhitelistConfig>,
    /// CHECK: User to remove
    pub user: UncheckedAccount<'info>,
    #[account(address = presale_config.authority)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct WithdrawTokens<'info> {
    #[account(mut, has_one = authority)]
    pub presale_config: Account<'info, PresaleConfig>,
    #[account(mut)]
    pub pepe_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub admin_pepe_account: Account<'info, TokenAccount>,
    /// CHECK: Presale authority
    pub presale_authority: UncheckedAccount<'info>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

// ============ ERROR HANDLING ============

#[error_code]
pub enum PresaleError {
    #[msg("Presale is paused")]
    PresalePaused,
    #[msg("Presale cap reached")]
    PresaleCapReached,
    #[msg("Not whitelisted")]
    NotWhitelisted,
    #[msg("Calculation error")]
    CalculationError,
    #[msg("Exceeds presale cap")]
    ExceedsPresaleCap,
    #[msg("Invalid percentage")]
    InvalidPercentage,
    #[msg("Invalid price")]
    InvalidPrice,
    #[msg("No tokens to withdraw")]
    NoTokensToWithdraw,
}

// ============ EVENTS ============

#[event]
pub struct PresaleInitialized {
    pub presale_price: u64,
    pub bonus_percentage: u8,
    pub presale_cap: u64,
}

#[event]
pub struct TokensPurchased {
    pub buyer: Pubkey,
    pub payment_token: String,
    pub payment_amount: u64,
    pub tokens_received: u64,
    pub bonus_tokens: u64,
}

#[event]
pub struct BonusPercentageUpdated {
    pub new_percentage: u8,
}

#[event]
pub struct WhitelistUpdated {
    pub user: Pubkey,
    pub status: bool,
}
