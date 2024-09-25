// HTF Notary module
module htf::permission {

  use iota::vec_map::{VecMap, Self};
  

  use htf::permission_to_attest::{PermissionToAttest};
  use htf::permission_to_accredit::{PermissionToAccredit};

  public struct Permissions has store {
    attestations : VecMap<ID, vector<PermissionToAttest>>,
    permissions_to_accredit : VecMap<ID, vector<PermissionToAccredit>>,
  }

  public(package) fun empty() : Permissions {
    Permissions {
      attestations : vec_map::empty(),
      permissions_to_accredit : vec_map::empty(),
    }
  }

}
