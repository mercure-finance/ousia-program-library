import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CollateralizedDebt } from "../target/types/collateralized_debt";
import { createSplToken } from "./tokens";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";
import { mintSplTokens, transferTokens } from "./tokens";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";

export function loadKeypairFromFile(path: string): anchor.web3.Keypair {
  return anchor.web3.Keypair.fromSecretKey(
    Buffer.from(JSON.parse(require("fs").readFileSync(path, "utf-8")))
  );
}

export interface OrderPlaced {
  amount: number;
  price: number;
  mint: anchor.web3.PublicKey;
  orderAccount: anchor.web3.PublicKey;
}

describe("ousia-program-library", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace
    .CollateralizedDebt as Program<CollateralizedDebt>;

  const signer = loadKeypairFromFile("/Users/devenv/.config/solana/id.json");

  const usdcMint = new anchor.web3.PublicKey(
    "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"
  );

  const usdcPythFeed = new anchor.web3.PublicKey(
    "5SSkXsEKQepHHAewytPVwdej4epN1nxgLVM84L4KXgy7"
  );

  const euroPythFeed = new anchor.web3.PublicKey(
    "BwkRMkWjfMvWeFLqfQzwhumQan4CxNtyQRbKxtkV4yzi"
  );

  const assetAccount = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("asset"), euroPythFeed.toBuffer(), Buffer.alloc(1)],
    program.programId
  )[0];

  console.log("assetAccount", assetAccount);

  const mintAccount = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("asset_account"), assetAccount.toBuffer()],
    program.programId
  )[0];

  const ata = getAssociatedTokenAddressSync(
    mintAccount,
    signer.publicKey,
    true,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
  );

  const signerMintATA = anchor.web3.PublicKey.findProgramAddressSync(
    [
      signer.publicKey.toBuffer(),
      TOKEN_PROGRAM_ID.toBuffer(),
      mintAccount.toBuffer(),
    ],
    ASSOCIATED_TOKEN_PROGRAM_ID
  )[0];

  const createKey = anchor.web3.Keypair.generate();

  const positionAccount = anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("position"),
      assetAccount.toBuffer(),
      createKey.publicKey.toBuffer(),
    ],
    program.programId
  )[0];

  const positionUsdcATA = anchor.web3.PublicKey.findProgramAddressSync(
    [
      positionAccount.toBuffer(),
      TOKEN_PROGRAM_ID.toBuffer(),
      usdcMint.toBuffer(),
    ],
    ASSOCIATED_TOKEN_PROGRAM_ID
  )[0];

  const mintAuthorityAccount = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("mint-authority"), mintAccount.toBuffer()],
    program.programId
  )[0];

  const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey(
    "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
  );

  const metadataAddress = anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      mintAccount.toBuffer(),
    ],
    TOKEN_METADATA_PROGRAM_ID
  )[0];

  before(async () => {
    console.log("before");
  });

  it("Create an asset", async () => {
    const signature = await program.methods
      .createNewAsset(
        false,
        2,
        120,
        "Mercure Coinbase",
        "mCOIN",
        "https://raw.githubusercontent.com/ousia-finance/token-metadata/main/coinbase.json"
      )
      .accounts({
        signer: signer.publicKey,
        assetAccount,
        mintAccount,
        mintAuthority: mintAuthorityAccount,
        priceFeed: euroPythFeed,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        metadataAccount: metadataAddress,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc({ skipPreflight: true });
    await program.provider.connection.confirmTransaction(signature);
    console.log(signature);
  });

  it("Create a new position", async () => {
    console.log("mint account key", mintAccount.toBase58());
    // wait 10s before continuing
    const mintAccountInfo = await program.provider.connection.getAccountInfo(
      mintAccount
    );
    console.log("mint account info", mintAccountInfo);
    console.log("asset account key", assetAccount.toBase58());
    console.log("position account key", positionAccount.toBase58());
    console.log("create key", createKey.publicKey.toBase58());
    console.log("signer", signer.publicKey.toBase58());
    console.log("mint authority", signer.publicKey.toBase58());

    const data = await program.account.assetAccount.fetch(assetAccount);

    console.log("asset account data", data);

    // TO-DO Figure out why it sends to same address

    const transfer = await transferTokens(
      usdcMint,
      signer,
      signer,
      positionAccount,
      5000000
    );

    console.log("transfer sig", transfer);
    program.provider.connection.confirmTransaction(transfer);

    const signature = await program.methods
      .openPosition(new anchor.BN(0.000000001 * 10 ** 6), false)
      .accounts({
        signer: signer.publicKey,
        assetAccount,
        mintAccount,
        mintAuthority: mintAuthorityAccount,
        associatedTokenAccount: signerMintATA,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        positionAccount,
        createKey: createKey.publicKey,
        priceFeed: euroPythFeed,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .remainingAccounts([
        {
          pubkey: positionUsdcATA,
          isWritable: false,
          isSigner: false,
        },
        {
          pubkey: usdcPythFeed,
          isWritable: false,
          isSigner: false,
        },
      ])
      .signers([signer, createKey])
      .rpc({ skipPreflight: true });

    console.log(signature);

    const verifyOwnerAdress = await program.account.positionAccount.all([
      {
        memcmp: {
          offset: 8,
          bytes: signer.publicKey.toBase58(),
        },
      },
      {
        memcmp: {
          offset: 40,
          bytes: mintAccount.toBase58(),
        },
      },
    ]);

    verifyOwnerAdress.map((item) => {
      console.log("create keys", item.account.createKey.toBase58());
    });
  });

  it("Close a position", async () => {
    const signature = await program.methods
      .closePosition(false, euroPythFeed, createKey.publicKey)
      .accounts({
        signer: signer.publicKey,
        assetAccount,
        mintAccount,
        mintAuthority: mintAuthorityAccount,
        associatedTokenAccount: signerMintATA,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        positionAccount,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc({ skipPreflight: true });

    console.log(signature);
  });
});
