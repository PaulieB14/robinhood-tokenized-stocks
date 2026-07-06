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
