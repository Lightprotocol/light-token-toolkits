# c-Token Institutional Payments Example

Working examples for receiving and sending c-Token payments on devnet.

## Prerequisites

1. Node.js 18+
2. [Helius API key](https://dev.helius.xyz/) (free tier works)
3. Solana CLI with a funded devnet wallet

## Setup

```bash
# Install dependencies
npm install

# Copy environment template and add your Helius API key
cp .env.example .env
# Edit .env and add your HELIUS_API_KEY
```

## Usage

### 1. Receive Payments

Create or get a c-Token ATA to receive payments.

1. Open `src/receive-payments.ts`
2. Set `MINT_PUBKEY` to your token mint address
3. Set `RECIPIENT_PUBKEY` to the wallet that will receive payments
4. Run:

```bash
npm run receive
```

Output: The ATA address to share with senders.

### 2. Send Payments

Transfer c-Tokens to a recipient.

1. Open `src/send-payments.ts`
2. Set `MINT_PUBKEY` to your token mint address
3. Set `RECIPIENT_PUBKEY` to the destination wallet
4. Set `AMOUNT` to the amount to send (in smallest units)
5. Run:

```bash
npm run send
```

Output: Transaction signature with explorer link.

## Configuration

Each script has configuration constants at the top:

| Variable | Description |
|----------|-------------|
| `MINT_PUBKEY` | The token mint address |
| `RECIPIENT_PUBKEY` | Destination wallet address |
| `AMOUNT` | Amount to send (in smallest units) |

The sender's keypair is loaded from `~/.config/solana/id.json` by default.

## Notes

- These examples run on **devnet**
- Make sure your wallet has SOL for transaction fees
- Make sure you have c-Tokens to send (use receive-payments first)
