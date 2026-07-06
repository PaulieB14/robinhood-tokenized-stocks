mod pb;

use pb::robinhood_v1::{ScaledTransfer, ScaledTransfers};
use substreams::scalar::BigInt;
use substreams::Hex;
use substreams_ethereum::pb::eth::v2 as eth;

// keccak256("TransferWithScaledUI(address,address,uint256,uint256)")
// Verified against on-chain logs on Robinhood Chain (eip155:4663).
const TRANSFER_WITH_SCALED_UI: [u8; 32] =
    hex_literal::hex!("37e7f0db430edc9dd31bc66f25f8449353aa0818f503b906747dd8f286cd3802");

#[substreams::handlers::map]
fn map_scaled_transfers(block: eth::Block) -> Result<ScaledTransfers, substreams::errors::Error> {
    let block_number = block.number;
    let block_timestamp = block
        .header
        .as_ref()
        .and_then(|h| h.timestamp.as_ref())
        .map(|t| t.seconds as u64)
        .unwrap_or_default();

    let mut transfers = Vec::new();

    for trace in block.transaction_traces.iter() {
        // 1 = SUCCEEDED
        if trace.status != 1 {
            continue;
        }
        let tx_hash = format!("0x{}", Hex(&trace.hash));

        if let Some(receipt) = trace.receipt.as_ref() {
            for log in receipt.logs.iter() {
                // topics: [sig, from, to]; data: value(32) || uiValue(32)
                if log.topics.len() == 3
                    && log.topics[0].as_slice() == TRANSFER_WITH_SCALED_UI
                    && log.data.len() >= 64
                {
                    let from = &log.topics[1][12..32];
                    let to = &log.topics[2][12..32];
                    let value = BigInt::from_unsigned_bytes_be(&log.data[0..32]);
                    let ui_value = BigInt::from_unsigned_bytes_be(&log.data[32..64]);

                    transfers.push(ScaledTransfer {
                        token: format!("0x{}", Hex(&log.address)),
                        from: format!("0x{}", Hex(from)),
                        to: format!("0x{}", Hex(to)),
                        value: value.to_string(),
                        ui_value: ui_value.to_string(),
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
