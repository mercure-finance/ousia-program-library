use std::ops::Deref;

use anchor_spl::token::Token;
use mpl_token_metadata::instruction as mpl_instruction;
use pyth_sdk_solana::state::load_price_account;
use solana_program::program::{invoke, invoke_signed};

use {
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, TokenAccount},
};

use crate::{error::ErrorCode, AssetAccount};

pub fn create_asset(
    ctx: Context<CreateAsset>,
    reverse_quotes: bool,
    interest_rate: u8,
    min_collateral_ratio: u16,
    name: String,
    symbol: String,
    uri: String,
) -> Result<()> {
    let asset_account = &mut ctx.accounts.asset_account;

    asset_account.feed = ctx.accounts.price_feed.key();
    asset_account.reversed_quote = reverse_quotes;
    asset_account.interest_rate_bp = interest_rate;
    asset_account.min_collateral_ratio = min_collateral_ratio;

    let mint_account = &mut ctx.accounts.mint_account.key();

    let pda_seeds = &[
        b"mint-authority",
        mint_account.as_ref(),
        &[*ctx.bumps.get("mint_authority").unwrap()],
    ];
    let pda_signer = &[&pda_seeds[..]];

    invoke_signed(
        &mpl_instruction::create_metadata_accounts_v3(
            ctx.accounts.token_metadata_program.key(), // Program ID (the Token Metadata Program)
            ctx.accounts.metadata_account.key(),       // Metadata account
            ctx.accounts.mint_account.key(),           // Mint account
            ctx.accounts.mint_authority.key(),         // Mint authority
            ctx.accounts.signer.key(),                 // Payer
            ctx.accounts.mint_authority.key(),         // Update authority
            name,                                      // Name
            symbol,                                    // Symbol
            uri,                                       // URI
            None,                                      // Creators
            0,                                         // Seller fee basis points
            true,                                      // Update authority is signer
            false,                                     // Is mutable
            None,                                      // Collection
            None,                                      // Uses
            None,                                      // Collection Details
        ),
        &[
            ctx.accounts.metadata_account.to_account_info(),
            ctx.accounts.mint_account.to_account_info(),
            ctx.accounts.mint_authority.to_account_info(),
            ctx.accounts.signer.to_account_info(),
            ctx.accounts.mint_authority.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ],
        pda_signer,
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(reverse_quotes: bool, _interest_rate: u8)]
pub struct CreateAsset<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(init, payer = signer, space = 200, seeds = [b"asset".as_ref(), price_feed.key().as_ref(), &[reverse_quotes as u8]], bump)]
    pub asset_account: Account<'info, AssetAccount>,
    pub price_feed: Account<'info, PriceFeed>,

    #[account(
        init,
        seeds = [b"asset_account", asset_account.key().as_ref()],
        bump,
        payer = signer,
        mint::decimals = 6,
        mint::authority = mint_authority.key(),
        mint::freeze_authority = mint_authority.key(),

    )]
    pub mint_account: Account<'info, Mint>,

    /// CHECK: We're about to create this with Metaplex
    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,

    #[account(seeds = [b"mint-authority", mint_account.key().as_ref()], bump)]
    pub mint_authority: SystemAccount<'info>,

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,

    /// CHECK: Metaplex will check this
    pub token_metadata_program: UncheckedAccount<'info>,

    pub rent: Sysvar<'info, Rent>,
}

#[derive(Clone)]
pub struct PriceFeed(pyth_sdk_solana::state::PriceFeed);

impl anchor_lang::Owner for PriceFeed {
    fn owner() -> Pubkey {
        // The mainnet Pyth program ID
        let oracle_addr = "gSbePebfvPy7tRqimPoVecS2UsBvYv46ynrzWocc92s";
        Pubkey::try_from(oracle_addr).unwrap()
    }
}

impl anchor_lang::AccountDeserialize for PriceFeed {
    fn try_deserialize_unchecked(data: &mut &[u8]) -> Result<Self> {
        let account = load_price_account(data).map_err(|_x| error!(ErrorCode::PythError))?;
        let zeros: [u8; 32] = [0; 32];
        let dummy_key = Pubkey::new_from_array(zeros);
        let feed = account.to_price_feed(&dummy_key);
        Ok(PriceFeed(feed))
    }
}

impl anchor_lang::AccountSerialize for PriceFeed {
    fn try_serialize<W: std::io::Write>(&self, _writer: &mut W) -> std::result::Result<(), Error> {
        Err(error!(ErrorCode::TryToSerializePriceAccount))
    }
}

impl Deref for PriceFeed {
    type Target = pyth_sdk_solana::state::PriceFeed;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
