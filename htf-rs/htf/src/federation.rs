use async_trait::async_trait;
use iota_sdk::types::id::ID;

use crate::htf::Federation;
use crate::types::TrustedPropertyName;

#[async_trait]
pub trait FederationOperations {
    async fn federation(&self) -> anyhow::Result<Federation>;
    async fn validate_credential(&self, credential_id: ID) -> anyhow::Result<()>;
    async fn has_permission_to_attest(&self, user_id: ID) -> anyhow::Result<bool>;
    async fn has_permissions_to_accredit(&self, user_id: ID) -> anyhow::Result<bool>;
    async fn has_federation_property(
        &self,

        property_name: &TrustedPropertyName,
    ) -> anyhow::Result<bool>;
}
