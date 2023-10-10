use std::ops::Deref;

use anchor_spl::{associated_token::AssociatedToken, token::Token, token_interface::MintTo};
use pyth_sdk_solana::{load_price_feed_from_account_info, state::load_price_account};
use solana_program::program_option::COption;

use {
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, TokenAccount},
};

use crate::{error::ErrorCode, AssetAccount, PositionAccount, TokenInfo};

use super::create_asset::PriceFeed;

pub fn open_position(
    ctx: Context<OpenPosition>,
    mint_amount: u64,
    _minting_token_reverse_quotes: bool,
) -> Result<()> {
    let asset_account = &mut ctx.accounts.asset_account;

    let position_account = &mut ctx.accounts.position_account;

    let remaining_accounts = ctx.remaining_accounts;

    let unix_timestamp = Clock::get()?.unix_timestamp;

    let price_feed = ctx
        .accounts
        .price_feed
        .get_price_no_older_than(unix_timestamp, 100000)
        .ok_or(ErrorCode::PythPriceTooOld)?;

    let price = price_feed.price;

    position_account.owner = ctx.accounts.signer.key();

    // normalized means converted to floating point
    let normalized_price = (price as f64) / (10f64.powi(price_feed.expo));
    let normalized_mint_amount =
        (mint_amount as f64) / (10f64.powi(ctx.accounts.mint_account.decimals as i32));

    let minimum_collateral_dollar_value = normalized_mint_amount
        * normalized_price
        * (ctx.accounts.asset_account.min_collateral_ratio as f64 / 100.0);

    let mut total_value: f64 = 0.0;

    let accepted_collateral_tokens: Vec<TokenInfo> = vec![TokenInfo {
        mint: Pubkey::try_from("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU").unwrap(), // usdc devnet mint
        price_feed: Pubkey::try_from("5SSkXsEKQepHHAewytPVwdej4epN1nxgLVM84L4KXgy7").unwrap(),
        decimals: 6,
    }];

    for token_info in accepted_collateral_tokens.iter() {
        // Derive the associated token account address
        let (associated_token_address, _) = Pubkey::find_program_address(
            &[
                &ctx.accounts.position_account.key().as_ref(),
                &anchor_spl::token::ID.as_ref(),
                &token_info.mint.as_ref(),
            ],
            &anchor_spl::associated_token::ID,
        );

        // Check if the derived address corresponds to an initialized token account in `ctx.remaining_accounts`
        let (_token_account_info, token_balance) = ctx
            .remaining_accounts
            .iter()
            .find_map(|acc| {
                if acc.key != &associated_token_address {
                    return None;
                }

                let data_slice: &mut [u8] = &mut *acc.data.borrow_mut();
                match TokenAccount::try_deserialize(&mut &*data_slice) {
                    Ok(token_account) => {
                        if token_account.mint == token_info.mint
                            && token_account.owner == ctx.accounts.position_account.key()
                            && token_account.delegate.is_none()
                        {
                            Some((acc, token_account.amount))
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            })
            .ok_or(ErrorCode::MissingTokenAccounts)?; // If not found, fail the transaction

        // Find price feed account by price_feed pubkey
        let price_feed_info = ctx
            .remaining_accounts
            .iter()
            .find(|acc| acc.key == &token_info.price_feed)
            .ok_or(ErrorCode::MissingPriceFeedAccounts)?;

        let price_feed = load_price_feed_from_account_info(price_feed_info).unwrap();

        let price_info = price_feed
            .get_price_no_older_than(unix_timestamp, 120)
            .unwrap();
        let i64_price = price_info.price;

        let divisor = 10f64.powi(-price_info.expo as i32);
        let price = i64_price as f64 / divisor;

        let f64_token_balance = token_balance as f64 / (10f64.powi(6));

        total_value += f64_token_balance * price as f64;
    }

    let minimum_collateral_value = ctx.accounts.asset_account.min_collateral_ratio as f64 / 100.0;

    let f64_mint_amount =
        mint_amount as f64 / (10f64.powi(ctx.accounts.mint_account.decimals as i32));

    let debt_asset_price_info = ctx
        .accounts
        .price_feed
        .get_price_no_older_than(unix_timestamp, 120000)
        .ok_or(ErrorCode::PythPriceTooOld)?;

    let mut debt_asset_price =
        debt_asset_price_info.price as f64 / (10f64.powi(-debt_asset_price_info.expo as i32));

    if ctx.accounts.asset_account.reversed_quote {
        debt_asset_price = 1.0 / debt_asset_price;
    }

    let debt_asset_total_value = f64_mint_amount * debt_asset_price;

    let minimum_collateral_amount = debt_asset_total_value * minimum_collateral_value;

    assert!(
        total_value >= minimum_collateral_amount,
        "total_value: {:?}, minimum_collateral_amount: {:?}",
        total_value,
        minimum_collateral_amount
    );
    let mint_account = &mut ctx.accounts.mint_account.key();
    msg!("mint_account: {:?}", mint_account);
    let pda_seeds = &[
        b"mint-authority",
        mint_account.as_ref(),
        &[*ctx.bumps.get("mint_authority").unwrap()],
    ];
    let pda_signer = &[&pda_seeds[..]];

    let cpi_accounts = MintTo {
        mint: ctx.accounts.mint_account.to_account_info(),
        to: ctx.accounts.associated_token_account.to_account_info(),
        authority: ctx.accounts.mint_authority.to_account_info(),
    };

    // TODO
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, pda_signer);
    anchor_spl::token_interface::mint_to(cpi_context, mint_amount)?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(minting_token_reverse_quotes: bool, _interest_rate: u8)]
pub struct OpenPosition<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(seeds = [b"asset".as_ref(), price_feed.key().as_ref(), &[minting_token_reverse_quotes as u8]], bump)]
    pub asset_account: Account<'info, AssetAccount>,

    #[account(init, payer = signer, space = 200, seeds = [b"position".as_ref(), asset_account.key().as_ref(), create_key.key().as_ref()], bump)]
    pub position_account: Account<'info, PositionAccount>,

    pub price_feed: Account<'info, PriceFeed>,

    pub create_key: Signer<'info>,

    #[account(
        mut,
        seeds = [b"asset_account", asset_account.key().as_ref()],
        bump,
        mint::decimals = 9,
        mint::authority = mint_authority.key(),
        mint::freeze_authority = mint_authority.key(),
    )]
    pub mint_account: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint_account,
        associated_token::authority = signer,
        associated_token::token_program = token_program
    )]
    pub associated_token_account: Account<'info, TokenAccount>,

    #[account(seeds = [b"mint-authority".as_ref(), mint_account.key().as_ref()], bump)]
    pub mint_authority: SystemAccount<'info>,

    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub system_program: Program<'info, System>,
}
