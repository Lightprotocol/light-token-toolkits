/**
 * Create a c-Token ATA
 *
 * Configure these values:
 */
const mintPubkey = "your-mint-pubkey"; // e.g. USDC: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
const recipientPubkey = "your-recipient-pubkey";

import { createRpc, CTOKEN_PROGRAM_ID } from "@lightprotocol/stateless.js";
import { Keypair, PublicKey, Transaction, sendAndConfirmTransaction } from "@solana/web3.js";
import {
  createAssociatedTokenAccountInterfaceIdempotentInstruction,
  createLoadAtaInstructions,
  getAssociatedTokenAddressInterface,
} from "@lightprotocol/compressed-token/unified";
import * as fs from "fs";
import * as path from "path";
import "dotenv/config";

const rpc = createRpc(`https://devnet.helius-rpc.com?api-key=${process.env.HELIUS_API_KEY}`);
const payer = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(path.join(process.env.HOME || "~", ".config/solana/id.json"), "utf-8"))));

async function main() {
  if (mintPubkey === "your-mint-pubkey" || recipientPubkey === "your-recipient-pubkey") {
    console.error("Configure mint and recipient");
    process.exit(1);
  }

  const mint = new PublicKey(mintPubkey);
  const recipient = new PublicKey(recipientPubkey);
  const ata = getAssociatedTokenAddressInterface(mint, recipient);

  const tx = new Transaction().add(
    createAssociatedTokenAccountInterfaceIdempotentInstruction(
      payer.publicKey,
      ata,
      recipient,
      mint,
      CTOKEN_PROGRAM_ID
    ),
    ...(await createLoadAtaInstructions(rpc, ata, recipient, mint, payer.publicKey))
  );

  const signature = await sendAndConfirmTransaction(rpc, tx, [payer]);

  console.log(`\nhttps://explorer.solana.com/tx/${signature}?cluster=devnet`);
}

main().catch(console.error);
