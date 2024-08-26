use examples::get_client;

/// Demonstrates how to create a Federation and publish it on chain.
///
/// In this example we connect to a locally running private network, but it can be adapted
/// to run on any IOTA node by setting the network and faucet endpoints.
///
/// See the following instructions on running your own private network
/// https://github.com/iotaledger/iota-test-validator

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let htf_client = get_client().await?;

  println!("Creating new federation");
  let federation = htf_client.new_federation().await?;

  println!("Federation created: {:#?}", federation);

  Ok(())
}
