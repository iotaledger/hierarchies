use async_trait::async_trait;
use iota_sdk::types::base_types::ObjectID;
use iota_sdk::types::id::ID;

use crate::client::HTFClient;
use crate::federation::FederationOperations;
use crate::htf::Federation;
use crate::types::TrustedPropertyName;

pub struct OffChainFederation {
    federation: Federation,
}

impl OffChainFederation {
    pub async fn new(client: &HTFClient, federation: ObjectID) -> anyhow::Result<Self> {
        let federation = client.get_object_by_id(federation).await?;
        Ok(Self { federation })
    }
}

#[async_trait]
impl FederationOperations for OffChainFederation {
    async fn federation(&self) -> anyhow::Result<Federation> {
        Ok(self.federation.clone())
    }
    async fn validate_credential(&self, credential_id: ID) -> anyhow::Result<()> {
        todo!()
    }
    async fn has_permission_to_attest(&self, user_id: ID) -> anyhow::Result<bool> {
        todo!()
    }
    async fn has_permissions_to_accredit(&self, user_id: ID) -> anyhow::Result<bool> {
        todo!()
    }
    async fn has_federation_property(
        &self,

        property_name: &TrustedPropertyName,
    ) -> anyhow::Result<bool> {
        todo!()
    }
}
