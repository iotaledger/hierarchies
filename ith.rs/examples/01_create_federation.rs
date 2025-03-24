use examples::{get_client, urls};

/// Demonstrates how to create a Federation and publish it on chain.
///
/// In this example we connect to a locally running private network, but it can be adapted
/// to run on any IOTA node by setting the network and faucet endpoints.
///
/// See the following instructions on running your own private network
/// https://github.com/iotaledger/iota/blob/develop/docs/content/developer/getting-started/connect.mdx

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let ith_client = get_client(urls::localnet::node(), urls::localnet::faucet()).await?;

  println!("Creating new federation");
  let federation = ith_client.new_federation(None).await?;

  println!("Federation created: {:#?}", federation);

  Ok(())
}
