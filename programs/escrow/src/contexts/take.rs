use crate::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{ transfer, Mint, Token, TokenAccount, Transfer, CloseAccount, close_account },
};

#[derive(Accounts)]
pub struct Take<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    #[account(mut)]
    pub maker: SystemAccount<'info>,

    // tokens to send and receive
    pub mint_a: Account<'info, Mint>,
    pub mint_b: Account<'info, Mint>,

    // taker's token accounts
    #[account(
        init_if_needed, 
        payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority = taker,
    )]
    pub taker_ata_a: Account<'info, TokenAccount>,

    // does already exist since taker is sending this token
    #[account(
        mut, 
        // constraints: guarantees the tokens coming in are the one we want
        associated_token::mint = mint_b,
        associated_token::authority = taker,
    )]
    pub taker_ata_b: Account<'info, TokenAccount>,

    #[account(
        init, 
        payer = taker,
        associated_token::mint = mint_b,
        associated_token::authority = maker,
    )]
    pub maker_ata_b: Account<'info, TokenAccount>,

    // maker creates the vault - derive the vault address from the escrow
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
    )]
    pub vault: Account<'info, TokenAccount>,

    // account to store escrow details
    #[account(
        mut, // so we can close the account
        close = maker,
        // note: prepend "escrow" to the seed to avoid collisions (turns "escrow" into bytes)
        seeds = [b"escrow".as_ref(), maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
        has_one = mint_a,
        has_one = mint_b
    )]
    pub escrow: Account<'info, Escrow>,

    pub system_program: Program<'info, System>,
    // since we're using spl tokens not lamports:
    pub token_program: Program<'info, Token>,
    // need this for ata
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl <'info>Take<'info> {
    pub fn deposit(&mut self) -> Result<()> {
        let accounts = Transfer {
            from: self.taker_ata_b.to_account_info(),
            to: self.maker_ata_b.to_account_info(),
            authority: self.taker.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), accounts);

        transfer(cpi_ctx, self.taker_ata_a.amount)
    }

    // Send money from vault to taker
    pub fn withdraw(&mut self) -> Result<()> {
        let seed = self.escrow.seed.to_le_bytes().clone();

        let signer_seeds = &[
            b"escrow", 
            self.maker.to_account_info().key.as_ref(),
            seed.as_ref(),
            &[self.escrow.bump]
            ];

        let binding = &[&signer_seeds[..]];

        let accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.taker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info()
        };

        // ! CpiContext::new_with_signer - use when signing on behalf of a PDA
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            binding
        );

        transfer(cpi_ctx, self.vault.amount)
    }

    // Close the vault
    pub fn close_vault(&mut self) -> Result<()> {
        // messy rust/anchor thing - just copy when needed
        let seed = self.escrow.seed.to_le_bytes().clone();

        let signer_seeds = &[
            b"escrow", 
            self.maker.to_account_info().key.as_ref(),
            seed.as_ref(),
            &[self.escrow.bump]
            ];

        let binding = &[&signer_seeds[..]];

        let close_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.taker.to_account_info(), // where to send the lamports from closing this account
            authority: self.escrow.to_account_info()
        };

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            close_accounts,
            binding
        );

        close_account(cpi_ctx)
    }

}