import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { OusiaBurnAndMint } from "../target/types/ousia_burn_and_mint";
import { createSplToken } from "./tokens";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { mintSplTokens, transferTokens } from "./tokens";
import { createClient } from "@supabase/supabase-js";

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
  signer: anchor.web3.PublicKey;
}

const stocks = [
  {
    name: "TSLA",
    mint: "",
  },
  {
    name: "COIN",
    mint: "",
  },
  {
    name: "AAPL",
    mint: "",
  },
];
const supabase = createClient(
  "http://localhost:8000",
  "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyAgCiAgICAicm9sZSI6ICJzZXJ2aWNlX3JvbGUiLAogICAgImlzcyI6ICJzdXBhYmFzZS1kZW1vIiwKICAgICJpYXQiOiAxNjQxNzY5MjAwLAogICAgImV4cCI6IDE3OTk1MzU2MDAKfQ.DaYlNEoUrrEn2Ig7tqibS-PHK5vgusbcbo7X36XVt4Q"
);

describe("ousia-program-library", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace
    .OusiaBurnAndMint as Program<OusiaBurnAndMint>;

  it("Create an order", async () => {});
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

      const amount = event.amount / 10 ** 6;

      console.log(amount, "amount");

      console.log(event.amount, "event.amount");

      supabase
        .from("orders")
        .insert([
          {
            amount: event.amount,
            price: event.price * 10 ** 6,
            stock: "TSLA",
            type: "BUY",
            user: event.signer.toBase58(),
            status: "open",
          },
        ])
        .then((res) => {
          console.log("res", res);
        });
    }
  );
});
