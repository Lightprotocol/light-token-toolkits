/**
 * Transfer c-Tokens to a recipient
 *
 * Configure these values:
 */
const mintPubkey = "your-mint-pubkey"; // e.g. USDC: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
const recipientPubkey = "your-recipient-pubkey";
const amount = 1000000; // in smallest units (e.g. 1 USDC = 1000000)

import { createRpc } from "@lightprotocol/stateless.js";
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  getAssociatedTokenAddressInterface,
  transferInterface,
} from "@lightprotocol/compressed-token/unified";
import * as fs from "fs";
import * as path from "path";
import "dotenv/config";

const rpc = createRpc(`https://devnet.helius-rpc.com?api-key=${process.env.HELIUS_API_KEY}`);
const payer = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(path.join(process.env.HOME || "~", ".config/solana/id.json"), "utf-8"))));

async function main() {
  if (
    mintPubkey === "your-mint-pubkey" ||
    recipientPubkey === "your-recipient-pubkey"
  ) {
    console.error("Configure mint and recipient");
    process.exit(1);
  }
  const mint = new PublicKey(mintPubkey);
  const recipient = new PublicKey(recipientPubkey);

  const sourceAta = getAssociatedTokenAddressInterface(mint, payer.publicKey);
  const destinationAta = getAssociatedTokenAddressInterface(mint, recipient);

  const signature = await transferInterface(
    rpc,
    payer,
    sourceAta,
    mint,
    destinationAta,
    payer,
    amount
  );

  console.log(`https://explorer.solana.com/tx/${signature}?cluster=devnet`);
}

main().catch(console.error);
