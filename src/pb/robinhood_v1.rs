// Hand-written prost bindings for proto/robinhood.proto (package robinhood.v1).

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ScaledTransfers {
    #[prost(message, repeated, tag = "1")]
    pub transfers: ::prost::alloc::vec::Vec<ScaledTransfer>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ScaledTransfer {
    #[prost(string, tag = "1")]
    pub token: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub from: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub to: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub value: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub ui_value: ::prost::alloc::string::String,
    #[prost(string, tag = "6")]
    pub tx_hash: ::prost::alloc::string::String,
    #[prost(uint64, tag = "7")]
    pub block_number: u64,
    #[prost(uint64, tag = "8")]
    pub block_timestamp: u64,
    #[prost(uint64, tag = "9")]
    pub log_index: u64,
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OracleUpdates {
    #[prost(message, repeated, tag = "1")]
    pub updates: ::prost::alloc::vec::Vec<OracleUpdate>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OracleUpdate {
    #[prost(string, tag = "1")]
    pub aggregator: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub answer: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub round_id: ::prost::alloc::string::String,
    #[prost(uint64, tag = "4")]
    pub updated_at: u64,
    #[prost(string, tag = "5")]
    pub tx_hash: ::prost::alloc::string::String,
    #[prost(uint64, tag = "6")]
    pub block_number: u64,
    #[prost(uint64, tag = "7")]
    pub block_timestamp: u64,
    #[prost(uint64, tag = "8")]
    pub log_index: u64,
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Swaps {
    #[prost(message, repeated, tag = "1")]
    pub swaps: ::prost::alloc::vec::Vec<Swap>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Swap {
    #[prost(string, tag = "1")]
    pub pool_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub sender: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub amount0: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub amount1: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub sqrt_price_x96: ::prost::alloc::string::String,
    #[prost(string, tag = "6")]
    pub liquidity: ::prost::alloc::string::String,
    #[prost(string, tag = "7")]
    pub tick: ::prost::alloc::string::String,
    #[prost(uint64, tag = "8")]
    pub fee: u64,
    #[prost(string, tag = "9")]
    pub tx_hash: ::prost::alloc::string::String,
    #[prost(uint64, tag = "10")]
    pub block_number: u64,
    #[prost(uint64, tag = "11")]
    pub block_timestamp: u64,
    #[prost(uint64, tag = "12")]
    pub log_index: u64,
}
