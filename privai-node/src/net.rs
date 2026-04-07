//! Public network facade for validator transport configuration and errors.
//!
//! The concrete validator session implementation lives in `session_impl.rs`.

pub use crate::session_impl::{
    BanList, ConnectionMeta, ConnectionPool, ConnectionPoolConfig, HandshakeMsg, NetConfig,
    NetError, PoolStats, RateLimiter, run_listener,
};
