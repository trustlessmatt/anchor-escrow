use anchor_lang::prelude::*;

declare_id!("8jvtttXC6tCC5An9midSavzVcA4kH8UUuKWozNTUicqR");

pub mod contexts;
pub use contexts::*;

pub mod state;
use state::*;

#[program]
pub mod escrow {
    use super::*;

    pub fn make(ctx: Context<Make>, seed: u64, deposit: u64, receive: u64) -> Result<()> {
        ctx.accounts.save_escrow(seed, receive, &ctx.bumps)?; // cheaper to pass reference to bumps than to clone it
        ctx.accounts.make(deposit)
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund()
        // now we need to close the vault
    }


}
