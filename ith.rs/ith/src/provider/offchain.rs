use std::collections::HashMap;

use iota_sdk::types::base_types::ObjectID;

use crate::client::ITHClientReadOnly;
use crate::types::Accreditations;
use crate::types::Federation;
use crate::types::{StatementName, StatementValue};

pub struct OffChainFederation {
  federation: Federation,
}

impl OffChainFederation {
  pub async fn new(client: &ITHClientReadOnly, federation: ObjectID) -> anyhow::Result<Self> {
    let federation = client.get_object_ref_by_id_with_bcs(&federation).await?;
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
  pub async fn sync(&mut self, client: &ITHClientReadOnly) -> anyhow::Result<()> {
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

  pub fn has_accreditation_to_attest(&self, user_id: ObjectID) -> bool {
    self
      .federation
      .governance
      .accreditations_to_attest
      .contains_key(&user_id)
  }
  pub fn is_accreditor(&self, user_id: ObjectID) -> bool {
    self
      .federation
      .governance
      .accreditations_to_accredit
      .contains_key(&user_id)
  }
  pub fn is_statement(&self, statement_name: &StatementName) -> bool {
    let federation = self.federation();

    federation
      .governance
      .statements
      .contains_property(statement_name)
  }

  pub fn validate_statements(
    &self,
    issuer_id: ObjectID,
    statements: impl IntoIterator<Item = (StatementName, StatementValue)>,
  ) -> anyhow::Result<bool> {
    let statements: HashMap<_, _> = statements.into_iter().collect();
    let federation = self.federation();

    // Has federation property
    statements.keys().try_for_each(|statement_name| {
      if !federation
        .governance
        .statements
        .contains_property(statement_name)
      {
        return Err(anyhow::anyhow!("property not found"));
      }
      Ok(())
    })?;

    // then check if names and values are permitted for given issuer
    let issuer_permissions_to_attest = federation
      .governance
      .accreditations_to_attest
      .get(&issuer_id)
      .ok_or_else(|| anyhow::anyhow!("issuer not found"))?;

    let res = issuer_permissions_to_attest.are_statements_allowed(&statements);

    Ok(res)
  }

  pub fn get_statements(&self) -> Vec<StatementName> {
    self
      .federation
      .governance
      .statements
      .data
      .keys()
      .cloned()
      .collect::<Vec<_>>()
  }

  pub fn get_accreditations_to_attest(&self, user_id: ObjectID) -> Option<Accreditations> {
    self
      .federation
      .governance
      .accreditations_to_attest
      .get(&user_id)
      .cloned()
  }

  pub fn get_accreditations_to_accredit(&self, user_id: ObjectID) -> Option<Accreditations> {
    self
      .federation
      .governance
      .accreditations_to_accredit
      .get(&user_id)
      .cloned()
  }
}
