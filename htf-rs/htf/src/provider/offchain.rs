use iota_sdk::types::base_types::ObjectID;
use iota_sdk::types::collection_types::VecMap;
use iota_sdk::types::id::ID;

use crate::client::HTFClientReadOnly;
use crate::types::trusted_property::{TrustedPropertyName, TrustedPropertyValue};
use crate::types::Federation;
use crate::utils::{Hashable, IntoCollectionHash};

pub struct OffChainFederation {
  federation: Federation,
}

impl OffChainFederation {
  pub async fn new(client: &HTFClientReadOnly, federation: ObjectID) -> anyhow::Result<Self> {
    let federation = client.get_object_by_id(federation).await?;
    Ok(Self { federation })
  }

  pub fn federation(&self) -> &Federation {
    &self.federation
  }

  /// Syncs the off-chain federation with the on-chain federation.
  ///
  /// It is recommended to call this method before performing any operations
  /// on the off-chain federation, to ensure that the off-chain federation is
  /// up-to-date with the on-chain federation.
  ///
  /// This function can be scheduled to run periodically to keep the off-chain
  /// federation in sync with the on-chain federation.
  pub async fn sync(&mut self, client: &HTFClientReadOnly) -> anyhow::Result<()> {
    self.federation = client.get_object_by_id(*self.federation.id.object_id()).await?;

    Ok(())
  }
}

impl OffChainFederation {
  pub fn federation_id(&self) -> ObjectID {
    *self.federation.id.object_id()
  }

  pub fn has_permission_to_attest(&self, user_id: ID) -> bool {
    self.federation.governance.attesters.contains_key(&Hashable(user_id))
  }
  pub fn has_permissions_to_accredit(&self, user_id: ID) -> bool {
    self.federation.governance.accreditors.contains_key(&Hashable(user_id))
  }
  pub fn has_federation_property(&self, property_name: &TrustedPropertyName) -> bool {
    let federation = self.federation();

    federation
      .governance
      .trusted_constraints
      .contains_property(property_name)
  }

  pub fn validate_trusted_properties(
    &self,
    issuer_id: ID,
    trusted_properties: VecMap<TrustedPropertyName, TrustedPropertyValue>,
  ) -> anyhow::Result<bool> {
    let federation = self.federation();
    let trusted_properties = trusted_properties.to_hashmap();

    // Has federation property
    trusted_properties.keys().try_for_each(|property_name| {
      if !federation
        .governance
        .trusted_constraints
        .contains_property(property_name)
      {
        return Err(anyhow::anyhow!("property not found"));
      }
      Ok(())
    })?;

    // then check if names and values are permitted for given issuser
    let issuer_permissions_to_attest = federation
      .governance
      .attesters
      .get(&Hashable(issuer_id))
      .ok_or_else(|| anyhow::anyhow!("issuer not found"))?;

    let res = issuer_permissions_to_attest.are_values_permitted(&trusted_properties);

    Ok(res)
  }

  pub fn get_federation_properties(&self) -> Vec<TrustedPropertyName> {
    self
      .federation
      .governance
      .trusted_constraints
      .data
      .keys()
      .cloned()
      .collect::<Vec<_>>()
  }
}
