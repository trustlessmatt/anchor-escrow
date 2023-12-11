use anchor_lang::prelude::*;

#[account]
pub struct Escrow {
    pub mint_a: Pubkey,
    pub mint_b: Pubkey, 
    pub receive: u64,
    pub bump: u8,
    pub seed: u64,
}

impl Space for Escrow {
    const INIT_SPACE: usize = 8 + 8 + 32 + 32 + 8 + 1 + 1;
    // discriminator = 8 bytes
    // seed = 8 bytes
    // 32 bytes for each mint
    // 8 bytes for receive amount
    // 1 byte for each bump
}