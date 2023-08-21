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

const usdcMintKeypair = loadKeypairFromFile(
  "/Users/devenv/Documents/GitHub/ousia-program-library/tests/keypair/usdcMintKeypair.json"
);

const teslaMintKeypair = loadKeypairFromFile(
  "/Users/devenv/Documents/GitHub/ousia-program-library/tests/keypair/teslaStockKeypair.json"
);

const coinMintKeypair = loadKeypairFromFile(
  "/Users/devenv/Documents/GitHub/ousia-program-library/tests/keypair/coinStockKeypair.json"
);

const aaplMintKeypair = loadKeypairFromFile(
  "/Users/devenv/Documents/GitHub/ousia-program-library/tests/keypair/aaplStockKeypair.json"
);

const signer = loadKeypairFromFile("/Users/devenv/.config/solana/id.json");
console.log("Signer", signer.publicKey.toBase58());

const createTokens = async () => {
  await createSplToken(
    usdcMintKeypair,
    signer,
    "USDC",
    "USDC",
    "https://www.usdc.com/"
  );

  console.log(usdcMintKeypair.publicKey.toBase58());

  await createSplToken(
    teslaMintKeypair,
    signer,
    "Tokenized - Tesla",
    "TSLA",
    "https://www.purchase.com/"
  );

  console.log(teslaMintKeypair.publicKey.toBase58());

  await createSplToken(
    aaplMintKeypair,
    signer,
    "Tokenized - Apple, Inc.",
    "AAPL",
    "https://www.purchase.com/"
  );

  await createSplToken(
    coinMintKeypair,
    signer,
    "Tokenized - Coinbase Global, Inc.",
    "COIN",
    "https://www.purchase.com/"
  );

  mintSplTokens(
    usdcMintKeypair.publicKey,
    signer,
    signer,
    new anchor.web3.PublicKey("EPkatMEFsRbohGELRDKsvQE6XNe1iy4evhjoGy4oHYkE"),
    1000000000000
  );

  mintSplTokens(
    teslaMintKeypair.publicKey,
    signer,
    signer,
    new anchor.web3.PublicKey("EPkatMEFsRbohGELRDKsvQE6XNe1iy4evhjoGy4oHYkE"),
    1 * 10 ** 6
  );

  mintSplTokens(
    aaplMintKeypair.publicKey,
    signer,
    signer,
    new anchor.web3.PublicKey("EPkatMEFsRbohGELRDKsvQE6XNe1iy4evhjoGy4oHYkE"),
    3 * 10 ** 6
  );

  mintSplTokens(
    coinMintKeypair.publicKey,
    signer,
    signer,
    new anchor.web3.PublicKey("EPkatMEFsRbohGELRDKsvQE6XNe1iy4evhjoGy4oHYkE"),
    1 * 10 ** 6
  );
};

createTokens();
