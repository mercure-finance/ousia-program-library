use std::ops::Deref;

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Token, Transfer, TransferChecked},
    token_interface::Burn,
};
use pyth_sdk_solana::{load_price_feed_from_account_info, state::load_price_account};
use solana_program::program_option::COption;

use {
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, TokenAccount},
};

use crate::{error::ErrorCode, AssetAccount, PositionAccount, TokenInfo};

use super::create_asset::PriceFeed;

pub fn close_position(
    ctx: Context<ClosePosition>,
    _minting_token_reverse_quotes: bool,
    _price_feed: Pubkey,
    _create_key: Pubkey,
) -> Result<()> {
    let asset_account = &mut ctx.accounts.asset_account;

    let position_account = &mut ctx.accounts.position_account;

    let remaining_accounts = ctx.remaining_accounts;

    let unix_timestamp = Clock::get()?.unix_timestamp;

    let accepted_collateral_tokens: Vec<TokenInfo> = vec![TokenInfo {
        mint: Pubkey::try_from("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU").unwrap(), // usdc devnet mint
        price_feed: Pubkey::try_from("5SSkXsEKQepHHAewytPVwdej4epN1nxgLVM84L4KXgy7").unwrap(),
        decimals: 6,
    }];

    let cpi_accounts = Burn {
        mint: ctx.accounts.mint_account.to_account_info(),
        from: ctx.accounts.associated_token_account.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
    anchor_spl::token_interface::burn(cpi_context, ctx.accounts.position_account.amount)?;
    Ok(())
}

#[derive(Accounts)]
#[instruction(minting_token_reverse_quotes: bool, price_feed: Pubkey, create_key: Pubkey)]
pub struct ClosePosition<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(seeds = [b"asset".as_ref(), price_feed.as_ref(), &[minting_token_reverse_quotes as u8]], bump)]
    pub asset_account: Account<'info, AssetAccount>,

    #[account(mut, seeds = [b"position".as_ref(), asset_account.key().as_ref(), create_key.key().as_ref()], bump)]
    pub position_account: Account<'info, PositionAccount>,

    #[account(
        mut,
        seeds = [b"asset_account", asset_account.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = mint_authority.key(),
        mint::freeze_authority = mint_authority.key(),
    )]
    pub mint_account: Account<'info, Mint>,

    #[account(
        mut,
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
