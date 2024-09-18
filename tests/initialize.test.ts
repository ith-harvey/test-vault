import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";

import { expect } from "chai";

import { Keypair, PublicKey, LAMPORTS_PER_SOL, SystemProgram, Transaction } from "@solana/web3.js";
import { TestVault } from "../target/types/test_vault";

import {
  COMMITMENT,
  //PDAAccounts,
  ParsedTokenTransfer,
  createMint,
  createTokenAccount,
  getPDAs,
} from "./utils";

describe("initialize", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const connection = provider.connection;

  const program = anchor.workspace.testVault as Program<TestVault>;
  const tokenProgram = anchor.utils.token.TOKEN_PROGRAM_ID;
  const systemProgram = SystemProgram.programId;

  it("Initializes the vault account and shares accounts", async () => {
    try {
      const owner = provider.wallet.publicKey;
      const mint = await createMint(provider);
      const ownerTokenAccount = await createTokenAccount(
        provider,
        provider.wallet.publicKey,
        mint,
        100_000 * LAMPORTS_PER_SOL
      );

      const { vault, vaultTokenAccount, vaultAuthority, sharesMint } = await getPDAs({
        owner,
        programId: program.programId,
        mint,
      });


      const transactionSignature = await program.methods
        .initializeVault()
        .accounts({
          vault,
          owner,
          mint,
          ownerTokenAccount,
          vaultAuthority,
          vaultTokenAccount,
          sharesMint,
          tokenProgram,
          systemProgram,
        })
        .rpc();

      console.log(`[Initialize] ${transactionSignature}`);

      // // Ensure that inner transfer succeded.
      // const transferIx: any = tx.meta.innerInstructions[0].instructions.find(
      //   (ix) =>
      //     (ix as any).parsed.type === "transfer" &&
      //     ix.programId.toBase58() == spl.TOKEN_PROGRAM_ID.toBase58()
      // );
      // const parsedInfo: ParsedTokenTransfer = transferIx.parsed.info;
      // expect(parsedInfo).eql({
      //   amount: "10",
      //   authority: owner.toBase58(),
      //   destination: vaultTokenAccount.toBase58(),
      //   source: ownerTokenAccount.toBase58(),
      // });

      // Check data
      const vaultData = await program.account.vault.fetch(vault);
      console.log(vaultData);
      expect(vaultData.owner.toBase58()).to.eq(owner.toBase58());
      expect(vaultData.initialized).to.eq(true);

      expect(vaultData.mint.toBase58()).to.eql(mint.toBase58());
      expect(vaultData.bumps.vault).to.not.eql(0);
      expect(vaultData.bumps.vaultAuthority).to.not.eql(0);
      expect(vaultData.bumps.vaultTokenAccount).to.not.eql(0);
    } catch (error) {
      console.error(error);
      throw new Error(`Failed to initialize vault: ${error.message}`);
    }
  });

  it("Deposits 10 tokens into the vault", async () => {
    try {

      const owner = provider.wallet.publicKey;
      const mint = await createMint(provider);
      const ownerTokenAccount = await createTokenAccount(
        provider,
        provider.wallet.publicKey,
        mint,
        100_000 * LAMPORTS_PER_SOL
      );

      const { vault, vaultTokenAccount, vaultAuthority, sharesMint } = await getPDAs({
        owner,
        programId: program.programId,
        mint,
      });


      await program.methods
        .initializeVault()
        .accounts({
          vault,
          owner,
          mint,
          ownerTokenAccount,
          vaultAuthority,
          vaultTokenAccount,
          sharesMint,
          tokenProgram,
          systemProgram,
        })
        .rpc();

      const ownerSharesAccount = getAssociatedTokenAddressSync(sharesMint, owner);

      const transactionSignature = await program.methods
        .deposit(new anchor.BN(10))
        .accounts({
          owner,
          ownerTokenAccount,
          mint,
          vault,
          vaultAuthority,
          vaultTokenAccount,
          sharesMint,
          ownerSharesAccount,
          tokenProgram,
          systemProgram,
        })
        .rpc();

      console.log(`[Deposit] ${transactionSignature}`);

      // Check data
      const vaultData = await program.account.vault.fetch(vault);
      expect(vaultData.deposit.toNumber()).to.eq(10);

      // Check token balances
      const vaultTokenAccountInfo = await spl.getAccount(connection, vaultTokenAccount);
      expect(vaultTokenAccountInfo.amount).to.eq(BigInt(10));

      // Check owner's shares balance
      const ownerSharesAccountInfo = await spl.getAccount(connection, ownerSharesAccount);
      expect(ownerSharesAccountInfo.amount).to.eq(BigInt(10), "Owner's shares balance should match the deposit amount");

    } catch (error) {
      console.error(error);
      throw new Error(`Failed to deposit into vault: ${error.message}`);
    }
  });

  it("Withdraws all 10 tokens from the vault", async () => {
    try {
      const owner = provider.wallet.publicKey;
      const mint = await createMint(provider);
      const ownerTokenAccount = await createTokenAccount(
        provider,
        provider.wallet.publicKey,
        mint,
        100_000 * LAMPORTS_PER_SOL
      );

      const { vault, vaultTokenAccount, vaultAuthority, sharesMint } = await getPDAs({
        owner,
        programId: program.programId,
        mint,
      });

      // Initialize the vault
      await program.methods
        .initializeVault()
        .accounts({
          vault,
          owner,
          mint,
          ownerTokenAccount,
          vaultAuthority,
          vaultTokenAccount,
          sharesMint,
          tokenProgram,
          systemProgram,
        })
        .rpc();

      const ownerSharesAccount = getAssociatedTokenAddressSync(sharesMint, owner);

      // Deposit 10 tokens
      await program.methods
        .deposit(new anchor.BN(10))
        .accounts({
          owner,
          ownerTokenAccount,
          mint,
          vault,
          vaultAuthority,
          vaultTokenAccount,
          sharesMint,
          ownerSharesAccount,
          tokenProgram,
          systemProgram,
        })
        .rpc();

      // Withdraw all 10 tokens
      const withdrawSignature = await program.methods
        .withdraw(new anchor.BN(10))
        .accounts({
          owner,
          ownerTokenAccount,
          mint,
          vault,
          vaultAuthority,
          vaultTokenAccount,
          sharesMint,
          ownerSharesAccount,
          tokenProgram,
        })
        .rpc();

      console.log(`[Withdraw] ${withdrawSignature}`);

      // Check withdraw data
      const vaultData = await program.account.vault.fetch(vault);
      expect(vaultData.deposit.toNumber()).to.eq(0);

      // Check token balances after withdrawal
      const vaultTokenAccountInfo = await spl.getAccount(connection, vaultTokenAccount);
      expect(vaultTokenAccountInfo.amount).to.eq(BigInt(0), "Vault token account should be empty after withdrawal");

      // Check owner's shares balance after withdrawal
      const ownerSharesAccountInfo = await spl.getAccount(connection, ownerSharesAccount);
      expect(ownerSharesAccountInfo.amount).to.eq(BigInt(0), "Owner's shares balance should be zero after withdrawal");

    } catch (error) {
      console.error(error);
      throw new Error(`Failed to withdraw from vault: ${error.message}`);
    }
  });
});
