mod pb;

use num_bigint::BigInt as SignedInt;
use pb::robinhood_v1::{
    OracleUpdate, OracleUpdates, ScaledTransfer, ScaledTransfers, Swap, Swaps,
};
use substreams::scalar::BigInt;
use substreams::Hex;
use substreams_ethereum::pb::eth::v2 as eth;

// Event topic0s, all verified against on-chain logs on Robinhood Chain (eip155:4663).
const TRANSFER_WITH_SCALED_UI: [u8; 32] =
    hex_literal::hex!("37e7f0db430edc9dd31bc66f25f8449353aa0818f503b906747dd8f286cd3802");
const ANSWER_UPDATED: [u8; 32] =
    hex_literal::hex!("0559884fd3a460db3073b7fc896cc77986f16e378210ded43186175bf646fc5f");
const V4_SWAP: [u8; 32] =
    hex_literal::hex!("40e9cecb9f5f1f1c5b9c97dec2917b7ee92e57ba5563708daca94dd84ad7112f");

// last 8 bytes of a 32-byte word as big-endian u64 (fine for timestamps / small uints)
fn word_u64(w: &[u8]) -> u64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&w[24..32]);
    u64::from_be_bytes(b)
}

fn block_ts(block: &eth::Block) -> u64 {
    block
        .header
        .as_ref()
        .and_then(|h| h.timestamp.as_ref())
        .map(|t| t.seconds as u64)
        .unwrap_or_default()
}

#[substreams::handlers::map]
fn map_scaled_transfers(block: eth::Block) -> Result<ScaledTransfers, substreams::errors::Error> {
    let block_number = block.number;
    let block_timestamp = block_ts(&block);
    let mut transfers = Vec::new();
    for trace in block.transaction_traces.iter() {
        if trace.status != 1 {
            continue;
        }
        let tx_hash = format!("0x{}", Hex(&trace.hash));
        if let Some(receipt) = trace.receipt.as_ref() {
            for log in receipt.logs.iter() {
                if log.topics.len() == 3
                    && log.topics[0].as_slice() == TRANSFER_WITH_SCALED_UI
                    && log.data.len() >= 64
                {
                    transfers.push(ScaledTransfer {
                        token: format!("0x{}", Hex(&log.address)),
                        from: format!("0x{}", Hex(&log.topics[1][12..32])),
                        to: format!("0x{}", Hex(&log.topics[2][12..32])),
                        value: BigInt::from_unsigned_bytes_be(&log.data[0..32]).to_string(),
                        ui_value: BigInt::from_unsigned_bytes_be(&log.data[32..64]).to_string(),
                        tx_hash: tx_hash.clone(),
                        block_number,
                        block_timestamp,
                        log_index: log.block_index as u64,
                    });
                }
            }
        }
    }
    Ok(ScaledTransfers { transfers })
}

#[substreams::handlers::map]
fn map_oracle_updates(block: eth::Block) -> Result<OracleUpdates, substreams::errors::Error> {
    let block_number = block.number;
    let block_timestamp = block_ts(&block);
    let mut updates = Vec::new();
    for trace in block.transaction_traces.iter() {
        if trace.status != 1 {
            continue;
        }
        let tx_hash = format!("0x{}", Hex(&trace.hash));
        if let Some(receipt) = trace.receipt.as_ref() {
            for log in receipt.logs.iter() {
                // AnswerUpdated(int256 indexed current, uint256 indexed roundId, uint256 updatedAt)
                if log.topics.len() == 3
                    && log.topics[0].as_slice() == ANSWER_UPDATED
                    && log.data.len() >= 32
                {
                    updates.push(OracleUpdate {
                        aggregator: format!("0x{}", Hex(&log.address)),
                        answer: SignedInt::from_signed_bytes_be(&log.topics[1]).to_string(),
                        round_id: BigInt::from_unsigned_bytes_be(&log.topics[2]).to_string(),
                        updated_at: word_u64(&log.data[0..32]),
                        tx_hash: tx_hash.clone(),
                        block_number,
                        block_timestamp,
                        log_index: log.block_index as u64,
                    });
                }
            }
        }
    }
    Ok(OracleUpdates { updates })
}

#[substreams::handlers::map]
fn map_v4_swaps(block: eth::Block) -> Result<Swaps, substreams::errors::Error> {
    let block_number = block.number;
    let block_timestamp = block_ts(&block);
    let mut swaps = Vec::new();
    for trace in block.transaction_traces.iter() {
        if trace.status != 1 {
            continue;
        }
        let tx_hash = format!("0x{}", Hex(&trace.hash));
        if let Some(receipt) = trace.receipt.as_ref() {
            for log in receipt.logs.iter() {
                // Swap(bytes32 indexed id, address indexed sender, int128 amount0,
                //      int128 amount1, uint160 sqrtPriceX96, uint128 liquidity,
                //      int24 tick, uint24 fee)
                if log.topics.len() == 3
                    && log.topics[0].as_slice() == V4_SWAP
                    && log.data.len() >= 192
                {
                    let d = &log.data;
                    swaps.push(Swap {
                        pool_id: format!("0x{}", Hex(&log.topics[1])),
                        sender: format!("0x{}", Hex(&log.topics[2][12..32])),
                        amount0: SignedInt::from_signed_bytes_be(&d[0..32]).to_string(),
                        amount1: SignedInt::from_signed_bytes_be(&d[32..64]).to_string(),
                        sqrt_price_x96: BigInt::from_unsigned_bytes_be(&d[64..96]).to_string(),
                        liquidity: BigInt::from_unsigned_bytes_be(&d[96..128]).to_string(),
                        tick: SignedInt::from_signed_bytes_be(&d[128..160]).to_string(),
                        fee: word_u64(&d[160..192]),
                        tx_hash: tx_hash.clone(),
                        block_number,
                        block_timestamp,
                        log_index: log.block_index as u64,
                    });
                }
            }
        }
    }
    Ok(Swaps { swaps })
}
