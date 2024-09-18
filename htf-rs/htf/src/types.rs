mod cap;
pub mod credentials;
pub mod event;
pub mod permission;
pub mod trusted_constraints;
pub mod trusted_property;

use std::collections::HashMap;

use credentials::CredentialState;
use iota_sdk::types::base_types::ObjectID;
use iota_sdk::types::id::UID;
use permission::{PermissionsToAccredit, PermissionsToAttest};
use serde::{Deserialize, Serialize};
use trusted_constraints::TrustedPropertyConstraints;

use crate::utils::deserialize_vec_map;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Federation {
  pub id: UID,
  pub governance: Governance,
  pub root_authorities: Vec<RootAuthority>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootAuthority {
  pub id: UID,
  pub account_id: ObjectID,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Governance {
  id: UID,
  pub trusted_constraints: TrustedPropertyConstraints,
  #[serde(deserialize_with = "deserialize_vec_map")]
  pub accreditors: HashMap<ObjectID, PermissionsToAccredit>,
  #[serde(deserialize_with = "deserialize_vec_map")]
  pub attesters: HashMap<ObjectID, PermissionsToAttest>,
  #[serde(deserialize_with = "deserialize_vec_map")]
  pub credentials_state: HashMap<ObjectID, CredentialState>,
}

#[test]
fn lol() {
  let string_fed = r#"{
    "governance": {
      "accreditors": {
        "contents": [
          {
            "key": "0x5f1e9fe407c6a3a2e88d4d876e5388321cd0f4dc14c8d55ecc98de13982ad564",
            "value": {
              "permissions": []
            }
          }
        ]
      },
      "attesters": {
        "contents": [
          {
            "key": "0x5f1e9fe407c6a3a2e88d4d876e5388321cd0f4dc14c8d55ecc98de13982ad564",
            "value": {
              "permissions": []
            }
          }
        ]
      },
      "credentials_state": {
        "contents": []
      },
      "id": {
        "id": "0x9124d283e72798320ada631fe9bff3da0eba69d0faa0d4cacbc6b24647da0205"
      },
      "trusted_constraints": {
        "data": {
          "contents": [
            {
              "key": {
                "names": ["Example LTD"]
              },
              "value": {
                "allow_any": false,
                "allowed_values": {
                  "contents": [
                    {
                      "number": null,
                      "text": "Hello"
                    }
                  ]
                },
                "expression": null,
                "property_name": {
                  "names": ["Example LTD"]
                }
              }
            }
          ]
        }
      }
    },
    "id": {
      "id": "0xcd29df4061f0ebbd57d50378a5ce4f0634d465c024a39916a3b1f61dea30356a"
    },
    "root_authorities": [
      {
        "account_id": "0x5f1e9fe407c6a3a2e88d4d876e5388321cd0f4dc14c8d55ecc98de13982ad564",
        "id": {
          "id": "0x266b908924bc4925b3dc211f0a37350e693aa5a07554e309abfe505276901783"
        }
      }
    ]
  }"#;
  let fed: Federation = serde_json::from_str(string_fed).unwrap();
  println!("Federation : {:#?}", fed);
}
