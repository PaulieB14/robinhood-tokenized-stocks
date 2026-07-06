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

**The oracle-vs-AMM basis** is a one-line downstream join: take the AMM price from
`map_stock_swaps` (`quote_amount/10^quote_decimals ÷ stock_amount/10^stock_decimals`) and
the fair price from `map_oracle_updates`, per stock. The package gives you both sides.

## Run

```bash
export SUBSTREAMS_API_TOKEN=<your Pinax substreams token>
substreams gui robinhood-tokenized-stocks@v0.3.0 map_scaled_transfers \
  -e robinhood.substreams.pinax.network:443 --start-block <recent>
```

Note: the store-backed `map_stock_swaps` backprocesses store state from genesis, so it
needs a plan that allows it (paid tier) or a continuous sink; free tiers cap processed
blocks. The three stateless maps run in any small window.

## Verification

`map_scaled_transfers`, `map_oracle_updates`, and `map_v4_swaps` were live-run against
Pinax and every field cross-checked against the raw on-chain logs (including signed
int128/int256 for swap amounts and ticks). `map_stock_swaps` reuses those verified
decoders plus a standard store join.

## Roadmap

- multiplier-update handler → adjusted supply
- `store_latest` prices + `map_basis` (market-hours-aware premium) — if AMM stock volume warrants it
- `substreams-sink-files` → parquet / `substreams-sink-sql` → ClickHouse
