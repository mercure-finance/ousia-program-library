use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use pyth_sdk_solana::load_price_feed_from_account_info;
use std::str::FromStr;

pub mod instructions;
use instructions::*;
pub mod error;
use error::ErrorCode as ProgramError;

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("EXeqAfY6BiBZbvbdGsw1EZgXapQMrJeLkGhEVCigAF6u");

#[program]
mod collateralized_debt {
    use super::*;

    pub fn create_new_asset(
        ctx: Context<CreateAsset>,
        reverse_quotes: bool,
        interest_rate: u8,
        min_collateral_ratio: u16,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        instructions::create_asset(
            ctx,
            reverse_quotes,
            interest_rate,
            min_collateral_ratio,
            name,
            symbol,
            uri,
        )
    }

    pub fn open_position(
        ctx: Context<OpenPosition>,
        mint_amount: u64,
        minting_token_reverse_quotes: bool,
    ) -> Result<()> {
        open_position::open_position(ctx, mint_amount, minting_token_reverse_quotes)
    }

    pub fn close_position(
        ctx: Context<ClosePosition>,
        minting_token_reverse_quotes: bool,
        feed_pubkey: Pubkey,
        create_key: Pubkey,
    ) -> Result<()> {
        close_position::close_position(ctx, minting_token_reverse_quotes, feed_pubkey, create_key)
    }

    pub fn liquidate(ctx: Context<LiquidatePosition>) -> Result<()> {
        let clock = &ctx.accounts.clock;

        let accepted_collateral_tokens: Vec<TokenInfo> = vec![TokenInfo {
            mint: Pubkey::try_from("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU").unwrap(), // usdc devnet mint
            price_feed: Pubkey::try_from("5SSkXsEKQepHHAewytPVwdej4epN1nxgLVM84L4KXgy7").unwrap(),
            decimals: 6,
        }];

        let mut total_value: f64 = 0.0;

        for token_info in accepted_collateral_tokens.iter() {
            // Find token account by mint, ensure the owner matches signer, and get its balance
            let (token_account_info, token_balance) = ctx
                .remaining_accounts
                .iter()
                .find_map(|acc| {
                    let mut data_slice: &mut [u8] = &mut *acc.data.borrow_mut();
                    match TokenAccount::try_deserialize(&mut &*data_slice) {
                        Ok(token_account)
                            if token_account.mint == token_info.mint
                                && token_account.owner == ctx.accounts.signer.key() =>
                        {
                            Some((acc, token_account.amount))
                        }
                        _ => None,
                    }
                })
                .ok_or(ErrorCode::InstructionMissing)?;

            // Find price feed account by price_feed pubkey
            let price_feed_info = ctx
                .remaining_accounts
                .iter()
                .find(|acc| acc.key == &token_info.price_feed)
                .ok_or(ErrorCode::InstructionMissing)?;

            let price_feed = load_price_feed_from_account_info(&price_feed_info).unwrap();

            let price = price_feed
                .get_price_no_older_than(clock.unix_timestamp, 120)
                .unwrap();

            total_value += token_balance as f64 * price.price as f64;
        }

        Ok(())
    }

    // this instruction is deprecated
    pub fn collect_interest(ctx: Context<CollectInterest>) -> Result<()> {
        let clock = &ctx.accounts.clock;

        assert!(
            clock.unix_timestamp > clock.unix_timestamp + 60 * 60 * 24,
            "The last collected interest wasn't collected more than 24h ago"
        );

        Ok(())
    }
}

#[derive(Accounts)]
pub struct LiquidatePosition<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub position_account: Account<'info, PositionAccount>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct CollectInterest<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub position_account: Account<'info, PositionAccount>,
    pub clock: Sysvar<'info, Clock>,
}

#[account]
pub struct PositionAccount {
    owner: Pubkey,
    mint: Pubkey,
    amount: u64,
    create_key: Pubkey,
}

#[account]
pub struct AssetAccount {
    feed: Pubkey,
    reversed_quote: bool,
    min_collateral_ratio: u16,
    interest_rate_bp: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Debug)]
pub struct AllAssets {
    pub tokens: Vec<TokenInfo>,
}

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Debug)]
pub struct TokenInfo {
    pub mint: Pubkey,       // Token mint
    pub price_feed: Pubkey, // Oracle or price feed address
    pub decimals: u8,
}
