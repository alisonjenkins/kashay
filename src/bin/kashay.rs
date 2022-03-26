use anyhow::Result;
use kashay::app::application::run;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    run().await
}
