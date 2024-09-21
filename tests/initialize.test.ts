import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";

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
  MPL_TOKEN_METADATA_PROGRAM_ID,
} from "./utils";

describe("initialize", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const connection = provider.connection;

  const program = anchor.workspace.testVault as Program<TestVault>;

  it("Initializes the vault account and deposits into the vault token account", async () => {
    try {
      const owner = provider.wallet.publicKey;
      const mint = await createMint(provider);
      const ownerTokenAccount = await createTokenAccount(
        provider,
        provider.wallet.publicKey,
        mint,
        100_000 * LAMPORTS_PER_SOL
      );


      // how do I see if the ownerTokenAccount is initialized?
      const info = await provider.connection.getAccountInfo(ownerTokenAccount);

      const { vault, vaultTokenAccount, vaultAuthority, sharesAccount, metadataAccount } = await getPDAs({
        owner,
        programId: program.programId,
        mint,
      });

      const tokenProgram = anchor.utils.token.TOKEN_PROGRAM_ID;
      const tokenMetadataProgram = MPL_TOKEN_METADATA_PROGRAM_ID;
      const systemProgram = SystemProgram.programId;

      const transactionSignature = await program.methods
        .initializeVault(new anchor.BN(10))
        .accounts({
          vault,
          owner,
          mint,
          ownerTokenAccount,
          vaultAuthority,
          vaultTokenAccount,
          sharesAccount,
          metadataAccount,
          tokenProgram,
          tokenMetadataProgram,
          systemProgram,
        })
        // .signers([sharesKeyPair])
        .rpc();

        // const transaction = new Transaction().add(initializeTransaction);
        // transaction.feePayer = owner;
        // console.log("transaction", transaction);
 
        // const transactionSignature = await connection.simulateTransaction(transaction);

        // const logs = transactionSignature.value.logs;
        // console.log("[Initialize] Transaction logs:");
        // logs.forEach((log, index) => {
        //     console.log(`Log ${index + 1}: ${log}`);
        // });

      // console.log(`[Initialize] ${initializeTransaction}`);

      // const tx = await connection.getParsedTransaction(
      //   initializeTransaction,
      //   COMMITMENT
      // );

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

      // // Check data
      // const vaultData = await program.account.vault.fetch(vault);
      // console.log(vaultData);
      // expect(vaultData.owner.toBase58()).to.eq(owner.toBase58());
      // expect(vaultData.initialized).to.eq(true);

      // expect(vaultData.depositedAmount.toNumber()).to.eq(10);
      // expect(vaultData.mint.toBase58()).to.eql(mint.toBase58());
      // expect(vaultData.bumps.vault).to.not.eql(0);
      // expect(vaultData.bumps.vaultAuthority).to.not.eql(0);
      // expect(vaultData.bumps.vaultTokenAccount).to.not.eql(0);
    } catch (error) {
      console.error(error);
      throw new Error(`Failed to initialize vault: ${error.message}`);
    }
  });
});
