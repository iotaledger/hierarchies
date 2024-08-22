use anyhow::Context;
use fastcrypto::hash::HashFunction;
use fastcrypto::traits::ToFromBytes;

use iota_sdk::rpc_types::{
    IotaData, IotaObjectDataOptions, IotaTransactionBlockResponse,
    IotaTransactionBlockResponseOptions,
};
use iota_sdk::types::base_types::{IotaAddress, ObjectID};
use iota_sdk::types::crypto::{DefaultHash, Signature, SignatureScheme};
use iota_sdk::types::quorum_driver_types::ExecuteTransactionRequestType;
use iota_sdk::types::transaction::{ProgrammableTransaction, Transaction, TransactionData};

use iota_sdk::{IotaClient, IotaClientBuilder};

use secret_storage::Signer;
use shared_crypto::intent::{Intent, IntentMessage};
use std::ops::Deref;

use crate::htf::Federation;
use crate::key::{IotaKeySignature, SigningInfo};
use crate::utils::convert_to_address;

// Builder for HTFClient
#[derive(Default)]
pub struct HTFClientBuilder {
    pub(crate) iota_client: Option<IotaClient>,
    pub(crate) sender_pk: Option<Vec<u8>>,
    pub(crate) htf_package_id: Option<ObjectID>,
    pub(crate) signer: Option<Box<dyn Signer<IotaKeySignature, KeyId = ()>>>, // TODO:: use a better type here
    pub(crate) gas_budget: Option<u64>,
}

impl HTFClientBuilder {
    #[must_use]
    pub fn htf_package_id(mut self, htf_package_id: ObjectID) -> Self {
        self.htf_package_id = Some(htf_package_id);
        self
    }

    #[must_use]
    pub fn iota_client(mut self, iota_client: IotaClient) -> Self {
        self.iota_client = Some(iota_client);
        self
    }
    /// Sets the `sender_public_key` value.
    #[must_use]
    pub fn sender_public_key(mut self, value: &[u8]) -> Self {
        self.sender_pk = Some(value.into());
        self
    }

    #[must_use]
    pub fn signer(mut self, signer: Box<dyn Signer<IotaKeySignature, KeyId = ()>>) -> Self {
        self.signer = Some(signer);
        self
    }

    #[must_use]
    pub fn gas_budget(mut self, gas_budget: u64) -> Self {
        self.gas_budget = Some(gas_budget);
        self
    }

    /// Consumes the builder and returns a new `HTFClient` instance.
    pub fn build(self) -> anyhow::Result<HTFClient> {
        HTFClient::from_builder(self)
    }
}

/// The `HTFClient` struct is responsible for managing the connection to the
/// IOTA network and
/// executing transactions on behalf of the HTF (Hierarchial Trust Framework) package.
pub struct HTFClient {
    /// This is the package ID of the HTF package.
    htf_package_id: ObjectID,
    /// This is the client used to connect to the IOTA network.
    iota_client: IotaClient,
    signing_info: Option<SigningInfo>,
    signer: Box<dyn Signer<IotaKeySignature, KeyId = ()>>,
    gas_budget: u64,
}

impl HTFClient {
    /// Creates a new `HTFClient` instance with the given IOTA network URL and
    /// the HTF package ID.
    ///
    /// This function initializes an `IotaClient` instance using the provided
    /// URL and returns a new `HTFClient`
    /// instance with the given HTF package ID.
    ///
    /// # Arguments
    /// * `url` - The URL of the IOTA network to connect to.
    /// * `package_id` - The package ID of the HTF package.
    ///
    /// # Returns
    /// A new `HTFClient` instance, or an error if the `IotaClient` could not
    /// be created.
    pub async fn new(
        url: &str,
        package_id: ObjectID,
        pub_key: Vec<u8>,
        signer: Box<dyn Signer<IotaKeySignature, KeyId = ()>>,
        gas_budget: u64,
    ) -> anyhow::Result<Self> {
        let client = IotaClientBuilder::default().build(url).await?;

        let address = convert_to_address(&pub_key)?;

        let signing_info = SigningInfo {
            sender_public_key: pub_key,
            sender_address: address,
        };

        Ok(Self {
            iota_client: client,
            htf_package_id: package_id,
            signing_info: Some(signing_info),
            signer,
            gas_budget,
        })
    }

    pub fn sender_address(&self) -> IotaAddress {
        self.signing_info
            .as_ref()
            .expect("signing info should be set")
            .sender_address
    }

    pub fn sender_public_key(&self) -> anyhow::Result<&[u8]> {
        self.signing_info
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("signing info should be set"))
            .map(|v| v.sender_public_key.as_ref())
    }

    pub fn htf_package_id(&self) -> ObjectID {
        self.htf_package_id
    }

    pub(crate) async fn execute_transaction(
        &self,
        tx: ProgrammableTransaction,
    ) -> anyhow::Result<IotaTransactionBlockResponse> {
        let tx_data = self.get_transaction_data(tx).await?;
        let kinesis_signature = self.sign_transaction_data(&tx_data).await?;

        // execute tx
        self.quorum_driver_api()
            .execute_transaction_block(
                Transaction::from_data(tx_data, vec![kinesis_signature]),
                IotaTransactionBlockResponseOptions::full_content(),
                Some(ExecuteTransactionRequestType::WaitForLocalExecution),
            )
            .await
            .map_err(anyhow::Error::from)
    }

    async fn get_transaction_data(
        &self,
        programmable_transaction: ProgrammableTransaction,
    ) -> anyhow::Result<TransactionData> {
        let gas_price = self
            .iota_client
            .read_api()
            .get_reference_gas_price()
            .await?;

        let sender = self
            .signing_info
            .as_ref()
            .expect("signing info should be set")
            .sender_address;

        let coin = self.get_coin_for_transaction().await?;
        let tx_data = TransactionData::new_programmable(
            sender,
            vec![coin.object_ref()],
            programmable_transaction,
            self.gas_budget,
            gas_price,
        );

        Ok(tx_data)
    }

    async fn get_coin_for_transaction(&self) -> anyhow::Result<iota_sdk::rpc_types::Coin> {
        let address = self
            .signing_info
            .as_ref()
            .expect("signing info should be set")
            .sender_address;

        let coins = self
            .iota_client
            .coin_read_api()
            .get_coins(address, None, None, None)
            .await?;

        coins
            .data
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No coins found for sender address"))
    }

    async fn sign_transaction_data(&self, tx_data: &TransactionData) -> anyhow::Result<Signature> {
        let SigningInfo {
            sender_public_key, ..
        } = self
            .signing_info
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No signing info found for sender address"))?;

        let intent = Intent::iota_transaction();
        let intent_msg = IntentMessage::new(intent, tx_data);
        let mut hasher = DefaultHash::default();
        hasher.update(bcs::to_bytes(&intent_msg)?);
        let digest = hasher.finalize().digest;

        let raw_signature = self.signer.sign(&digest).await?;

        let binding = [
            [SignatureScheme::ED25519.flag()].as_slice(),
            &raw_signature,
            sender_public_key,
        ]
        .concat();

        let signature_bytes: &[u8] = binding.as_slice();

        Signature::from_bytes(signature_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to create signature: {}", e))
    }

    pub fn from_builder(builder: HTFClientBuilder) -> anyhow::Result<Self> {
        let iota_client = builder
            .iota_client
            .ok_or_else(|| anyhow::anyhow!("IOTA client not set"))?;

        let Some(pk) = builder.sender_pk else {
            return Err(anyhow::anyhow!("Sender public key not set"));
        };

        let sender_address = convert_to_address(&pk)?;

        let signing_info = SigningInfo {
            sender_address,
            sender_public_key: pk,
        };

        let htf_package_id = builder
            .htf_package_id
            .ok_or_else(|| anyhow::anyhow!("HTF package ID not set"))?;

        let signer = builder
            .signer
            .ok_or_else(|| anyhow::anyhow!("Signer not set"))?;

        let gas_budget = builder
            .gas_budget
            .ok_or_else(|| anyhow::anyhow!("Gas budget not set"))?;

        Ok(Self {
            iota_client,
            signing_info: Some(signing_info),
            htf_package_id,
            signer,
            gas_budget,
        })
    }
}

impl HTFClient {
    pub async fn new_federation(&mut self) -> anyhow::Result<Federation> {
        let federation = Federation::create_new_federation(self).await?;

        Ok(federation)
    }

    pub async fn get_object_by_id<R>(&self, id: ObjectID) -> anyhow::Result<R>
    where
        R: serde::de::DeserializeOwned,
    {
        let res = self
            .iota_client
            .read_api()
            .get_object_with_options(id, IotaObjectDataOptions::new().with_content())
            .await?;

        let Some(data) = res.data else {
            return Err(anyhow::anyhow!("no data"));
        };

        let data = data
            .content
            .ok_or_else(|| anyhow::anyhow!("missing content"))
            .and_then(|content| content.try_into_move().context("invalid content"))
            .and_then(|data| {
                serde_json::from_value(data.fields.to_json_value()).context("invalid data")
            })?;

        Ok(data)
    }
}

impl Deref for HTFClient {
    type Target = IotaClient;

    fn deref(&self) -> &Self::Target {
        &self.iota_client
    }
}
