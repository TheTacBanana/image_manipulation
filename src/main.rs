#![feature(async_closure)]

/// Entry point for a standalone binary
fn main() {
    pollster::block_on(cs256::run());
}
