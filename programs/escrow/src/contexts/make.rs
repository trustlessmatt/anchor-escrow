use crate::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{ transfer, Mint, Token, TokenAccount, Transfer },
};

#[derive(Accounts)]
// need a seed if we want to create multiple escrows
#[instruction(seed: u64)]
pub struct Make<'info> {
    // creator of escrow
    #[account(mut)]
    pub maker: Signer<'info>,

    // token to send and receive
    pub mint_a: Account<'info, Mint>,
    pub mint_b: Account<'info, Mint>,

    // maker's token account
    #[account(
        mut, 
        // constraints: guarantees the tokens coming in are the one we want
        associated_token::mint = mint_a,
        associated_token::authority = maker,
    )]
    pub maker_ata_a: Account<'info, TokenAccount>,

    // maker creates the vault - derive the vault address from the escrow
    #[account(
        init,
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
    )]
    pub vault: Account<'info, TokenAccount>,

    // account to store escrow details
    #[account(
        init,
        payer = maker,
        space = Escrow::INIT_SPACE,
        // note: prepend "escrow" to the seed to avoid collisions (turns "escrow" into bytes)
        seeds = [b"escrow".as_ref(), maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub escrow: Account<'info, Escrow>,

    pub system_program: Program<'info, System>,
    // since we're using spl tokens not lamports:
    pub token_program: Program<'info, Token>,
    // need this for ata
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info>Make<'info> {
  pub fn save_escrow(&mut self, seed: u64, receive: u64, bumps: &MakeBumps) -> Result<()> {
    self.escrow.set_inner(Escrow {
      seed,
      mint_a: self.mint_a.key(),
      mint_b: self.mint_b.key(),
      receive,
      bump: bumps.escrow,
    });
    Ok(())
  }

  // note: if we want to create multiple escrows, we need to pass in a seed
  pub fn make(&mut self, deposit: u64) -> Result<()> {
      // alt to set_inner:
      // ctx.accounts.escrow.seed = seed;
      // ctx.accounts.escrow.mint_a = ctx.accounts.mint_a.key();
      // ...

      // another alt to set_inner:
      // let escrow = &mut ctx.accounts.escrow;
      // escrow.seed = seed;
      // escrow.mint_a = ctx.accounts.mint_a.key();
      // ...

      // define accounts
      let transfer_accounts = Transfer {
          // from maker's token account
          from: self.maker_ata_a.to_account_info(),
          // to vault
          to: self.vault.to_account_info(),
          // authority is maker
          authority: self.maker.to_account_info(),
      };
      // create cpi context
      let cpi_ctx = CpiContext::new(
          self.token_program.to_account_info(), 
          transfer_accounts
      );
      // transfer tokens
      transfer(cpi_ctx, deposit)
  }
}
