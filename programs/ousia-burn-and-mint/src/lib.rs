use std::str::FromStr;
use {
    anchor_lang::prelude::Pubkey,
    anchor_lang::prelude::*,
    anchor_spl::{associated_token, token},
};

declare_id!("Fa63c9tRsE3uJ3GQ7q22KAWq38pq4tfyR2AkqpCprgay");

#[program]
mod ousia_burn_and_mint {
    use super::*;
    pub fn place_order(ctx: Context<PlaceOrder>, amount: u64, price: u64, _id: Pubkey, order_type: OrderType) -> Result<()> {

    let adjusted_price = price as f64 / 10f64.powi(ctx.accounts.purchase_token_mint_account.decimals as i32);
    let total = adjusted_price * (amount as f64);
    let adjusted_total = total * 10f64.powi(ctx.accounts.usdc_mint_account.decimals as i32);

    if order_type == OrderType::Buy {
        token::transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::TransferChecked {
                    from: ctx.accounts.buyer_usdc_ata.to_account_info(),
                    to: ctx.accounts.oder_account_usdc_ata.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                    mint: ctx.accounts.usdc_mint_account.to_account_info(),
                },
            ),
            adjusted_total as u64,
            6,
        )?;
    } else if order_type == OrderType::Sell {
        token::transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::TransferChecked {
                    from: ctx.accounts.buyer_purchase_token_account.to_account_info(),
                    to: ctx.accounts.order_account_purchase_token_account.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                    mint: ctx.accounts.purchase_token_mint_account.to_account_info(),
                },
            ),
            amount,
            6,
        )?;
    }

        ctx.accounts.order_account.amount = amount as u32;
        ctx.accounts.order_account.price = adjusted_price;
        ctx.accounts.order_account.mint = ctx.accounts.purchase_token_mint_account.key();
        ctx.accounts.order_account.owner = ctx.accounts.signer.key();

        emit!(OrderPlaced {
            amount: amount as u32,
            price: adjusted_price,
            mint: ctx.accounts.purchase_token_mint_account.key(),
            order_account: ctx.accounts.order_account.key(),
            signer: ctx.accounts.signer.key(),
            order_type: order_type,
        });

        Ok(())

    }

    pub fn fill_order(ctx: Context<FillOrder>, id: Pubkey) -> Result<()> {
         if ctx.accounts.order_account.order_type == OrderType::Buy {
        token::transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::TransferChecked {
                    from: ctx.accounts.buyer_usdc_ata.to_account_info(),
                    to: ctx.accounts.oder_account_usdc_ata.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                    mint: ctx.accounts.usdc_mint_account.to_account_info(),
                },
            ),
            ctx.accounts.order_account.amount as u64,
            6,
        )?;
    } else if ctx.accounts.order_account.order_type == OrderType::Sell {
        token::transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::TransferChecked {
                    from: ctx.accounts.buyer_purchase_token_account.to_account_info(),
                    to: ctx.accounts.order_account_purchase_token_account.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                    mint: ctx.accounts.purchase_token_mint_account.to_account_info(),
                },
            ),
        ctx.accounts.order_account.amount as u64 * ctx.accounts.order_account.price as u64,
            6,
        )?;
    }
        
        Ok(())
    }
}


#[derive(Accounts)]
#[instruction(_amount: u64, _price: f64, id: Pubkey, order_type: OrderType)]
pub struct PlaceOrder<'info> {

    // -- usdc token accounts -- 

    // usdc mint
    #[account(
        mut,
        mint::decimals = 6,
        mint::authority = mint_authority.key(),
        // address = Pubkey::from_str("ddedededde").unwrap()
    )]
    pub usdc_mint_account: Account<'info, token::Mint>,

    // buyers usdc token account
    #[account(
        mut,
        associated_token::mint = usdc_mint_account,
        associated_token::authority = signer,
    )]
    pub buyer_usdc_ata: Account<'info, token::TokenAccount>,

    // order account usdc token account
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = usdc_mint_account,
        associated_token::authority = order_account,
    )]
    pub oder_account_usdc_ata: Account<'info, token::TokenAccount>,

   
    // -- purchase token accounts --

    // purchase token mint
    #[account(
        mut,
        mint::decimals = 6,
        mint::authority = mint_authority.key(),
    )]
    pub purchase_token_mint_account: Box<Account<'info, token::Mint>>,

    // buyers purchase token account
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = purchase_token_mint_account,
        associated_token::authority = signer,
    )]
    pub buyer_purchase_token_account: Box<Account<'info, token::TokenAccount>>,

    // order account purchase token account
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = purchase_token_mint_account,
        associated_token::authority = order_account,
    )]
    pub order_account_purchase_token_account: Box<Account<'info, token::TokenAccount>>,


    // mint authority
    #[account(
        address = Pubkey::from_str("44LZ5pUPJTc3TyrEu6qUgmwxS4HGkmxuTjpxj7iNeaRt").unwrap()
    )]
    pub mint_authority: SystemAccount<'info>,

    // order account
    #[account(
        init, 
        payer = signer, 
        seeds=[b"order", signer.key().as_ref(), id.as_ref()], 
        bump, 
        space = 260,
    )]
    pub order_account: Account<'info, Order>,


    // signer
    #[account(mut)]
    pub signer: Signer<'info>,

    // programs
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, token::Token>,
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
}

#[derive(Accounts)]
#[instruction(id: Pubkey)]
pub struct FillOrder<'info> {

    // -- usdc token accounts -- 

    // usdc mint
    #[account(
        mut,
        mint::decimals = 6,
        mint::authority = mint_authority.key(),
        // address = Pubkey::from_str("ddedededde").unwrap()
    )]
    pub usdc_mint_account: Account<'info, token::Mint>,

    // buyers usdc token account
    #[account(
        mut,
        associated_token::mint = usdc_mint_account,
        associated_token::authority = signer,
    )]
    pub buyer_usdc_ata: Account<'info, token::TokenAccount>,

    // order account usdc token account
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = usdc_mint_account,
        associated_token::authority = order_account,
    )]
    pub oder_account_usdc_ata: Account<'info, token::TokenAccount>,

   
    // -- purchase token accounts --

    // purchase token mint
    #[account(
        mut,
        mint::decimals = 6,
        mint::authority = mint_authority.key(),
    )]
    pub purchase_token_mint_account: Box<Account<'info, token::Mint>>,

    // buyers purchase token account
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = purchase_token_mint_account,
        associated_token::authority = signer,
    )]
    pub buyer_purchase_token_account: Box<Account<'info, token::TokenAccount>>,

    // order account purchase token account
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = purchase_token_mint_account,
        associated_token::authority = order_account,
    )]
    pub order_account_purchase_token_account: Box<Account<'info, token::TokenAccount>>,


    // mint authority
    #[account(
        address = Pubkey::from_str("44LZ5pUPJTc3TyrEu6qUgmwxS4HGkmxuTjpxj7iNeaRt").unwrap()
    )]
    pub mint_authority: SystemAccount<'info>,

    // order account
    #[account(
        mut,
        seeds=[b"order", signer.key().as_ref(), id.as_ref()], 
        bump,
    )]
    pub order_account: Account<'info, Order>,


    // signer
    #[account(mut)]
    pub signer: Signer<'info>,

    // programs
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, token::Token>,
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
}

#[account]
pub struct Order {
    amount: u32,
    price: f64,
    mint: Pubkey,
    owner: Pubkey,
    order_type: OrderType,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum OrderType {
    Buy,
    Sell,
}

#[event]
pub struct OrderPlaced {
    amount: u32,
    price: f64,
    mint: Pubkey,
    order_account: Pubkey,
    signer: Pubkey,
    order_type: OrderType,
}
