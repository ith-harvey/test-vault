import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import { expect } from "chai";
import { PublicKey, LAMPORTS_PER_SOL, SystemProgram } from "@solana/web3.js";
import { TestVault } from "../target/types/test_vault";
import { COMMITMENT, createMint, createTokenAccount, getPDAs } from "./utils";

describe("initialize", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const connection = provider.connection;
  const program = anchor.workspace.testVault as Program<TestVault>;
  const tokenProgram = anchor.utils.token.TOKEN_PROGRAM_ID;
  const systemProgram = SystemProgram.programId;

  let owner: PublicKey, mint: PublicKey, ownerTokenAccount: PublicKey;
  let vault: PublicKey, vaultTokenAccount: PublicKey, vaultAuthority: PublicKey, sharesMint: PublicKey;

  async function setup() {
    owner = provider.wallet.publicKey;
    mint = await createMint(provider);
    ownerTokenAccount = await createTokenAccount(
      provider,
      owner,
      mint,
      100_000 * LAMPORTS_PER_SOL
    );

    ({ vault, vaultTokenAccount, vaultAuthority, sharesMint } = await getPDAs({
      owner,
      programId: program.programId,
      mint,
    }));
  }

  async function initializeVault() {
    try {
      const tx = await program.methods
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
      console.log(`[Initialize] ${tx}`);
      return tx;
    } catch (error) {
      console.error("Error initializing vault:", error);
      throw error;
    }
  }

  beforeEach(setup);

  it("Initializes the vault account and shares accounts", async () => {
    await initializeVault();

    const vaultData = await program.account.vault.fetch(vault);
    expect(vaultData.owner.toBase58()).to.eq(owner.toBase58());
    expect(vaultData.initialized).to.eq(true);
    expect(vaultData.mint.toBase58()).to.eql(mint.toBase58());
    expect(vaultData.bumps.vault).to.not.eql(0);
    expect(vaultData.bumps.vaultAuthority).to.not.eql(0);
    expect(vaultData.bumps.vaultTokenAccount).to.not.eql(0);
  });

  it("Deposits 10 tokens into the vault", async () => {
    await initializeVault();
    const ownerSharesAccount = getAssociatedTokenAddressSync(sharesMint, owner);

    try {
      const tx = await program.methods
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
        .rpc(COMMITMENT);
      console.log(`[Deposit] ${tx}`);

      const vaultData = await program.account.vault.fetch(vault);
      expect(vaultData.deposit.toNumber()).to.eq(10);

      const vaultTokenAccountInfo = await spl.getAccount(connection, vaultTokenAccount);
      expect(vaultTokenAccountInfo.amount).to.eq(BigInt(10));

      const ownerSharesAccountInfo = await spl.getAccount(connection, ownerSharesAccount);
      expect(ownerSharesAccountInfo.amount).to.eq(BigInt(10), "Owner's shares balance should match the deposit amount");
    } catch (error) {
      console.error("Error depositing tokens:", error);
      throw error;
    }
  });

  it("Withdraws all 10 tokens from the vault", async () => {
    await initializeVault();
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
      .rpc(COMMITMENT);

    // Withdraw all 10 tokens
    try {
      const tx = await program.methods
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
        .rpc(COMMITMENT);
      console.log(`[Withdraw] ${tx}`);

      const vaultData = await program.account.vault.fetch(vault);
      expect(vaultData.deposit.toNumber()).to.eq(0);

      const vaultTokenAccountInfo = await spl.getAccount(connection, vaultTokenAccount);
      expect(vaultTokenAccountInfo.amount).to.eq(BigInt(0), "Vault token account should be empty after withdrawal");

      const ownerSharesAccountInfo = await spl.getAccount(connection, ownerSharesAccount);
      expect(ownerSharesAccountInfo.amount).to.eq(BigInt(0), "Owner's shares balance should be zero after withdrawal");
    } catch (error) {
      console.error("Error withdrawing tokens:", error);
      throw error;
    }
  });
});
