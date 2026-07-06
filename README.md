# robinhood_tokenized_stocks

Substreams for **Robinhood Chain** (`eip155:4663`) tokenized stocks — the first
package indexing the chain's ERC-8056 scaled-UI token events.

## Why this exists

Robinhood Chain's tokenized stocks (NVDA, AAPL, TSLA, …) are ERC-20s that also
emit **`TransferWithScaledUI(address indexed from, address indexed to, uint256
value, uint256 uiValue)`** (topic0 `0x37e7f0db430edc9dd31bc66f25f8449353aa0818f503b906747dd8f286cd3802`).
The `uiValue` is the scaled/adjusted amount. **Indexing the naive ERC-20 `Transfer`
overstates supply/balances (~56% per RWA.xyz)** — this package captures `uiValue`
so downstream balances stay correct.

Matching is by **topic0**, so it auto-captures every ERC-8056 token on the chain —
no address list to maintain as Robinhood adds tickers.

## Modules

| Module | Kind | Output |
|---|---|---|
| `map_scaled_transfers` | map | `robinhood.v1.ScaledTransfers` — `{token, from, to, value, ui_value, tx_hash, block_number, block_timestamp, log_index}` per event |

## Run

```bash
export SUBSTREAMS_API_TOKEN=<your Pinax substreams token>
substreams run robinhood-tokenized-stocks-v0.1.0.spkg map_scaled_transfers \
  -e robinhood.substreams.pinax.network:443 \
  --start-block <recent> --stop-block +500
```

Verified live: 5,000 blocks → 114 scaled transfers across 7 stock tokens, all
fields populated.

## Roadmap

- `map_oracle_updates` — Chainlink `AnswerUpdated` (on-chain fair price per stock)
- `map_stock_swaps` — Uniswap V2/V3/V4 `Swap` on stock-token pools (AMM price)
- multiplier-update handler — track the ERC-8056 scaling multiplier for adjusted supply
- `map_basis` — oracle vs AMM premium/discount, market-hours-aware (the derived signal)
