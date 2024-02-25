#![feature(async_closure)]

fn main() {
    pollster::block_on(cs256::run());
}
