use anyhow::Result;
use kashay::app::application::run;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    run().await
}
