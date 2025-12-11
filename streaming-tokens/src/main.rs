use borsh::BorshDeserialize;
use futures::StreamExt;
use helius_laserstream::solana::storage::confirmed_block::{Message, TransactionStatusMeta};
use helius_laserstream::{subscribe, LaserstreamConfig};
use light_compressed_account::Pubkey;
use light_ctoken_interface::state::extensions::ExtensionStruct;
use light_ctoken_interface::state::mint::CompressedMint;
use light_event::parse::event_from_light_transaction;

const LIGHT_SYSTEM_PROGRAM: &str = "SySTEM1eSU2p4BGQfQpimFEWWSC1XDFeun3Nqzz3rT7";
const CTOKEN_PROGRAM_ID: &str = "cTokenmWW8bLPjZEBAUgYy3zKxQZW6VKi7bqNFEVv3m";
const COMPRESSED_MINT_DISCRIMINATOR: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 1];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let api_key = std::env::var("HELIUS_API_KEY")?;
    let endpoint = "https://laserstream-devnet-ewr.helius-rpc.com".to_string();

    println!("Connecting to Helius Laserstream (devnet)...");
    println!("Filtering for Light System Program: {}", LIGHT_SYSTEM_PROGRAM);

    let config = LaserstreamConfig::new(endpoint, api_key);

    let request = helius_laserstream::grpc::SubscribeRequest {
        transactions: [(
            "light".to_string(),
            helius_laserstream::grpc::SubscribeRequestFilterTransactions {
                vote: Some(false),
                failed: Some(false),
                account_include: vec![LIGHT_SYSTEM_PROGRAM.to_string()],
                ..Default::default()
            },
        )]
        .into(),
        ..Default::default()
    };

    let (stream, _handle) = subscribe(config, request);
    tokio::pin!(stream);

    println!("Connected! Waiting for Light Protocol transactions...\n");

    while let Some(update) = stream.next().await {
        match update {
            Ok(msg) => {
                if let Some(update_oneof) = msg.update_oneof {
                    match update_oneof {
                        helius_laserstream::grpc::subscribe_update::UpdateOneof::Transaction(tx_info) => {
                            if let Some(tx_wrapper) = &tx_info.transaction {
                                // Get signature from the transaction wrapper
                                let sig = bs58::encode(&tx_wrapper.signature).into_string();
                                println!("Transaction: {} {}", tx_info.slot, sig);

                                if let Some(tx) = &tx_wrapper.transaction {
                                    if let Some(msg) = &tx.message {
                                        process_transaction(msg, tx_wrapper.meta.as_ref());
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                eprintln!("Stream error: {:?}", e);
            }
        }
    }

    Ok(())
}

fn process_transaction(msg: &Message, meta: Option<&TransactionStatusMeta>) {
    // Extract account keys (including loaded addresses from meta)
    let mut account_keys: Vec<Pubkey> = msg
        .account_keys
        .iter()
        .filter_map(|k| {
            if k.len() == 32 {
                let arr: [u8; 32] = k.as_slice().try_into().ok()?;
                Some(Pubkey::from(arr))
            } else {
                None
            }
        })
        .collect();

    // Add loaded addresses from meta (for address lookup tables)
    if let Some(meta) = meta {
        for addr in &meta.loaded_writable_addresses {
            if addr.len() == 32 {
                if let Ok(arr) = <[u8; 32]>::try_from(addr.as_slice()) {
                    account_keys.push(Pubkey::from(arr));
                }
            }
        }
        for addr in &meta.loaded_readonly_addresses {
            if addr.len() == 32 {
                if let Ok(arr) = <[u8; 32]>::try_from(addr.as_slice()) {
                    account_keys.push(Pubkey::from(arr));
                }
            }
        }
    }

    // Extract ALL instructions (top-level + inner)
    let mut program_ids = Vec::new();
    let mut instruction_data = Vec::new();
    let mut accounts_per_ix = Vec::new();

    // Process top-level instructions
    for ix in &msg.instructions {
        let program_id_idx = ix.program_id_index as usize;
        if program_id_idx < account_keys.len() {
            program_ids.push(account_keys[program_id_idx]);
            instruction_data.push(ix.data.clone());
            let ix_accounts: Vec<Pubkey> = ix
                .accounts
                .iter()
                .filter_map(|&idx| account_keys.get(idx as usize).copied())
                .collect();
            accounts_per_ix.push(ix_accounts);
        }
    }

    // Process inner instructions (CPIs) - this is where Light events are!
    if let Some(meta) = meta {
        for inner_ixs in &meta.inner_instructions {
            for inner_ix in &inner_ixs.instructions {
                let program_id_idx = inner_ix.program_id_index as usize;
                if program_id_idx < account_keys.len() {
                    program_ids.push(account_keys[program_id_idx]);
                    instruction_data.push(inner_ix.data.clone());
                    let ix_accounts: Vec<Pubkey> = inner_ix
                        .accounts
                        .iter()
                        .filter_map(|&idx| account_keys.get(idx as usize).copied())
                        .collect();
                    accounts_per_ix.push(ix_accounts);
                }
            }
        }
    }

    // Parse Light Protocol events
    match event_from_light_transaction(&program_ids, &instruction_data, accounts_per_ix) {
        Ok(Some(batches)) => {
            for batch in batches {
                let event = &batch.event;
                println!(
                    "  Parsed: {} inputs, {} outputs",
                    event.input_compressed_account_hashes.len(),
                    event.output_compressed_accounts.len()
                );

                // Check for compressed mints
                let ctoken_pubkey = bs58::decode(CTOKEN_PROGRAM_ID)
                    .into_vec()
                    .ok()
                    .and_then(|v| <[u8; 32]>::try_from(v).ok())
                    .map(Pubkey::from);

                for output in event.output_compressed_accounts.iter() {
                    let owner = output.compressed_account.owner;
                    let owner_str = bs58::encode(owner.to_bytes()).into_string();

                    // Check if this is a cToken account
                    if ctoken_pubkey.map(|p| p == owner).unwrap_or(false) {
                        if let Some(data) = &output.compressed_account.data {
                            // Check discriminator for CompressedMint
                            if data.discriminator == COMPRESSED_MINT_DISCRIMINATOR {
                                // Deserialize the mint
                                // Get address if available
                                let address_str = output
                                    .compressed_account
                                    .address
                                    .map(|a| bs58::encode(a).into_string())
                                    .unwrap_or_else(|| "none".to_string());

                                match CompressedMint::try_from_slice(&data.data) {
                                    Ok(mint) => {
                                        let mint_pubkey = bs58::encode(mint.metadata.mint.to_bytes()).into_string();
                                        println!("    Mint Address: {}", address_str);
                                        println!("      Supply: {}", mint.base.supply);
                                        println!("      Decimals: {}", mint.base.decimals);

                                        // Extract TokenMetadata extension
                                        if let Some(extensions) = &mint.extensions {
                                            for ext in extensions {
                                                if let ExtensionStruct::TokenMetadata(m) = ext {
                                                    let name = String::from_utf8_lossy(&m.name);
                                                    let symbol = String::from_utf8_lossy(&m.symbol);
                                                    let uri = String::from_utf8_lossy(&m.uri);
                                                    println!("      Name: {}", name);
                                                    println!("      Symbol: {}", symbol);
                                                    println!("      URI: {}", uri);
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        println!("    Mint deserialize error: {:?}", e);
                                    }
                                }
                            } else {
                                println!("    cToken output: owner={}, discriminator={:?}", owner_str, data.discriminator);
                            }
                        }
                    } else {
                        println!("    Output: owner={}", owner_str);
                    }
                }

                // Print new addresses if any
                if !batch.new_addresses.is_empty() {
                    println!("    New addresses: {}", batch.new_addresses.len());
                }
            }
        }
        Ok(None) => {
            println!("  No compressed account events");
        }
        Err(e) => {
            println!("  Parse error: {:?}", e);
        }
    }
}
