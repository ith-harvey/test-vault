import * as anchor from "@coral-xyz/anchor";
import {
  PublicKey,
  Keypair,
  Connection,
  LAMPORTS_PER_SOL,
  Finality,
} from "@solana/web3.js";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction,
  createMint as createSPLMint,
  getMinimumBalanceForRentExemptMint,
  createMintToInstruction,
} from "@solana/spl-token";


export interface Params {
  cliffSeconds: anchor.BN;
  durationSeconds: anchor.BN;
  secondsPerSlice: anchor.BN;
  startUnix: anchor.BN;
  grantTokenAmount: anchor.BN;
}

export interface ParsedTokenTransfer {
  amount: string;
  authority: string;
  destination: string;
  source: string;
}

export interface PDAAccounts {
  vault: PublicKey;
  vaultTokenAccount: PublicKey;
  vaultAuthority: PublicKey;
  sharesMint: PublicKey; // Add this line
}

export const COMMITMENT: { commitment: Finality } = { commitment: "confirmed" };

export const createTokenAccount = async (
  provider: anchor.AnchorProvider,
  user: anchor.web3.PublicKey,
  mint: anchor.web3.PublicKey,
  fundingAmount?: number
): Promise<anchor.web3.PublicKey> => {
  const userAssociatedTokenAccount = await getAssociatedTokenAddress(
    mint,
    user,
    false,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
  );

  // Fund user with some SOL
  let txFund = new anchor.web3.Transaction();
  if (user.toBase58() !== provider.wallet.publicKey.toBase58()) {
    txFund.add(
      anchor.web3.SystemProgram.transfer({
        fromPubkey: provider.wallet.publicKey,
        toPubkey: user,
        lamports: 5 * anchor.web3.LAMPORTS_PER_SOL,
      })
    );
  }
  txFund.add(
    createAssociatedTokenAccountInstruction(
      provider.wallet.publicKey,
      userAssociatedTokenAccount,
      user,
      mint,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    )
  );
  if (fundingAmount !== undefined) {
    txFund.add(
      createMintToInstruction(
        mint,
        userAssociatedTokenAccount,
        provider.wallet.publicKey,
        fundingAmount,
        [],
        TOKEN_PROGRAM_ID
      )
    );
  }

  const txFundTokenSig = await provider.sendAndConfirm(txFund, [], COMMITMENT);
  console.log(
    `[${userAssociatedTokenAccount.toBase58()}] New associated account for mint ${mint.toBase58()}: ${txFundTokenSig}`
  );
  return userAssociatedTokenAccount;
};

export const createMint = async (
  provider: anchor.AnchorProvider
): Promise<anchor.web3.PublicKey> => {
  const wallet = provider.wallet;
  const connection = provider.connection;

  // Get the minimum lamports required for the mint
  const lamportsForMint = await getMinimumBalanceForRentExemptMint(connection);

  // Create the mint account
  const mint = await createSPLMint(
    connection,
    wallet.payer,
    wallet.publicKey,
    wallet.publicKey,
    9,
    undefined,
    { commitment: COMMITMENT.commitment },
    TOKEN_PROGRAM_ID
  );

  console.log(`[${mint.toBase58()}] Created new mint account`);
  return mint;
};

export const getPDAs = async (params: {
  programId: PublicKey;
  owner: PublicKey;
  mint: PublicKey;
}): Promise<PDAAccounts> => {
  const [vault] = await PublicKey.findProgramAddress(
    [Buffer.from("vault"), params.owner.toBuffer(), params.mint.toBuffer()],
    params.programId
  );
  const [vaultAuthority] = await PublicKey.findProgramAddress(
    [Buffer.from("authority"), vault.toBuffer()],
    params.programId
  );
  const [vaultTokenAccount] = await PublicKey.findProgramAddress(
    [Buffer.from("tokens"), vault.toBuffer()],
    params.programId
  );
  const [sharesMint] = await PublicKey.findProgramAddress(
    [Buffer.from("shares_mint"), vault.toBuffer()],
    params.programId
  );
return {
    vault,
    vaultAuthority,
    vaultTokenAccount,
    sharesMint,
  };
};