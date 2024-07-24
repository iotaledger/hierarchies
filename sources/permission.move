// HTF Notary module
module htf::permission {

  use sui::table::{Self, Table};

  use htf::permission_to_attest::{PermissionToAttest};
  use htf::permission_to_accredit::{PermissionToAccredit};

  public struct Permissions has store {
    attestations : Table<ID, vector<PermissionToAttest>>,
    permissions_to_accredit : Table<ID, vector<PermissionToAccredit>>,
  }

  public(package) fun empty(ctx :&mut TxContext) : Permissions {
    Permissions {
      attestations : table::new(ctx),
      permissions_to_accredit : table::new(ctx),
    }
   }

}
