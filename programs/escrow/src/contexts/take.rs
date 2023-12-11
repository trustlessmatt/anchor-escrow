use crate::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{ close_account, transfer, CloseAccount, Mint, Token, TokenAccount, Transfer },
};


// pub fn take(ctx: Context<Take>, seed: u64, make_amount: u64, take_amount: u64) -> Result<()> {
    //     // saving state to escrow here
    //     ctx.accounts.escrow.set_inner(Escrow{
    //         maker_token: ctx.accounts.maker_token.key(),
    //         receive_amount: take_amount,
    //         taker_token: ctx.accounts.taker_token.key(),
    //         bump: ctx.bumps.escrow,
    //         seed,
    //     });
    //     // define accounts
    //     let accounts = Transfer {
    //         from: ctx.accounts.maker_ata_a.to_account_info(),
    //         to: ctx.accounts.escrow_ata.to_account_info(),
    //         authority: ctx.accounts.signer.to_account_info(),
    //     };
    //     // create cpi context
    //     let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), accounts);
    //     // transfer tokens
    //     transfer(cpi_ctx, make_amount)
    // }