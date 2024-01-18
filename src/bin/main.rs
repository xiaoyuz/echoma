use echoma::{client::Client, Result};

#[tokio::main]
pub async fn main() -> Result<()> {
    let client = Client::new().await?;
    client.start().await
}
