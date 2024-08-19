use examples::get_client;
use htf::htf::Federation;

/// Demonstrates how to create a a Federation and publish it on chain.
///
/// In this example we connect to a locally running private network, but it can be adapted
/// to run on any IOTA node by setting the network and faucet endpoints.
///
/// See the following instructions on running your own private network
/// https://github.com/iotaledger/hornet/tree/develop/private_tangle

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let htf_client = get_client().await?;

    println!("Creating new federation");
    let federation = Federation::create_new_federation(&htf_client).await?;

    println!("Federation created: {:#?}", federation);

    Ok(())
}
