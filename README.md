# robinhood_tokenized_stocks

Substreams for **Robinhood Chain** (`eip155:4663`) tokenized stocks — the first
package indexing the chain's stock-token data. Streams off Pinax's firehose.

## Why this exists

Robinhood Chain's tokenized stocks (NVDA, AAPL, TSLA, …) are ERC-20s that also emit
**`TransferWithScaledUI(from, to, value, uiValue)`** (ERC-8056, topic0 `0x37e7f0db…`).
Index the `uiValue`, not the naive `Transfer` — naive supply indexing overstates ~56%
(per RWA.xyz). Everything here matches by **topic0**, so no address lists to maintain.

## Modules

| Module | Kind | Output | Notes |
|---|---|---|---|
| `map_scaled_transfers` | map | `ScaledTransfers` | ERC-8056 stock-token transfers with correct `ui_value` |
| `map_oracle_updates` | map | `OracleUpdates` | Chainlink `AnswerUpdated` — on-chain fair price per feed (equities 8-dp) |
| `map_v4_swaps` | map | `Swaps` | all Uniswap V4 `Swap` events off the PoolManager |
| `store_stock_tokens` | store | — | auto-discovered set of stock-token addresses (from the transfers) |
| `store_pools` | store | — | V4 `poolId → currency0,currency1` (from `Initialize`) |
| `map_stock_swaps` | map | `StockSwaps` | V4 swaps **filtered to stock pools** + resolved to tokens, with amounts + decimals |
| `db_out` | map | `DatabaseChanges` | the three stateless feeds as `transfers` / `oracle_updates` / `swaps` rows — ready for a substreams-websocket fan-out or SQL/Parquet sink |

**The oracle-vs-AMM basis** is a one-line downstream join: take the AMM price from
`map_stock_swaps` (`quote_amount/10^quote_decimals ÷ stock_amount/10^stock_decimals`) and
the fair price from `map_oracle_updates`, per stock. The package gives you both sides.

## Run

```bash
export SUBSTREAMS_API_TOKEN=<your Pinax substreams token>

# any stateless map (transfers / oracle / swaps) — runs in any small window, cheap
substreams gui robinhood-tokenized-stocks@v0.4.0 map_scaled_transfers \
  -e robinhood.substreams.pinax.network:443 --start-block <recent>

# db_out — all three feeds as DatabaseChanges, ready for a WS/SQL/Parquet sink
substreams run robinhood-tokenized-stocks@v0.4.0 db_out \
  -e robinhood.substreams.pinax.network:443 --start-block <recent> --stop-block +5000
```

Note: `db_out` and the three stateless maps depend only on per-block decoding — no
backprocess — so they stream a recent window for pennies. Only the store-backed
`map_stock_swaps` backprocesses store state from genesis, so it needs a plan that allows
it (paid tier) or a continuous sink; free tiers cap processed blocks.

## Verification

`map_scaled_transfers`, `map_oracle_updates`, and `map_v4_swaps` were live-run against
Pinax and every field cross-checked against the raw on-chain logs (including signed
int128/int256 for swap amounts and ticks). `db_out` was live-run over a recent 5,000-block
window (2,677 transfers + 299 swaps + 12 oracle rows) and every table was then
**cross-checked against raw on-chain logs via RPC** — each field re-decoded from the
canonical ABI and compared row-by-row (incl. large negative int128 swap amounts and
negative ticks), plus a per-block `eth_getLogs` completeness pass confirming zero dropped
and zero duplicated rows. `map_stock_swaps` reuses those verified decoders plus a standard
store join.

## Roadmap

- multiplier-update handler → adjusted supply
- `store_latest` prices + `map_basis` (market-hours-aware premium) — if AMM stock volume warrants it
- `substreams-sink-files` → parquet / `substreams-sink-sql` → ClickHouse
