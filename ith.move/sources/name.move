
module ith::statement_name {
  use iota::object::{Self, ID, UID};
  use iota::tx_context::TxContext;
  use std::string::String;


  /// StatementName represents a name of a fact. It can be a single name or a vector of names.
  public struct StatementName  has copy, drop, store {
    names : vector<String>,
  }

  public fun names(self : &StatementName) : &vector<String> {
    &self.names
  }

  public fun new_statement_name(v : String) : StatementName {
    let mut names = vector::empty();
    names.push_back(v);
    StatementName {
      names,
    }
  }

  public fun new_statement_name_from_vector(v : vector<String>) : StatementName {
    StatementName {
      names : v,
    }
  }
}
