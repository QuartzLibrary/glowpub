#[cfg(not(target_arch = "wasm32"))]
mod cli;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    cli::main().await;
}

#[cfg(target_arch = "wasm32")]
fn main() {
    println!("Not implemented for wasm32");
}
