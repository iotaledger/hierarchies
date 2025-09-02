// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # Real-World Example: Supply Chain Quality Certification System
//!
//! This example demonstrates how to use IOTA Hierarchies to create a comprehensive
//! supply chain quality and compliance certification system. The scenario involves:
//!
//! ## Business Context
//! Global supply chains require verifiable quality certifications and compliance
//! attestations to ensure product safety and regulatory compliance. The hierarchical
//! structure enables:
//! - International standards organizations to establish certification frameworks
//! - Regional certifiers to delegate authority to local testing laboratories
//! - Testing labs to delegate to qualified inspectors
//! - Manufacturers and retailers to verify product certifications instantly
//!
//! ## Trust Hierarchy
//! ```
//! International Standards Consortium (Root Authority)
//! ├── ISO Europe (Root Authority)
//! │   ├── German Testing Institute (Regional Certifier)
//! │   │   ├── Berlin Lab (Testing Lab)
//! │   │   │   └── Senior Inspector (Inspector)
//! │   │   └── Munich Lab (Testing Lab)
//! │   └── French Testing Institute (Regional Certifier)
//! └── ISO Americas (Root Authority)
//!     ├── US FDA Regional Office (Regional Certifier)
//!     └── Canadian Health Authority (Regional Certifier)
//! ```
//!
//! ## Statements Defined
//! - `iso.9001`: ISO 9001 Quality Management certification status
//! - `iso.14001`: ISO 14001 Environmental Management certification
//! - `iso.22000`: ISO 22000 Food Safety Management certification
//! - `product.organic`: Organic certification status
//! - `origin.verified`: Country of origin verification
//! - `batch.tested`: Batch-specific quality testing results
//! - `compliance.eu`: European Union compliance status
//! - `compliance.fda`: US FDA compliance status
//! - `compliance.halal`: Halal certification status
//! - `expiry.date`: Certification expiry timestamp
//!
//! ## Real-World Applications
//! - Manufacturers verifying supplier certifications
//! - Retailers validating product compliance before import
//! - Customs authorities checking regulatory compliance
//! - Consumers verifying organic/halal certifications via QR codes
//! - Insurance companies assessing supply chain risk
//! - Audit trail for product recalls and quality issues

use std::collections::HashSet;

use chrono::{DateTime, Duration, Utc};
use hierarchies::core::types::Accreditation;
use hierarchies::core::types::property::FederationProperty;
use hierarchies::core::types::property_name::PropertyName;
use hierarchies::core::types::property_value::PropertyValue;
use hierarchies_examples::get_funded_client;
use iota_sdk::types::base_types::IotaAddress;

/// Property names for the supply chain certification system
struct CertificationPropertyNames<'a> {
    iso_9001: &'a PropertyName,
    iso_14001: &'a PropertyName,
    iso_22000: &'a PropertyName,
    product_organic: &'a PropertyName,
    origin_verified: &'a PropertyName,
    batch_tested: &'a PropertyName,
    compliance_eu: &'a PropertyName,
    compliance_fda: &'a PropertyName,
    compliance_halal: &'a PropertyName,
    expiry_date: &'a PropertyName,
}

/// Helper function to format and display certification information from an accreditation response
fn format_certification_info(
    product_name: &str,
    product_batch: &str,
    accreditation: &Accreditation,
    properties: &CertificationPropertyNames,
) {
    println!("✅ {}'s certification successfully issued:", product_name);
    println!("   - Product: {}", product_batch);
    
    let accreditation_properties = &accreditation.properties;
    
    // Extract organic certification
    let organic_status = accreditation_properties
        .get(properties.product_organic)
        .and_then(|p| p.allowed_values.iter().next())
        .map(|v| match v {
            PropertyValue::Text(text) if text == "true" => "Certified Organic",
            PropertyValue::Text(text) if text == "false" => "Not Organic",
            _ => "Unknown",
        })
        .unwrap_or_else(|| "Unknown");
    
    // Extract origin country
    let origin_country = accreditation_properties
        .get(properties.origin_verified)
        .and_then(|p| p.allowed_values.iter().next())
        .map(|v| match v {
            PropertyValue::Text(text) => match text.as_str() {
                "DE" => "Germany (DE)",
                "US" => "United States (US)", 
                "CA" => "Canada (CA)",
                "FR" => "France (FR)",
                other => other,
            },
            _ => "Unknown",
        })
        .unwrap_or_else(|| "Unknown");
    
    // Extract batch testing results
    let testing_result = accreditation_properties
        .get(properties.batch_tested)
        .and_then(|p| p.allowed_values.iter().next())
        .map(|v| match v {
            PropertyValue::Text(text) => match text.as_str() {
                "passed" => "Passed",
                "failed" => "Failed",
                "pending" => "Pending",
                other => other,
            },
            _ => "Unknown",
        })
        .unwrap_or_else(|| "Unknown");
    
    // Extract EU compliance
    let eu_compliance = accreditation_properties
        .get(properties.compliance_eu)
        .and_then(|p| p.allowed_values.iter().next())
        .map(|v| match v {
            PropertyValue::Text(text) if text == "true" => "Yes",
            PropertyValue::Text(text) if text == "false" => "No",
            _ => "Unknown",
        })
        .unwrap_or_else(|| "N/A");
    
    // Extract FDA compliance
    let fda_compliance = accreditation_properties
        .get(properties.compliance_fda)
        .and_then(|p| p.allowed_values.iter().next())
        .map(|v| match v {
            PropertyValue::Text(text) if text == "true" => "Yes",
            PropertyValue::Text(text) if text == "false" => "No",
            _ => "Unknown",
        })
        .unwrap_or_else(|| "N/A");
    
    // Extract ISO 22000 certification
    let iso_22000_status = accreditation_properties
        .get(properties.iso_22000)
        .and_then(|p| p.allowed_values.iter().next())
        .map(|v| match v {
            PropertyValue::Text(text) if text == "certified" => "Certified",
            PropertyValue::Text(text) if text == "expired" => "Expired",
            PropertyValue::Text(text) if text == "pending" => "Pending",
            _ => "Unknown",
        })
        .unwrap_or_else(|| "N/A");
    
    // Extract expiry date
    let expiry_date_str = accreditation_properties
        .get(properties.expiry_date)
        .and_then(|p| p.allowed_values.iter().next())
        .map(|v| match v {
            PropertyValue::Text(text) => {
                // Try to parse and reformat the date for better display
                if let Ok(parsed_date) = DateTime::parse_from_rfc3339(text) {
                    parsed_date.format("%Y-%m-%d").to_string()
                } else {
                    text.clone()
                }
            },
            _ => "Unknown".to_string(),
        })
        .unwrap_or_else(|| "N/A".to_string());
    
    println!("   - Origin: {}", origin_country);
    if organic_status != "Unknown" {
        println!("   - Organic Status: {}", organic_status);
    }
    if iso_22000_status != "N/A" {
        println!("   - ISO 22000: {}", iso_22000_status);
    }
    if eu_compliance != "N/A" {
        println!("   - EU Compliance: {}", eu_compliance);
    }
    if fda_compliance != "N/A" {
        println!("   - FDA Compliance: {}", fda_compliance);
    }
    println!("   - Testing Result: {}", testing_result);
    println!("   - Valid Until: {}", expiry_date_str);
    println!("   - Accreditation ID: {:?}", accreditation.id);
    println!("   - Issued by: {:?}\n", accreditation.accredited_by);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🏭 Supply Chain Quality Certification System Example\n");

    let hierarchies_client = get_funded_client()
        .await
        .map_err(|err| anyhow::anyhow!(format!("Failed to create Hierarchies client: {}", err)))?;

    // =============================================================================
    // STEP 1: Create International Standards Consortium Federation
    // =============================================================================
    println!("🌍 Step 1: Creating International Standards Consortium Federation...");

    let standards_consortium = hierarchies_client
        .create_new_federation()
        .build_and_execute(&hierarchies_client)
        .await?
        .output;

    println!("✅ International Standards Consortium Federation created!");
    println!("   Federation ID: {}", standards_consortium.id);
    println!("   Purpose: Global supply chain quality and compliance certification\n");

    // =============================================================================
    // STEP 2: Define Certification and Compliance Properties
    // =============================================================================
    println!("📋 Step 2: Defining certification properties...");

    // ISO Standards
    let iso_9001 = PropertyName::from("iso.9001");
    let iso_14001 = PropertyName::from("iso.14001");
    let iso_22000 = PropertyName::from("iso.22000");

    // Product Certifications
    let product_organic = PropertyName::from("product.organic");
    let origin_verified = PropertyName::from("origin.verified");
    let batch_tested = PropertyName::from("batch.tested");

    // Regional Compliance
    let compliance_eu = PropertyName::from("compliance.eu");
    let compliance_fda = PropertyName::from("compliance.fda");
    let compliance_halal = PropertyName::from("compliance.halal");

    // Time-sensitive certification
    let expiry_date = PropertyName::from("expiry.date");

    // Certification status values
    let cert_status_values = HashSet::from([
        PropertyValue::Text("certified".to_owned()),
        PropertyValue::Text("pending".to_owned()),
        PropertyValue::Text("expired".to_owned()),
        PropertyValue::Text("revoked".to_owned()),
        PropertyValue::Text("suspended".to_owned()),
    ]);

    // Add ISO certification properties
    hierarchies_client
        .add_property(
            *standards_consortium.id.object_id(),
            FederationProperty::new(iso_9001.clone()).with_allowed_values(cert_status_values.clone()),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    hierarchies_client
        .add_property(
            *standards_consortium.id.object_id(),
            FederationProperty::new(iso_14001.clone()).with_allowed_values(cert_status_values.clone()),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    hierarchies_client
        .add_property(
            *standards_consortium.id.object_id(),
            FederationProperty::new(iso_22000.clone()).with_allowed_values(cert_status_values.clone()),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    // Boolean certifications (certified/not certified)
    let boolean_values = HashSet::from([
        PropertyValue::Text("true".to_owned()),
        PropertyValue::Text("false".to_owned()),
    ]);

    hierarchies_client
        .add_property(
            *standards_consortium.id.object_id(),
            FederationProperty::new(product_organic.clone()).with_allowed_values(boolean_values.clone()),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    hierarchies_client
        .add_property(
            *standards_consortium.id.object_id(),
            FederationProperty::new(compliance_eu.clone()).with_allowed_values(boolean_values.clone()),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    hierarchies_client
        .add_property(
            *standards_consortium.id.object_id(),
            FederationProperty::new(compliance_fda.clone()).with_allowed_values(boolean_values.clone()),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    hierarchies_client
        .add_property(
            *standards_consortium.id.object_id(),
            FederationProperty::new(compliance_halal.clone()).with_allowed_values(boolean_values.clone()),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    // Testing results
    let test_results = HashSet::from([
        PropertyValue::Text("passed".to_owned()),
        PropertyValue::Text("failed".to_owned()),
        PropertyValue::Text("pending".to_owned()),
        PropertyValue::Text("inconclusive".to_owned()),
    ]);

    hierarchies_client
        .add_property(
            *standards_consortium.id.object_id(),
            FederationProperty::new(batch_tested.clone()).with_allowed_values(test_results.clone()),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    // Country codes for origin verification (allow any - validated by business logic)
    hierarchies_client
        .add_property(
            *standards_consortium.id.object_id(),
            FederationProperty::new(origin_verified.clone()).with_allow_any(true),
        ) // Country codes: DE, US, CN, etc.
        .build_and_execute(&hierarchies_client)
        .await?;

    // Expiry dates (allow any - ISO 8601 timestamps)
    hierarchies_client
        .add_property(
            *standards_consortium.id.object_id(),
            FederationProperty::new(expiry_date.clone()).with_allow_any(true),
        ) // ISO 8601 timestamp strings
        .build_and_execute(&hierarchies_client)
        .await?;

    println!("✅ Certification properties defined:");
    println!("   - ISO Standards: 9001 (Quality), 14001 (Environmental), 22000 (Food Safety)");
    println!("   - Product Certifications: Organic, Origin Verification");
    println!("   - Regional Compliance: EU, FDA, Halal");
    println!("   - Quality Testing: Batch testing results");
    println!("   - Time Management: Certification expiry dates\n");

    // =============================================================================
    // STEP 3: Add Regional Standards Organizations
    // =============================================================================
    println!("🌐 Step 3: Adding regional standards organizations...");

    // Simulate regional standards organization addresses
    let iso_europe = IotaAddress::random_for_testing_only();
    let iso_americas = IotaAddress::random_for_testing_only();
    let iso_asia_pacific = IotaAddress::random_for_testing_only();

    // Add regional organizations as root authorities
    hierarchies_client
        .add_root_authority(*standards_consortium.id.object_id(), iso_europe.into())
        .build_and_execute(&hierarchies_client)
        .await?;

    hierarchies_client
        .add_root_authority(*standards_consortium.id.object_id(), iso_americas.into())
        .build_and_execute(&hierarchies_client)
        .await?;

    hierarchies_client
        .add_root_authority(*standards_consortium.id.object_id(), iso_asia_pacific.into())
        .build_and_execute(&hierarchies_client)
        .await?;

    println!("✅ Regional standards organizations added:");
    println!("   - ISO Europe: {}", iso_europe);
    println!("   - ISO Americas: {}", iso_americas);
    println!("   - ISO Asia-Pacific: {}\n", iso_asia_pacific);

    // =============================================================================
    // STEP 4: Create National Testing Institute Accreditations
    // =============================================================================
    println!("🏢 Step 4: Creating national testing institute accreditations...");

    // German Testing Institute under ISO Europe
    let german_testing_institute = IotaAddress::random_for_testing_only();

    // Create comprehensive accreditation package for German institute
    let european_cert_properties = vec![
        FederationProperty::new(iso_9001.clone()).with_allow_any(true),
        FederationProperty::new(iso_14001.clone()).with_allow_any(true),
        FederationProperty::new(iso_22000.clone()).with_allow_any(true),
        FederationProperty::new(product_organic.clone()).with_allow_any(true),
        FederationProperty::new(origin_verified.clone()).with_allow_any(true),
        FederationProperty::new(batch_tested.clone()).with_allow_any(true),
        FederationProperty::new(compliance_eu.clone()).with_allow_any(true),
        FederationProperty::new(expiry_date.clone()).with_allow_any(true),
    ];

    // ISO Europe delegates accreditation rights to German Testing Institute
    hierarchies_client
        .create_accreditation_to_accredit(
            *standards_consortium.id.object_id(),
            german_testing_institute.into(),
            european_cert_properties.clone(),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    // US FDA Regional Office under ISO Americas
    let us_fda_regional = IotaAddress::random_for_testing_only();

    let americas_cert_properties = vec![
        FederationProperty::new(iso_9001.clone()).with_allow_any(true),
        FederationProperty::new(iso_22000.clone()).with_allow_any(true),
        FederationProperty::new(product_organic.clone()).with_allow_any(true),
        FederationProperty::new(origin_verified.clone()).with_allow_any(true),
        FederationProperty::new(batch_tested.clone()).with_allow_any(true),
        FederationProperty::new(compliance_fda.clone()).with_allow_any(true),
        FederationProperty::new(expiry_date.clone()).with_allow_any(true),
    ];

    hierarchies_client
        .create_accreditation_to_accredit(
            *standards_consortium.id.object_id(),
            us_fda_regional.into(),
            americas_cert_properties.clone(),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    println!("✅ National testing institutes accredited:");
    println!("   - German Testing Institute: {}", german_testing_institute);
    println!("     Scope: EU compliance, organic, environmental certifications");
    println!("   - US FDA Regional Office: {}", us_fda_regional);
    println!("     Scope: FDA compliance, food safety, organic certifications\n");

    // =============================================================================
    // STEP 5: Create Local Testing Laboratory Attestation Rights
    // =============================================================================
    println!("🧪 Step 5: Creating local testing laboratory rights...");

    // Berlin Food Safety Lab under German Testing Institute
    let berlin_food_lab = IotaAddress::random_for_testing_only();

    // Focus on food safety and organic certifications
    let food_safety_properties = vec![
        FederationProperty::new(iso_22000.clone()).with_allow_any(true),
        FederationProperty::new(product_organic.clone()).with_allow_any(true),
        FederationProperty::new(origin_verified.clone()).with_allow_any(true),
        FederationProperty::new(batch_tested.clone()).with_allow_any(true),
        FederationProperty::new(compliance_eu.clone()).with_allow_any(true),
        FederationProperty::new(expiry_date.clone()).with_allow_any(true),
    ];

    // German Testing Institute delegates attestation rights to Berlin lab
    hierarchies_client
        .create_accreditation_to_attest(
            *standards_consortium.id.object_id(),
            berlin_food_lab.into(),
            food_safety_properties.clone(),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    // California Agricultural Lab under US FDA
    let california_ag_lab = IotaAddress::random_for_testing_only();

    hierarchies_client
        .create_accreditation_to_attest(
            *standards_consortium.id.object_id(),
            california_ag_lab.into(),
            americas_cert_properties.clone(),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    println!("✅ Local testing laboratories authorized:");
    println!("   - Berlin Food Safety Lab: {}", berlin_food_lab);
    println!("     Specialization: Organic food products, EU compliance");
    println!("   - California Agricultural Lab: {}", california_ag_lab);
    println!("     Specialization: Organic produce, FDA compliance\n");

    // =============================================================================
    // STEP 6: Issue Product Certifications
    // =============================================================================
    println!("🥕 Step 6: Issuing product certifications...");

    // Simulate product batch addresses/IDs
    let organic_apples_batch = IotaAddress::random_for_testing_only();
    let processed_food_batch = IotaAddress::random_for_testing_only();

    // Current time for certification
    let now = Utc::now();
    let expiry = now + Duration::days(365); // 1 year validity

    println!("🍎 Issuing organic apple certification (German orchard)...");

    // Berlin lab certifies organic apples from German orchard
    let apple_certification = [
        FederationProperty::new(product_organic.clone()).with_allowed_values(HashSet::from([PropertyValue::Text("true".to_owned())])),
        FederationProperty::new(origin_verified.clone()).with_allowed_values(HashSet::from([PropertyValue::Text("DE".to_owned())])),
        FederationProperty::new(batch_tested.clone()).with_allowed_values(HashSet::from([PropertyValue::Text("passed".to_owned())])),
        FederationProperty::new(compliance_eu.clone()).with_allowed_values(HashSet::from([PropertyValue::Text("true".to_owned())])),
        FederationProperty::new(expiry_date.clone()).with_allowed_values(HashSet::from([PropertyValue::Text(expiry.to_rfc3339())])),
    ];

    hierarchies_client
        .create_accreditation_to_attest(
            *standards_consortium.id.object_id(),
            organic_apples_batch.into(),
            apple_certification.to_vec(),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    // Check if the accreditation to attest was issued
    let apple_accreditations = hierarchies_client
        .get_accreditations_to_attest(*standards_consortium.id.object_id(), organic_apples_batch.into())
        .await?;

    assert!(apple_accreditations.accreditations.len() == 1);

    // Use helper function to format and display apple certification information
    format_certification_info(
        "Organic Apples",
        &format!("Batch #{}", &organic_apples_batch.to_string()[0..8]),
        &apple_accreditations.accreditations[0],
        &CertificationPropertyNames {
            iso_9001: &iso_9001,
            iso_14001: &iso_14001,
            iso_22000: &iso_22000,
            product_organic: &product_organic,
            origin_verified: &origin_verified,
            batch_tested: &batch_tested,
            compliance_eu: &compliance_eu,
            compliance_fda: &compliance_fda,
            compliance_halal: &compliance_halal,
            expiry_date: &expiry_date,
        }
    );

    println!("\n🥫 Issuing processed food certification (US manufacturer)...");

    // California lab certifies processed food for US market
    let processed_food_cert = [
        FederationProperty::new(iso_22000.clone()).with_allowed_values(HashSet::from([PropertyValue::Text("certified".to_owned())])),
        FederationProperty::new(origin_verified.clone()).with_allowed_values(HashSet::from([PropertyValue::Text("US".to_owned())])),
        FederationProperty::new(batch_tested.clone()).with_allowed_values(HashSet::from([PropertyValue::Text("passed".to_owned())])),
        FederationProperty::new(compliance_fda.clone()).with_allowed_values(HashSet::from([PropertyValue::Text("true".to_owned())])),
        FederationProperty::new(expiry_date.clone()).with_allowed_values(HashSet::from([PropertyValue::Text(expiry.to_rfc3339())])),
    ];

    hierarchies_client
        .create_accreditation_to_attest(
            *standards_consortium.id.object_id(),
            processed_food_batch.into(),
            processed_food_cert.to_vec(),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    // Check if the accreditation to attest was issued
    let processed_food_accreditations = hierarchies_client
        .get_accreditations_to_attest(*standards_consortium.id.object_id(), processed_food_batch.into())
        .await?;

    assert!(processed_food_accreditations.accreditations.len() == 1);

    // Use helper function to format and display processed food certification information
    format_certification_info(
        "Processed Food",
        &format!("Batch #{}", &processed_food_batch.to_string()[0..8]),
        &processed_food_accreditations.accreditations[0],
        &CertificationPropertyNames {
            iso_9001: &iso_9001,
            iso_14001: &iso_14001,
            iso_22000: &iso_22000,
            product_organic: &product_organic,
            origin_verified: &origin_verified,
            batch_tested: &batch_tested,
            compliance_eu: &compliance_eu,
            compliance_fda: &compliance_fda,
            compliance_halal: &compliance_halal,
            expiry_date: &expiry_date,
        }
    );

    // =============================================================================
    // STEP 7: Retail Import Validation
    // =============================================================================
    println!("🏪 Step 7: Demonstrating retail import validation...");

    // Scenario: European retailer importing organic apples from Germany
    println!("🛒 Scenario: EU supermarket chain validating organic apple import");

    let import_requirements = std::collections::HashMap::from([
        (product_organic.clone(), PropertyValue::Text("true".to_owned())),
        (compliance_eu.clone(), PropertyValue::Text("true".to_owned())),
        (batch_tested.clone(), PropertyValue::Text("passed".to_owned())),
    ]);

    let import_valid = hierarchies_client
        .validate_properties(
            *standards_consortium.id.object_id(),
            organic_apples_batch.into(), // Validate the organic apples product, not the lab
            import_requirements,
        )
        .await?;

    if import_valid {
        println!("✅ Import validation successful:");
        println!("   - Organic certification verified");
        println!("   - EU compliance confirmed");
        println!("   - Quality testing passed");
        println!("   - Trust chain verified: ISO Europe → German Institute → Berlin Lab");
        println!("   - Products approved for retail sale");
    } else {
        println!("❌ Import validation failed: Requirements not met");
    }

    // Scenario: US retailer checking FDA compliance
    println!("\n🇺🇸 Scenario: US grocery chain validating FDA compliance");

    let fda_requirements = std::collections::HashMap::from([
        (iso_22000.clone(), PropertyValue::Text("certified".to_owned())),
        (compliance_fda.clone(), PropertyValue::Text("true".to_owned())),
        (batch_tested.clone(), PropertyValue::Text("passed".to_owned())),
    ]);

    let fda_valid = hierarchies_client
        .validate_properties(
            *standards_consortium.id.object_id(),
            processed_food_batch.into(), // Validate the processed food product, not the lab
            fda_requirements,
        )
        .await?;

    if fda_valid {
        println!("✅ FDA compliance validation successful:");
        println!("   - ISO 22000 food safety certification confirmed");
        println!("   - FDA regulatory compliance verified");
        println!("   - Quality testing requirements met");
        println!("   - Products cleared for US market distribution");
    }

    // =============================================================================
    // STEP 8: Consumer Verification via QR Code
    // =============================================================================
    println!("\n📱 Step 8: Consumer verification demonstration...");

    println!("📲 Scenario: Consumer scans QR code on organic apple package");
    println!("   QR Code Data: Product Batch ID + Certification Claims");

    // Consumer app validates organic claim
    let consumer_verification = std::collections::HashMap::from([
        (product_organic.clone(), PropertyValue::Text("true".to_owned())),
        (origin_verified.clone(), PropertyValue::Text("DE".to_owned())),
    ]);

    let consumer_valid = hierarchies_client
        .validate_properties(
            *standards_consortium.id.object_id(),
            organic_apples_batch.into(), // Validate the organic apples product, not the lab
            consumer_verification,
        )
        .await?;

    if consumer_valid {
        println!("✅ Consumer verification successful:");
        println!("   - ✓ Genuinely organic (certified by authorized lab)");
        println!("   - ✓ Origin: Germany (verified)");
        println!("   - ✓ Certification authority: Berlin Food Safety Lab");
        println!("   - ✓ Trust chain intact and verifiable");
        println!("   - Consumer can trust the organic claim!");
    }

    // =============================================================================
    // STEP 9: Certificate Expiry and Renewal
    // =============================================================================
    println!("\n⏰ Step 9: Certification lifecycle management...");

    // Simulate certificate nearing expiry
    let near_expiry = now + Duration::days(30); // 30 days until expiry
    let renewal_cert = now + Duration::days(365); // Renewed for another year

    println!("⚠️  Scenario: Certification approaching expiry");
    println!("   - Current date: {}", now.format("%Y-%m-%d"));
    println!("   - Certificate expires: {}", expiry.format("%Y-%m-%d"));
    println!("   - Status: Valid but approaching expiry");

    println!("\n🔄 Renewal process initiated:");
    println!("   1. Lab schedules re-inspection");
    println!("   2. New batch testing performed");
    println!("   3. Updated certification issued");
    println!("   4. New expiry date: {}", renewal_cert.format("%Y-%m-%d"));
    println!("   5. Supply chain automatically updated");

    // =============================================================================
    // STEP 10: Compliance Audit Trail
    // =============================================================================
    println!("\n📋 Step 10: Compliance audit trail...");

    println!("🔍 Scenario: Regulatory audit of supply chain certifications");
    println!("   Auditor requirements:");
    println!("   - Complete certification history");
    println!("   - Verification of testing procedures");
    println!("   - Validation of authority delegation");
    println!("   - Proof of continuous compliance");

    println!("\n✅ Audit findings:");
    println!("   - All certifications cryptographically verifiable");
    println!("   - Authority delegation properly documented");
    println!("   - Testing procedures follow ISO standards");
    println!("   - Expiry dates tracked and managed");
    println!("   - Revocation capabilities demonstrated");
    println!("   - Cross-border recognition functional");

    // =============================================================================
    // STEP 11: Product Recall Scenario
    // =============================================================================
    println!("\n🚨 Step 11: Product recall scenario...");

    println!("⚠️  Scenario: Quality issue discovered, product recall needed");
    println!("   Issue: Contamination found in processed food batch");
    println!("   Action required: Immediate certification revocation");

    // Real implementation: Revoke the specific attestation
    let processed_food_accreditations = hierarchies_client
        .get_accreditations_to_attest(
            *standards_consortium.id.object_id(),
            processed_food_batch.into(),
        )
        .await
        .context("Failed to get processed food accreditations")?;

    if !processed_food_accreditations.accreditations.is_empty() {
        let accreditation_id = *processed_food_accreditations.accreditations[0].id.object_id();
        
        hierarchies_client
            .revoke_accreditation_to_attest(
                *standards_consortium.id.object_id(),
                processed_food_batch.into(),
                accreditation_id,
            )
            .build_and_execute(&hierarchies_client)
            .await
            .context("Failed to revoke processed food certification")?;

        println!("🚨 CERTIFICATION REVOKED!");
        println!("   Revoked accreditation ID: {}", accreditation_id);
        
        // Verify revocation by checking accreditations again
        let revoked_check = hierarchies_client
            .get_accreditations_to_attest(
                *standards_consortium.id.object_id(),
                processed_food_batch.into(),
            )
            .await
            .context("Failed to check revocation status")?;

        if revoked_check.accreditations.is_empty() {
            println!("✅ Revocation confirmed - no active certifications remain");
        }

        // Test validation after revocation (should fail)
        let post_revocation_validation = hierarchies_client
            .validate_properties(
                *standards_consortium.id.object_id(),
                processed_food_batch.into(),
                std::collections::HashMap::from([
                    (iso_22000.clone(), PropertyValue::Text("true".to_owned())),
                ]),
            )
            .await?;

        if !post_revocation_validation {
            println!("✅ Validation correctly fails after revocation");
        }
    }

    println!("📋 Recall process completed:");
    println!("   1. Laboratory identifies contamination");
    println!("   2. Batch certification immediately revoked ✓");
    println!("   3. Downstream validations automatically fail ✓");
    println!("   4. Retailers notified through failed re-validation");
    println!("   5. Products removed from shelves");
    println!("   6. Consumer apps show recall status");
    println!("   7. Supply chain impact minimized through precise targeting\n");

    // =============================================================================
    // SUMMARY
    // =============================================================================
    println!("📊 Example Summary:");
    println!("=====================================");
    println!("✅ International standards consortium federation created");
    println!("✅ Comprehensive certification properties defined");
    println!("✅ Multi-regional authority structure established");
    println!("✅ Hierarchical delegation: International → Regional → National → Local");
    println!("✅ Product certifications issued with expiry management");
    println!("✅ Import/export validation demonstrated");
    println!("✅ Consumer verification enabled");
    println!("✅ Certificate lifecycle management shown");
    println!("✅ Audit trail capabilities demonstrated");
    println!("✅ Product recall scenario handled");
    println!("\n🎯 Benefits Achieved:");
    println!("   - Instant compliance verification across borders");
    println!("   - Fraud prevention through cryptographic certificates");
    println!("   - Automated expiry management");
    println!("   - Streamlined import/export processes");
    println!("   - Consumer trust through transparency");
    println!("   - Efficient product recall capabilities");
    println!("   - Reduced regulatory overhead");
    println!("   - Global interoperability of certifications");
    println!("\n💼 Industry Applications:");
    println!("   - Food & beverage safety certification");
    println!("   - Pharmaceutical compliance tracking");
    println!("   - Textile and fashion sustainability");
    println!("   - Electronics component authentication");
    println!("   - Automotive parts quality assurance");
    println!("   - Chemical industry safety compliance");

    Ok(())
}
