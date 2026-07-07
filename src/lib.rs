mod pb;

use num_bigint::BigInt as SignedInt;
use pb::robinhood_v1::{
    OracleUpdate, OracleUpdates, ScaledTransfer, ScaledTransfers, StockSwap, StockSwaps, Swap, Swaps,
};
use substreams::scalar::BigInt;
use substreams::store::{
    StoreGet, StoreGetInt64, StoreGetString, StoreNew, StoreSet, StoreSetInt64, StoreSetString,
};
use substreams::Hex;
use substreams_database_change::pb::database::DatabaseChanges;
use substreams_database_change::tables::Tables;
use substreams_ethereum::pb::eth::v2 as eth;

// Event topic0s, all verified against on-chain logs on Robinhood Chain (eip155:4663).
const TRANSFER_WITH_SCALED_UI: [u8; 32] =
    hex_literal::hex!("37e7f0db430edc9dd31bc66f25f8449353aa0818f503b906747dd8f286cd3802");
const ANSWER_UPDATED: [u8; 32] =
    hex_literal::hex!("0559884fd3a460db3073b7fc896cc77986f16e378210ded43186175bf646fc5f");
const V4_SWAP: [u8; 32] =
    hex_literal::hex!("40e9cecb9f5f1f1c5b9c97dec2917b7ee92e57ba5563708daca94dd84ad7112f");
const INITIALIZE: [u8; 32] =
    hex_literal::hex!("dd466e674ea557f56295e2d0218a125ea4b4f0f6f3307b95f85e6110838d6438");

// USDG (Global Dollar), the real Robinhood Chain stable, 6 decimals.
const USDG: &str = "0x5fc5360d0400a0fd4f2af552add042d716f1d168";

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

// ── stores (v0.3.0) ──────────────────────────────────────────────────────────

// Every address that emits TransferWithScaledUI is a Robinhood stock token.
// Auto-discovered — no hardcoded ticker list.
#[substreams::handlers::store]
fn store_stock_tokens(transfers: ScaledTransfers, store: StoreSetInt64) {
    for t in transfers.transfers.iter() {
        store.set(0, &t.token, &1i64);
    }
}

// V4 poolId -> "currency0,currency1" from Initialize events.
#[substreams::handlers::store]
fn store_pools(block: eth::Block, store: StoreSetString) {
    for trace in block.transaction_traces.iter() {
        if trace.status != 1 {
            continue;
        }
        if let Some(receipt) = trace.receipt.as_ref() {
            for log in receipt.logs.iter() {
                if log.topics.len() == 4 && log.topics[0].as_slice() == INITIALIZE {
                    let pool = format!("0x{}", Hex(&log.topics[1]));
                    let c0 = format!("0x{}", Hex(&log.topics[2][12..32]));
                    let c1 = format!("0x{}", Hex(&log.topics[3][12..32]));
                    store.set(log.ordinal, &pool, &format!("{},{}", c0, c1));
                }
            }
        }
    }
}

// V4 swaps filtered to stock pools + resolved to their tokens.
#[substreams::handlers::map]
fn map_stock_swaps(
    block: eth::Block,
    stock_tokens: StoreGetInt64,
    pools: StoreGetString,
) -> Result<StockSwaps, substreams::errors::Error> {
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
                if log.topics.len() != 3
                    || log.topics[0].as_slice() != V4_SWAP
                    || log.data.len() < 64
                {
                    continue;
                }
                let pool = format!("0x{}", Hex(&log.topics[1]));
                let meta = match pools.get_last(&pool) {
                    Some(m) => m,
                    None => continue,
                };
                let mut parts = meta.split(',');
                let c0 = parts.next().unwrap_or("").to_string();
                let c1 = parts.next().unwrap_or("").to_string();
                let c0_stock = stock_tokens.get_last(&c0).is_some();
                let c1_stock = stock_tokens.get_last(&c1).is_some();
                if !c0_stock && !c1_stock {
                    continue;
                }
                let d = &log.data;
                let a0 = SignedInt::from_signed_bytes_be(&d[0..32])
                    .to_string()
                    .trim_start_matches('-')
                    .to_string();
                let a1 = SignedInt::from_signed_bytes_be(&d[32..64])
                    .to_string()
                    .trim_start_matches('-')
                    .to_string();
                let (stock_token, quote_token, stock_amount, quote_amount) = if c0_stock {
                    (c0, c1, a0, a1)
                } else {
                    (c1, c0, a1, a0)
                };
                let usdg_quote = quote_token.eq_ignore_ascii_case(USDG);
                swaps.push(StockSwap {
                    pool_id: pool,
                    stock_token,
                    quote_token,
                    stock_amount,
                    quote_amount,
                    stock_decimals: 18,
                    quote_decimals: if usdg_quote { 6 } else { 18 },
                    usdg_quote,
                    tx_hash: tx_hash.clone(),
                    block_number,
                    block_timestamp,
                    log_index: log.block_index as u64,
                });
            }
        }
    }
    Ok(StockSwaps { swaps })
}

// ── db_out (v0.4.0) ──────────────────────────────────────────────────────────
// Emit the three stateless feeds as sf.substreams.sink.database.v1.DatabaseChanges
// (tables: transfers / oracle_updates / swaps) so a substreams-websocket server or
// a SQL sink can fan them out. Depends only on stateless maps -> no backprocess.
#[substreams::handlers::map]
fn db_out(
    transfers: ScaledTransfers,
    oracle: OracleUpdates,
    swaps: Swaps,
) -> Result<DatabaseChanges, substreams::errors::Error> {
    let mut tables = Tables::new();

    for t in transfers.transfers {
        let pk = format!("{}-{}", t.tx_hash, t.log_index);
        tables
            .create_row("transfers", pk)
            .set("token", t.token)
            .set("from", t.from)
            .set("to", t.to)
            .set("value", t.value)
            .set("ui_value", t.ui_value)
            .set("tx_hash", t.tx_hash)
            .set("block_num", t.block_number.to_string())
            .set("timestamp", t.block_timestamp.to_string())
            .set("log_index", t.log_index.to_string());
    }

    for o in oracle.updates {
        let pk = format!("{}-{}", o.tx_hash, o.log_index);
        tables
            .create_row("oracle_updates", pk)
            .set("aggregator", o.aggregator)
            .set("answer", o.answer)
            .set("round_id", o.round_id)
            .set("updated_at", o.updated_at.to_string())
            .set("tx_hash", o.tx_hash)
            .set("block_num", o.block_number.to_string())
            .set("timestamp", o.block_timestamp.to_string())
            .set("log_index", o.log_index.to_string());
    }

    for s in swaps.swaps {
        let pk = format!("{}-{}", s.tx_hash, s.log_index);
        tables
            .create_row("swaps", pk)
            .set("pool_id", s.pool_id)
            .set("sender", s.sender)
            .set("amount0", s.amount0)
            .set("amount1", s.amount1)
            .set("sqrt_price_x96", s.sqrt_price_x96)
            .set("liquidity", s.liquidity)
            .set("tick", s.tick)
            .set("fee", s.fee.to_string())
            .set("tx_hash", s.tx_hash)
            .set("block_num", s.block_number.to_string())
            .set("timestamp", s.block_timestamp.to_string())
            .set("log_index", s.log_index.to_string());
    }

    Ok(tables.to_database_changes())
}

// ── erc8056_transfers (v0.5.0) ────────────────────────────────────────────────
// A DatabaseChanges table that mirrors the column layout of Pinax's canonical
// erc20_transfers (pinax-network/substreams-evm) so ws.pinax.network can host it
// directly as `erc8056_transfers`. Same block/tx/log context columns + event
// from/to/amount, PLUS the ERC-8056 `ui_amount` (scaled-UI value) — the reason
// this token standard exists. Composite PK matches Pinax: (minute, timestamp,
// block_num, tx_index, log_index, block_hash).
fn hex0x(b: &[u8]) -> String {
    format!("0x{}", Hex(b))
}

fn bigint_dec(bi: Option<&eth::BigInt>) -> String {
    bi.map(|b| BigInt::from_unsigned_bytes_be(&b.bytes).to_string())
        .unwrap_or_default()
}

#[substreams::handlers::map]
fn erc8056_transfers(block: eth::Block) -> Result<DatabaseChanges, substreams::errors::Error> {
    let mut tables = Tables::new();
    let block_num = block.number;
    let block_hash = hex0x(&block.hash);
    let ts = block_ts(&block) as i64;
    let minute = ts / 60;
    let mut any = false;

    for trace in block.transaction_traces.iter() {
        if trace.status != 1 {
            continue;
        }
        let receipt = match trace.receipt.as_ref() {
            Some(r) => r,
            None => continue,
        };
        for (log_index, log) in receipt.logs.iter().enumerate() {
            if log.topics.len() != 3
                || log.topics[0].as_slice() != TRANSFER_WITH_SCALED_UI
                || log.data.len() < 64
            {
                continue;
            }
            any = true;
            let key = [
                ("minute", minute.to_string()),
                ("timestamp", ts.to_string()),
                ("block_num", block_num.to_string()),
                ("tx_index", trace.index.to_string()),
                ("log_index", log_index.to_string()),
                ("block_hash", block_hash.clone()),
            ];
            let row = tables.create_row("erc8056_transfers", key);
            // clock
            row.set("block_num", block_num);
            row.set("block_hash", &block_hash);
            row.set("timestamp", ts);
            row.set("minute", minute);
            // log (Transactions -> Log -> Data)
            row.set("log_index", log_index as u32);
            row.set("log_block_index", log.block_index);
            row.set("log_address", hex0x(&log.address));
            row.set("log_ordinal", log.ordinal);
            row.set(
                "log_topics",
                log.topics.iter().map(|t| hex0x(t)).collect::<Vec<_>>().join(","),
            );
            row.set("log_data", hex0x(&log.data));
            // transaction
            row.set("tx_index", trace.index);
            row.set("tx_hash", hex0x(&trace.hash));
            row.set("tx_from", hex0x(&trace.from));
            row.set("tx_to", hex0x(&trace.to));
            row.set("tx_nonce", trace.nonce);
            row.set("tx_gas_price", bigint_dec(trace.gas_price.as_ref()));
            row.set("tx_gas_limit", trace.gas_limit);
            row.set("tx_gas_used", trace.gas_used);
            row.set("tx_value", bigint_dec(trace.value.as_ref()));
            // event: ERC-8056 TransferWithScaledUI(from, to, value, uiValue)
            row.set("from", hex0x(&log.topics[1][12..32]));
            row.set("to", hex0x(&log.topics[2][12..32]));
            row.set("amount", BigInt::from_unsigned_bytes_be(&log.data[0..32]).to_string());
            row.set("ui_amount", BigInt::from_unsigned_bytes_be(&log.data[32..64]).to_string());
        }
    }

    // blocks table — only when the block carried ≥1 transfer (mirrors Pinax)
    if any {
        let brow = tables.create_row("blocks", [("block_num", block_num.to_string())]);
        brow.set("block_num", block_num);
        brow.set("block_hash", &block_hash);
        brow.set("timestamp", ts);
        brow.set("minute", minute);
    }

    Ok(tables.to_database_changes())
}
