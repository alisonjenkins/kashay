use anyhow::Result;
use kashay::app::application::run;

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}
