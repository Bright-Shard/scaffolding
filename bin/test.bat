cargo test --workspace
set RUSTDOCFLAGS="-D warnings"
cargo doc --features std
