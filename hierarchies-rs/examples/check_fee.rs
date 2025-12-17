// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use hierarchies::client::HierarchiesClient;
use hierarchies::core::types::property::FederationProperty;
use hierarchies::core::types::property_name::PropertyName;
use hierarchies::core::types::property_value::PropertyValue;
use hierarchies_examples::get_funded_client;
use iota_sdk::{
    rpc_types::{IotaTransactionBlockEffectsAPI, IotaTransactionBlockResponse},
    types::{base_types::ObjectID, gas_coin::NANOS_PER_IOTA},
};

use anyhow::Context;
use clap::Parser;
use log::info;
use product_common::test_utils::InMemSigner;
use serde::{Deserialize, Serialize};

/// Command line arguments for property and accreditor counts.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of properties to create (default: 20)
    #[arg(short, long, default_value_t = 20)]
    properties: u64,
    /// Number of accreditors to create (default: 5)
    #[arg(short, long, default_value_t = 5)]
    accreditors: u64,
    /// Number of properties per accreditor (default: 2)
    #[arg(short = 'n', long, default_value_t = 2)]
    properties_per_accreditor: u64,
}

fn init_logger() {
    let _ = env_logger::builder().filter_level(log::LevelFilter::Info).try_init();
}

/// Main function to calculate the fees for the creation of a federation with a given number of properties and accreditors.
/// The fees are calculated for the creation of the federation, the properties, the accreditations and the validation of the properties.
/// The fees are saved to a YAML file.
/// The YAML file is named "costs_{number_of_properties}_properties_{number_of_accreditors}_accreditors.yaml".
/// The YAML file contains the following fields:
/// - iota_price: the price of IOTA in USD
/// - creation_cost_iota: the cost of the creation of the federation in IOTA
/// - properties_cost_iota: the cost of the creation of the properties in IOTA
/// - accreditations_cost_iota: the cost of the creation of the accreditations in IOTA
/// - validation_cost_iota: the cost of the validation of the properties in IOTA
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logger();

    // Parse command-line arguments
    let args = Args::parse();

    let number_of_properties = args.properties;
    let number_of_accreditors = args.accreditors;

    let hierarchies_client = get_funded_client().await?;
    let mut calculator = Calculator::new(hierarchies_client).await?;

    calculator.create_data_matrix(
        number_of_properties,
        number_of_accreditors,
        args.properties_per_accreditor,
    )?;
    calculator.create_federation().await?;
    calculator.create_property().await?;
    calculator.create_accreditation().await?;
    calculator.validate_properties().await?;
    calculator.costs_matrix.save_to_yaml(&Path::new(&format!(
        "costs_{number_of_properties}_properties_{number_of_accreditors}_accreditors.yaml"
    )))?;

    Ok(())
}

/// Convert gas units to IOTA tokens
fn gas_to_iota(gas: u64) -> f64 {
    gas as f64 / NANOS_PER_IOTA as f64
}

fn extract_gas_from_response(response: IotaTransactionBlockResponse) -> u64 {
    if let Some(effects) = &response.effects {
        return effects.gas_cost_summary().net_gas_usage() as u64;
    }
    0
}
/// Fetch IOTA price in USD from CoinGecko API
async fn fetch_iota_price() -> anyhow::Result<f64> {
    let url = "https://api.coingecko.com/api/v3/simple/price?ids=iota&vs_currencies=usd";
    let response = reqwest::get(url)
        .await
        .context("Failed to fetch IOTA price from CoinGecko")?;

    let json: serde_json::Value = response.json().await.context("Failed to parse CoinGecko response")?;

    json.get("iota")
        .and_then(|iota| iota.get("usd"))
        .and_then(|usd| usd.as_f64())
        .context("Failed to extract IOTA price from API response")
}

pub struct Calculator {
    client: HierarchiesClient<InMemSigner>,
    federation_id: Option<ObjectID>,
    data_matrix: DataMatrix,
    pub costs_matrix: CostsMatrix,
}

#[derive(Default)]
pub struct DataMatrix {
    pub properties: Vec<FederationProperty>,
    pub accreditors: HashMap<ObjectID, Vec<FederationProperty>>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct CostsMatrix {
    iota_price: f64,
    creation_cost_iota: f64,
    properties_cost_iota: f64,
    accreditations_cost_iota: f64,
    validation_cost_iota: f64,
}

impl CostsMatrix {
    pub fn total_cost_iota(&self) -> f64 {
        self.creation_cost_iota + self.properties_cost_iota + self.accreditations_cost_iota + self.validation_cost_iota
    }
    pub fn total_cost_dollars(&self) -> f64 {
        self.total_cost_iota() * self.iota_price
    }
    pub fn creation_cost_dollars(&self) -> f64 {
        self.creation_cost_iota * self.iota_price
    }
    pub fn properties_cost_dollars(&self) -> f64 {
        self.properties_cost_iota * self.iota_price
    }
    pub fn accreditations_cost_dollars(&self) -> f64 {
        self.accreditations_cost_iota * self.iota_price
    }

    pub fn save_to_yaml(&self, path: &Path) -> anyhow::Result<()> {
        let file = File::create(path).context("Failed to create file")?;
        let mut writer = BufWriter::new(file);
        serde_yaml::to_writer(&mut writer, self)?;
        Ok(())
    }
}

impl Calculator {
    pub async fn new(client: HierarchiesClient<InMemSigner>) -> anyhow::Result<Self> {
        let iota_price = fetch_iota_price().await?;
        let costs_matrix = CostsMatrix {
            iota_price,
            ..CostsMatrix::default()
        };
        Ok(Self {
            client,
            federation_id: None,
            data_matrix: DataMatrix::default(),
            costs_matrix,
        })
    }

    pub fn create_data_matrix(
        &mut self,
        number_of_properties: u64,
        number_of_accreditors: u64,
        properties_per_accreditor: u64,
    ) -> anyhow::Result<()> {
        for i in 0..number_of_properties {
            let property_name = PropertyName::from(format!("Property Value {}", i));
            let value = PropertyValue::Text(format!("Hello {}", i));
            let allowed_values = HashSet::from([value.clone(), value.clone(), value.clone()]);
            let property = FederationProperty::new(property_name.clone()).with_allowed_values(allowed_values);
            self.data_matrix.properties.push(property);
        }

        let mut accreditors = HashMap::new();
        for _ in 0..number_of_accreditors {
            let accreditor_id = ObjectID::random();
            accreditors.insert(accreditor_id, Vec::new());
        }

        let mut cycle = self.data_matrix.properties.iter().cycle();

        for (_, properties) in accreditors.iter_mut() {
            for _ in 0..properties_per_accreditor {
                let property = cycle.next().unwrap().clone();
                properties.push(property);
            }
        }
        self.data_matrix.accreditors = accreditors;

        Ok(())
    }

    pub async fn create_federation(&mut self) -> anyhow::Result<()> {
        let transaction_output = self
            .client
            .create_new_federation()
            .build_and_execute(&self.client)
            .await?;
        let federation_id = *transaction_output.output.id.object_id();

        self.costs_matrix.creation_cost_iota = gas_to_iota(extract_gas_from_response(transaction_output.response));
        info!("Creation cost in IOTA: {}", self.costs_matrix.creation_cost_iota);
        self.federation_id = Some(federation_id);

        Ok(())
    }

    pub async fn create_property(&mut self) -> anyhow::Result<()> {
        let federation_id = self.federation_id.context("Federation ID is not set")?;
        let properties = self.data_matrix.properties.clone();
        for property in properties {
            info!("Adding property: {:?}", property.name);
            let transaction_output = self
                .client
                .add_property(federation_id, property.clone())
                .build_and_execute(&self.client)
                .await?;
            self.costs_matrix.properties_cost_iota = self.costs_matrix.properties_cost_iota
                + gas_to_iota(extract_gas_from_response(transaction_output.response));
            info!("Property cost in IOTA: {}", self.costs_matrix.properties_cost_iota);
        }
        Ok(())
    }

    pub async fn create_accreditation(&mut self) -> anyhow::Result<()> {
        let federation_id = self.federation_id.context("Federation ID is not set")?;
        let accreditors = self.data_matrix.accreditors.clone();

        for (accreditor_id, properties) in accreditors {
            info!("Adding accreditation: to accreditor: {:?}", accreditor_id);
            let transaction_output = self
                .client
                .create_accreditation_to_accredit(federation_id, accreditor_id, properties.clone())
                .build_and_execute(&self.client)
                .await?;
            self.costs_matrix.accreditations_cost_iota = self.costs_matrix.accreditations_cost_iota
                + gas_to_iota(extract_gas_from_response(transaction_output.response));
            info!(
                "Accreditation cost in IOTA: {}",
                self.costs_matrix.accreditations_cost_iota
            );
        }
        Ok(())
    }

    pub async fn validate_properties(&mut self) -> anyhow::Result<()> {
        let federation_id = self.federation_id.context("Federation ID is not set")?;
        let accreditor_id = self.data_matrix.accreditors.keys().next().unwrap().clone();
        let last_property_name = self.data_matrix.properties.last().unwrap().name.clone();

        let not_matched_property_value = PropertyValue::Text("unmatched value".to_string());
        info!(
            "validation of property: {:?} for accreditor: {:?}",
            last_property_name, accreditor_id
        );

        let transaction_output = self
            .client
            .validate_property_non_free(
                federation_id,
                accreditor_id,
                last_property_name,
                not_matched_property_value,
            )
            .build_and_execute(&self.client)
            .await?;
        self.costs_matrix.validation_cost_iota = gas_to_iota(extract_gas_from_response(transaction_output.response));
        info!("validation result is: {:?}", transaction_output.output);
        info!("Validation cost in IOTA: {}", self.costs_matrix.validation_cost_iota);
        Ok(())
    }
}
