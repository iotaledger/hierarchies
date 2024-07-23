// HTF Notary module
module htf::permission {

  use sui::table::{Self, Table};

  use htf::permission_to_atest::{PermissionToAtest};
  use htf::permission_to_acredit::{PermissionToAcredit};

  public struct Permissions has store {
    atestations : Table<ID, vector<PermissionToAtest>>,
    permissions_to_acredit : Table<ID, vector<PermissionToAcredit>>,
  }

  public(package) fun empty(ctx :&mut TxContext) : Permissions {
    Permissions {
      atestations : table::new(ctx),
      permissions_to_acredit : table::new(ctx),
    }
   }

}
