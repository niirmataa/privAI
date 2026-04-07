//! Thin re-export layer for validator session transport internals.
//!
//! This module exists only as a compatibility facade while the validator
//! session transport stays inside `privai-node`.
//! It re-exports concrete implementation types (`ConnectionPool`, `BanList`,
//! `NetConfig`, etc.) from `session_impl.rs`.
//!
//! Architectural rule:
//! - this is **not** the validator consensus overlay
//! - this is **not** the escrow / `NXMS/1` / `NXMS/2` packet protocol
//! - higher-level modules should prefer `ValidatorSessionTransport`
//!   (from `session_transport.rs`) instead of importing these types directly

pub use crate::session_impl::{
    run_listener, BanList, ConnectionMeta, ConnectionPool, ConnectionPoolConfig, HandshakeMsg,
    NetConfig, NetError, PoolStats, RateLimiter,
};
