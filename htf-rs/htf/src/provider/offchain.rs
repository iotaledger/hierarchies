use std::collections::HashMap;

use iota_sdk::types::base_types::ObjectID;

use crate::client::HTFClientReadOnly;
use crate::types::permission::{PermissionsToAccredit, PermissionsToAttest};
use crate::types::trusted_property::{TrustedPropertyName, TrustedPropertyValue};
use crate::types::Federation;

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
    self.federation = client
      .get_object_by_id(*self.federation.id.object_id())
      .await?;

    Ok(())
  }
}

impl OffChainFederation {
  pub fn federation_id(&self) -> ObjectID {
    *self.federation.id.object_id()
  }

  pub fn has_permission_to_attest(&self, user_id: ObjectID) -> bool {
    self.federation.governance.attesters.contains_key(&user_id)
  }
  pub fn has_permissions_to_accredit(&self, user_id: ObjectID) -> bool {
    self
      .federation
      .governance
      .accreditors
      .contains_key(&user_id)
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
    issuer_id: ObjectID,
    trusted_properties: HashMap<TrustedPropertyName, TrustedPropertyValue>,
  ) -> anyhow::Result<bool> {
    let federation = self.federation();

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

    // then check if names and values are permitted for given issuer
    let issuer_permissions_to_attest = federation
      .governance
      .attesters
      .get(&issuer_id)
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

  pub fn find_permissions_to_attest(&self, user_id: ObjectID) -> Option<PermissionsToAttest> {
    self.federation.governance.attesters.get(&user_id).cloned()
  }

  pub fn find_permissions_to_accredit(&self, user_id: ObjectID) -> Option<PermissionsToAccredit> {
    self.federation.governance.accreditors.get(&user_id).cloned()
  }
}
