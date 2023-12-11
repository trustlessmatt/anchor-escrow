import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { Escrow } from "../target/types/escrow";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  Transaction,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";

import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountIdempotentInstruction,
  createInitializeMint2Instruction,
  createMintToInstruction,
  getAssociatedTokenAddressSync,
  getMinimumBalanceForRentExemptMint,
  MINT_SIZE,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { randomBytes } from "crypto";

describe("escrow", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Escrow as Program<Escrow>;
  const provider = anchor.getProvider();
  const connection = provider.connection;

  const signer = Keypair.generate();

  const confirm = async (signature: string) => {
    const block = await connection.getLatestBlockhash();
    await connection.confirmTransaction({ signature, ...block });
    return signature;
  };

  const log = async (signature: string): Promise<string> => {
    console.log(`Your transaction signature: https://explorer.solana.com/transaction/${signature}?
    cluster=custom&customUrl=${connection.rpcEndpoint}`);
    return signature;
  };

  it("Airdrop", async () => {
    await connection
      .requestAirdrop(signer.publicKey, LAMPORTS_PER_SOL * 10)
      .then(confirm)
      .then(log);
  });

  /////// escrow test ///////
  const seed = new BN(randomBytes(8));

  const maker = Keypair.generate();
  const taker = Keypair.generate();
  const mintA = Keypair.generate();
  const mintB = Keypair.generate();
  const makerAtaA = getAssociatedTokenAddressSync(
    mintA.publicKey,
    maker.publicKey
  );
  const makerAtaB = getAssociatedTokenAddressSync(
    mintB.publicKey,
    maker.publicKey
  );
  const takerAtaA = getAssociatedTokenAddressSync(
    mintA.publicKey,
    taker.publicKey
  );
  const takerAtaB = getAssociatedTokenAddressSync(
    mintB.publicKey,
    taker.publicKey
  );

  const escrow = PublicKey.findProgramAddressSync(
    [Buffer.from("escrow"), maker.publicKey.toBuffer(), seed.toBuffer("le", 8)],
    program.programId
  )[0];

  it("Create token mints", async () => {
    let lamports = await getMinimumBalanceForRentExemptMint(connection);
    let tx = new Transaction();
    tx.instructions = [
      // create mint accounts
      // mint a
      SystemProgram.createAccount({
        fromPubkey: provider.publicKey,
        newAccountPubkey: mintA.publicKey,
        space: MINT_SIZE,
        lamports,
        programId: TOKEN_PROGRAM_ID,
      }),
      // mint b
      SystemProgram.createAccount({
        fromPubkey: provider.publicKey,
        newAccountPubkey: mintB.publicKey,
        space: MINT_SIZE,
        lamports,
        programId: TOKEN_PROGRAM_ID,
      }),
      // initialize mint accounts
      // maker mint (a)
      createInitializeMint2Instruction(
        mintA.publicKey,
        6,
        maker.publicKey,
        null
      ),
      // taker mint (b)
      createInitializeMint2Instruction(
        mintB.publicKey,
        6,
        taker.publicKey,
        null
      ),
      createAssociatedTokenAccountIdempotentInstruction(
        provider.publicKey,
        makerAtaA,
        maker.publicKey,
        mintA.publicKey
      ),
      createAssociatedTokenAccountIdempotentInstruction(
        provider.publicKey,
        takerAtaB,
        taker.publicKey,
        mintB.publicKey
      ),
      createMintToInstruction(mintA.publicKey, makerAtaA, maker.publicKey, 1e9),
      createMintToInstruction(mintB.publicKey, takerAtaB, taker.publicKey, 1e9),
    ];
    await provider.sendAndConfirm(tx, [mintA, mintB, maker, taker]).then(log);
  });

  it("Make", async () => {
    await program.methods
      .make(seed, new BN(1e6), new BN(1e6))
      .accounts({
        maker: maker.publicKey,
        mintA: mintA.publicKey,
        mintB: mintB.publicKey,
        makerAtaA,
        escrow,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([maker])
      .rpc()
      .then(confirm)
      .then(log);
  });

  it("Refund", async () => {
    await program.methods
      .refund()
      .accounts({
        maker: maker.publicKey,
        mintA: mintA.publicKey,
        makerAtaA,
        escrow,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([maker])
      .rpc()
      .then(confirm)
      .then(log);
  });

  // xit("Take", async () => {
  //   await program.methods
  //     .take()
  //     .accounts({
  //       taker: taker.publicKey,
  //       maker: maker.publicKey,
  //       mintA: mintA.publicKey,
  //       mintB: mintB.publicKey,
  //       takerAtaA,
  //       takerAtaB,
  //       makerAtaB,
  //       escrow,
  //       vault,
  //       associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       systemProgram: SystemProgram.programId,
  //     })
  //     .signers([taker])
  //     .rpc({ skipPreflight: true })
  //     .then(confirm)
  //     .then(log);
  // });
});
