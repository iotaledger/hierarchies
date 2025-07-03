pub mod operations;
pub mod transactions;
pub mod types;

use iota_interaction::types::programmable_transaction_builder::ProgrammableTransactionBuilder as Ptb;
use iota_interaction::types::transaction::{Argument, ObjectArg};
use iota_interaction::types::{IOTA_CLOCK_OBJECT_ID, IOTA_CLOCK_OBJECT_SHARED_VERSION};

/// Adds a reference to the on-chain clock to `ptb`'s arguments.
pub(crate) fn get_clock_ref(ptb: &mut Ptb) -> Argument {
    ptb.obj(ObjectArg::SharedObject {
        id: IOTA_CLOCK_OBJECT_ID,
        initial_shared_version: IOTA_CLOCK_OBJECT_SHARED_VERSION,
        mutable: false,
    })
    .expect("network has a singleton clock instantiated")
}
