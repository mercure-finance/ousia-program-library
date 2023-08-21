import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { OusiaBurnAndMint } from "../target/types/ousia_burn_and_mint";
import { createSplToken } from "./tokens";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { mintSplTokens, transferTokens } from "./tokens";

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
    .OusiaBurnAndMint as Program<OusiaBurnAndMint>;

  const signer = loadKeypairFromFile("/Users/devenv/.config/solana/id.json");
  console.log("Signer", signer.publicKey.toBase58());

  const usdcMintKeypair = anchor.web3.Keypair.generate();
  const purchaseMintKeypair = anchor.web3.Keypair.generate();
  const orderAccountIdKeypair = anchor.web3.Keypair.generate();

  const signerUsdcATA = anchor.web3.PublicKey.findProgramAddressSync(
    [
      signer.publicKey.toBuffer(),
      TOKEN_PROGRAM_ID.toBuffer(),
      usdcMintKeypair.publicKey.toBuffer(),
    ],
    ASSOCIATED_TOKEN_PROGRAM_ID
  )[0];

  const signerPurchaseATA = anchor.web3.PublicKey.findProgramAddressSync(
    [
      signer.publicKey.toBuffer(),
      TOKEN_PROGRAM_ID.toBuffer(),
      purchaseMintKeypair.publicKey.toBuffer(),
    ],
    ASSOCIATED_TOKEN_PROGRAM_ID
  )[0];

  before(async () => {
    // Create SPL-Token mint.
    await createSplToken(
      usdcMintKeypair,
      signer,
      "USDC",
      "USDC",
      "https://www.usdc.com/"
    );

    await createSplToken(
      purchaseMintKeypair,
      signer,
      "Purchase",
      "PURCHASE",
      "https://www.purchase.com/"
    );

    await mintSplTokens(
      usdcMintKeypair.publicKey,
      signer,
      signer,
      signer.publicKey,
      10000000043434324
    );

    await mintSplTokens(
      purchaseMintKeypair.publicKey,
      signer,
      signer,
      signer.publicKey,
      1
    );
  });

  it("Create an order", async () => {
    const orderAccountATA = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("order"),
        signer.publicKey.toBuffer(),
        orderAccountIdKeypair.publicKey.toBuffer(),
      ],
      program.programId
    )[0];

    const orderUsdcATA = anchor.web3.PublicKey.findProgramAddressSync(
      [
        orderAccountATA.toBuffer(),
        TOKEN_PROGRAM_ID.toBuffer(),
        usdcMintKeypair.publicKey.toBuffer(),
      ],
      ASSOCIATED_TOKEN_PROGRAM_ID
    )[0];

    const orderPurchaseATA = anchor.web3.PublicKey.findProgramAddressSync(
      [
        orderAccountATA.toBuffer(),
        TOKEN_PROGRAM_ID.toBuffer(),
        purchaseMintKeypair.publicKey.toBuffer(),
      ],
      ASSOCIATED_TOKEN_PROGRAM_ID
    )[0];

    // Add your test here.
    const tx = await program.methods
      .placeBuyOrder(
        new anchor.BN(100),
        new anchor.BN(21135321),
        orderAccountIdKeypair.publicKey,
        { buy: {} }
      )
      .accounts({
        usdcMintAccount: usdcMintKeypair.publicKey,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        signer: signer.publicKey,
        buyerUsdcAta: signerUsdcATA,
        oderAccountUsdcAta: orderUsdcATA,
        orderAccount: orderAccountATA,
        mintAuthority: signer.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        purchaseTokenMintAccount: purchaseMintKeypair.publicKey,
        buyerPurchaseTokenAccount: signerPurchaseATA,
        orderAccountPurchaseTokenAccount: orderPurchaseATA,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc({ skipPreflight: true });

    console.log("Your transaction signature", tx);
  });
  program.addEventListener(
    "OrderPlaced",
    (event: OrderPlaced, slot, signature) => {
      console.log(
        "New order event",
        event.amount,
        event.price,
        event.mint.toBase58(),
        event.orderAccount.toBase58()
      );
      console.log("New order slot", slot);
      console.log("New order signature", signature);
    }
  );
});
