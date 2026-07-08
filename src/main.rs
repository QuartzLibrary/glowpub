// The CLI is native-only; on wasm32 the binary is an empty stub so that
// whole-workspace builds (and CI) can target wasm32-unknown-unknown.
// Use the library's `generate` module for wasm.

#[cfg(not(target_arch = "wasm32"))]
mod cli;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    cli::main().await
}

#[cfg(target_arch = "wasm32")]
fn main() {}
