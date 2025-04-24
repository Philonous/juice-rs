//! Rust bindings for [libjuice](https://github.com/paullouisageneau/libjuice).
//!
//! ## ICE agent usage
//! Please refer to
//! [tests](https://github.com/VollmondT/juice-rs/blob/main/tests/connectivity.rs), also refer to
//! the original library
//! [tests](https://github.com/paullouisageneau/libjuice/blob/master/test/connectivity.c).

pub use agent::{Agent, Builder, State, handler::Handler};
pub use error::{Error, Result};
pub use log::set_log_level;
pub use server::{Builder as ServerBuilder, Credentials as ServerCredentials, Server};

mod agent;
mod error;
mod log;
mod server;

#[cfg(test)]
mod test_util;
