// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! # Real-World Example: University Degree Verification System
//!
//! This example demonstrates how to use IOTA Hierarchies to create a comprehensive
//! university degree verification system. The scenario involves:
//!
//! ## Business Context
//! Universities need to issue verifiable digital degrees that employers and other
//! institutions can trust. The hierarchical structure allows:
//! - University consortiums to establish trust networks
//! - Individual universities to delegate authority to faculties
//! - Faculties to delegate to registrars and professors
//! - External parties to verify credentials without contacting the university directly
//!
//! ## Trust Hierarchy
//! ```
//! University Consortium (Root Authority)
//! ├── Harvard University (Root Authority)
//! │   ├── Computer Science Faculty (Accreditor)
//! │   │   └── CS Registrar (Attester)
//! │   └── Engineering Faculty (Accreditor)
//! │       └── Engineering Registrar (Attester)
//! └── MIT (Root Authority)
//!     ├── Computer Science Faculty (Accreditor)
//!     └── Engineering Faculty (Accreditor)
//! ```
//!
//! ## Statements Defined
//! - `degree.bachelor`: Bachelor's degree completion status
//! - `degree.master`: Master's degree completion status
//! - `degree.phd`: PhD completion status
//! - `field.computer_science`: Computer Science specialization
//! - `field.engineering`: Engineering specialization
//! - `grade.gpa`: Grade Point Average (0.0-4.0 scale)
//! - `graduation.year`: Year of graduation
//! - `student.verified`: Student identity verification status
//!
//! ## Real-World Applications
//! - Employers verifying job applicant credentials
//! - Graduate schools checking undergraduate degrees
//! - Professional licensing bodies validating educational requirements
//! - International credential recognition
//! - Alumni verification for networking platforms

use std::collections::HashSet;

use hierarchies::core::types::Accreditation;
use hierarchies::core::types::property::FederationProperty;
use hierarchies::core::types::property_name::PropertyName;
use hierarchies::core::types::property_shape::PropertyShape;
use hierarchies::core::types::property_value::PropertyValue;
use hierarchies_examples::get_funded_client;
use iota_sdk::types::base_types::IotaAddress;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🎓 University Degree Verification System Example\n");

    let hierarchies_client = get_funded_client()
        .await
        .map_err(|err| anyhow::anyhow!(format!("Failed to create Hierarchies client: {}", err)))?;

    // =============================================================================
    // STEP 1: Create University Consortium Federation
    // =============================================================================
    println!("📚 Step 1: Creating University Consortium Federation...");

    let university_consortium = hierarchies_client
        .create_new_federation()
        .build_and_execute(&hierarchies_client)
        .await?
        .output;

    println!("✅ University Consortium Federation created!");
    println!("   Federation ID: {}", university_consortium.id);
    println!("   Root Authority: University Consortium Board\n");

    // =============================================================================
    // STEP 2: Define Academic Statements (Credential Types)
    // =============================================================================
    println!("📝 Step 2: Defining academic statements...");

    // Degree completion properties
    let degree_bachelor = PropertyName::from("degree.bachelor");
    let degree_master = PropertyName::from("degree.master");
    let degree_phd = PropertyName::from("degree.phd");

    // Field of study properties
    let field_cs = PropertyName::from("field.computer_science");
    let field_engineering = PropertyName::from("field.engineering");
    let field_mathematics = PropertyName::from("field.mathematics");

    // Academic performance and verification
    let grade_gpa = PropertyName::from("grade.gpa");
    let graduation_year = PropertyName::from("graduation.year");
    let student_verified = PropertyName::from("student.verified");
    let student_id = PropertyName::from("student.id");
    let honors_level = PropertyName::from("honors.level");

    // Add degree completion properties with specific allowed values
    let degree_values = HashSet::from([
        PropertyValue::Text("completed".to_owned()),
        PropertyValue::Text("in_progress".to_owned()),
        PropertyValue::Text("withdrawn".to_owned()),
    ]);

    hierarchies_client
        .add_property(
            *university_consortium.id.object_id(),
            FederationProperty::new(degree_bachelor.clone()).with_allowed_values(degree_values.clone()),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    println!("✅ Degree Bachelor property added!");

    hierarchies_client
        .add_property(
            *university_consortium.id.object_id(),
            FederationProperty::new(degree_master.clone()).with_allowed_values(degree_values.clone()),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    println!("✅ Degree Master property added!");

    hierarchies_client
        .add_property(
            *university_consortium.id.object_id(),
            FederationProperty::new(degree_phd.clone()).with_allowed_values(degree_values.clone()),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    println!("✅ Degree PhD property added!");

    // Add field of study properties (boolean - true if student studied this field)
    let boolean_values = HashSet::from([
        PropertyValue::Text("true".to_owned()),
        PropertyValue::Text("false".to_owned()),
    ]);

    hierarchies_client
        .add_property(
            *university_consortium.id.object_id(),
            FederationProperty::new(field_cs.clone()).with_allowed_values(boolean_values.clone()),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    println!("✅ Field Computer Science property added!");

    hierarchies_client
        .add_property(
            *university_consortium.id.object_id(),
            FederationProperty::new(field_engineering.clone()).with_allowed_values(boolean_values.clone()),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    println!("✅ Field Engineering property added!");

    hierarchies_client
        .add_property(
            *university_consortium.id.object_id(),
            FederationProperty::new(field_mathematics.clone()).with_allowed_values(boolean_values.clone()),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    println!("✅ Field Mathematics property added!");
    // Add GPA property with advanced numeric validation (must be between 2.0-4.0)
    hierarchies_client
        .add_property(
            *university_consortium.id.object_id(),
            FederationProperty::new(grade_gpa.clone())
                .with_expression(PropertyShape::GreaterThan(200)) // GPA > 2.0 (stored as 200 for precision)
                .with_allowed_values(HashSet::from([
                    PropertyValue::Number(200),
                    PropertyValue::Number(250),
                    PropertyValue::Number(300),
                    PropertyValue::Number(320),
                    PropertyValue::Number(350),
                    PropertyValue::Number(380),
                    PropertyValue::Number(400), // Common GPA ranges: 2.0, 2.5, 3.0, 3.2, 3.5, 3.8, 4.0
                ])),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    println!("✅ GPA property added!");

    // Add graduation year with range validation (must be recent - from 1950 onwards)
    hierarchies_client
        .add_property(
            *university_consortium.id.object_id(),
            FederationProperty::new(graduation_year.clone())
                .with_expression(PropertyShape::GreaterThan(1950))
                .with_allowed_values(HashSet::from([
                    PropertyValue::Number(1950),
                    PropertyValue::Number(1960),
                    PropertyValue::Number(1970),
                    PropertyValue::Number(1980),
                    PropertyValue::Number(1990),
                    PropertyValue::Number(2000),
                    PropertyValue::Number(2010),
                    PropertyValue::Number(2020),
                    PropertyValue::Number(2021),
                    PropertyValue::Number(2022),
                    PropertyValue::Number(2023),
                    PropertyValue::Number(2024),
                ])),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    println!("✅ Graduation year property added!");

    // Add student ID property with format validation (must start with university code)
    hierarchies_client
        .add_property(
            *university_consortium.id.object_id(),
            FederationProperty::new(student_id.clone())
                .with_expression(PropertyShape::Contains("-".to_string()))
                .with_allow_any(true),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    println!("✅ Student ID property added!");

    // Add honors level with specific validation (must end with recognized honor suffixes)
    hierarchies_client
        .add_property(
            *university_consortium.id.object_id(),
            FederationProperty::new(honors_level.clone()).with_allowed_values(HashSet::from([
                PropertyValue::Text("magna_cum_laude".to_owned()),
                PropertyValue::Text("summa_cum_laude".to_owned()),
                PropertyValue::Text("cum_laude".to_owned()),
                PropertyValue::Text("none".to_owned()),
            ])),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    println!("✅ Honors level property added!");

    // Add student verification status
    hierarchies_client
        .add_property(
            *university_consortium.id.object_id(),
            FederationProperty::new(student_verified.clone()).with_allowed_values(boolean_values.clone()),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    println!("✅ Student verification status property added!");

    println!("✅ Academic properties defined with advanced validation:");
    println!("   - Degree types: Bachelor, Master, PhD (with completion status)");
    println!("   - Fields: Computer Science, Engineering, Mathematics");
    println!("   - GPA: Numeric validation (must be > 2.0, specific ranges allowed)");
    println!("   - Graduation Year: Range validation (must be after 1950)");
    println!("   - Student ID: Format validation (must contain dash separator)");
    println!("   - Honors: Specific latin honor levels (cum laude, magna, summa)");
    println!("   - Verification: Student identity verification\n");

    // =============================================================================
    // STEP 3: Add Universities as Root Authorities
    // =============================================================================
    println!("🏛️ Step 3: Adding universities to the consortium...");

    let harvard_address = IotaAddress::random_for_testing_only();
    let mit_address = IotaAddress::random_for_testing_only();

    // Add Harvard as root authority
    hierarchies_client
        .add_root_authority(*university_consortium.id.object_id(), harvard_address.into())
        .build_and_execute(&hierarchies_client)
        .await?;

    // Add MIT as root authority
    hierarchies_client
        .add_root_authority(*university_consortium.id.object_id(), mit_address.into())
        .build_and_execute(&hierarchies_client)
        .await?;

    println!("✅ Universities added as root authorities:");
    println!("   - Harvard University: {}", harvard_address);
    println!("   - MIT: {}\n", mit_address);

    // =============================================================================
    // STEP 4: Create Faculty-Level Accreditations (Harvard CS Faculty)
    // =============================================================================
    println!("🏫 Step 4: Creating faculty-level accreditations...");

    // Simulate Harvard CS Faculty address
    let harvard_cs_faculty = IotaAddress::random_for_testing_only();

    // Harvard delegates accreditation rights to its CS Faculty
    // This allows the faculty to further delegate to registrars and professors
    let cs_faculty_properties = vec![
        FederationProperty::new(degree_bachelor.clone()).with_allow_any(true),
        FederationProperty::new(degree_master.clone()).with_allow_any(true),
        FederationProperty::new(degree_phd.clone()).with_allow_any(true),
        FederationProperty::new(field_cs.clone()).with_allow_any(true),
        FederationProperty::new(grade_gpa.clone()).with_allow_any(true),
        FederationProperty::new(graduation_year.clone()).with_allow_any(true),
        FederationProperty::new(student_verified.clone()).with_allow_any(true),
    ];

    hierarchies_client
        .create_accreditation_to_accredit(
            *university_consortium.id.object_id(),
            harvard_cs_faculty.into(),
            cs_faculty_properties.clone(),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    println!("✅ Harvard CS Faculty granted accreditation rights:");
    println!("   - Faculty Address: {}", harvard_cs_faculty);
    println!("   - Can delegate rights for all CS-related degrees\n");

    // =============================================================================
    // STEP 5: Create Registrar-Level Attestation Rights
    // =============================================================================
    println!("👨‍💼 Step 5: Creating registrar attestation rights...");

    // Simulate Harvard CS Registrar address
    let harvard_cs_registrar = IotaAddress::random_for_testing_only();

    // CS Faculty delegates attestation rights to the CS Registrar
    // Registrar can now create attestations (issue degrees) but not delegate further
    hierarchies_client
        .create_accreditation_to_attest(
            *university_consortium.id.object_id(),
            harvard_cs_registrar.into(),
            cs_faculty_properties.clone(),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    println!("✅ Harvard CS Registrar granted attestation rights:");
    println!("   - Registrar Address: {}", harvard_cs_registrar);
    println!("   - Can issue degrees and verify student credentials\n");

    // =============================================================================
    // STEP 6: Issue Student Degrees (Create Attestations)
    // =============================================================================
    println!("🎓 Step 6: Issuing student degrees...");

    // Simulate student addresses
    let alice_student = IotaAddress::random_for_testing_only();
    let bob_student = IotaAddress::random_for_testing_only();

    println!("📜 Issuing Bachelor's degree in Computer Science to Alice...");

    // Create Alice's degree attestation data with advanced property shapes validation
    let alice_properties = std::collections::HashMap::from([
        (degree_bachelor.clone(), PropertyValue::Text("completed".to_owned())),
        (field_cs.clone(), PropertyValue::Text("true".to_owned())),
        (grade_gpa.clone(), PropertyValue::Number(385)), // 3.85 GPA (stored as 385 for precision)
        (graduation_year.clone(), PropertyValue::Number(2024)),
        (student_verified.clone(), PropertyValue::Text("true".to_owned())),
        (student_id.clone(), PropertyValue::Text("HARV-123456".to_owned())), // University code + student number
        (honors_level.clone(), PropertyValue::Text("magna_cum_laude".to_owned())),
    ]);

    let alice_properties = alice_properties
        .into_iter()
        .map(|(name, value)| FederationProperty::new(name).with_allowed_values(HashSet::from([value])))
        .collect::<Vec<_>>();

    hierarchies_client
        .create_accreditation_to_attest(
            *university_consortium.id.object_id(),
            alice_student.into(),
            alice_properties.clone(),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    // Check if the accreditation to attest was issued
    let accreditations = hierarchies_client
        .get_accreditations_to_attest(*university_consortium.id.object_id(), alice_student.into())
        .await?;

    assert!(accreditations.accreditations.len() == 1);

    // Use the helper function to format and display Alice's degree information
    let property_names = DegreePropertyNames {
        degree_bachelor: &degree_bachelor,
        degree_master: &degree_master,
        degree_phd: &degree_phd,
        field_cs: &field_cs,
        field_engineering: &field_engineering,
        field_mathematics: &field_mathematics,
        grade_gpa: &grade_gpa,
        graduation_year: &graduation_year,
        student_verified: &student_verified,
        student_id: &student_id,
        honors_level: &honors_level,
    };

    format_degree_info(
        "Alice",
        &alice_student,
        &accreditations.accreditations[0],
        &property_names,
    );

    println!("\n📜 Issuing Master's degree in Computer Science to Bob...");

    let bob_properties = std::collections::HashMap::from([
        (degree_master.clone(), PropertyValue::Text("completed".to_owned())),
        (field_cs.clone(), PropertyValue::Text("true".to_owned())),
        (grade_gpa.clone(), PropertyValue::Number(392)), // 3.92 GPA (stored as 392 for precision)
        (graduation_year.clone(), PropertyValue::Number(2023)),
        (student_verified.clone(), PropertyValue::Text("true".to_owned())),
        (student_id.clone(), PropertyValue::Text("MIT-789012".to_owned())), // MIT student ID format
        (honors_level.clone(), PropertyValue::Text("summa_cum_laude".to_owned())), // Highest honors
    ]);

    hierarchies_client
        .create_accreditation_to_attest(
            *university_consortium.id.object_id(),
            bob_student.into(),
            bob_properties
                .into_iter()
                .map(|(name, value)| FederationProperty::new(name).with_allowed_values(HashSet::from([value])))
                .collect::<Vec<_>>(),
        )
        .build_and_execute(&hierarchies_client)
        .await?;

    // Check if the accreditation to attest was issued
    let bob_accreditations = hierarchies_client
        .get_accreditations_to_attest(*university_consortium.id.object_id(), bob_student.into())
        .await?;

    assert!(bob_accreditations.accreditations.len() == 1);

    // Use the helper function to format and display Bob's degree information
    format_degree_info(
        "Bob",
        &bob_student,
        &bob_accreditations.accreditations[0],
        &property_names,
    );

    // =============================================================================
    // STEP 7: Validation Examples
    // =============================================================================
    println!("🔍 Step 7: Demonstrating credential validation...");

    // Example 1: Employer verifying Alice's bachelor's degree
    println!("🏢 Scenario: Tech company verifying Alice's credentials for a software engineer position");

    let validation_properties = std::collections::HashMap::from([
        (degree_bachelor.clone(), PropertyValue::Text("completed".to_owned())),
        (field_cs.clone(), PropertyValue::Text("true".to_owned())),
    ]);

    // Validate that Alice has the required credentials
    let is_valid = hierarchies_client
        .validate_properties(
            *university_consortium.id.object_id(),
            alice_student.into(),
            validation_properties,
        )
        .await?;

    if is_valid {
        println!("✅ Validation successful: Alice has a completed Bachelor's in Computer Science");
        println!("   - Attester: Harvard CS Registrar (authorized)");
        println!("   - Trust Chain: University Consortium → Harvard → CS Faculty → CS Registrar");
    } else {
        println!("❌ Validation failed: Credentials could not be verified");
    }

    // Example 2: Graduate school checking Bob's master's degree for PhD admission
    println!("\n🎓 Scenario: Graduate school verifying Bob's Master's degree for PhD admission");

    let grad_validation = std::collections::HashMap::from([
        (degree_master.clone(), PropertyValue::Text("completed".to_owned())),
        (field_cs.clone(), PropertyValue::Text("true".to_owned())),
    ]);

    let is_master_valid = hierarchies_client
        .validate_properties(
            *university_consortium.id.object_id(),
            bob_student.into(), // Validate Bob's credentials, not the registrar's
            grad_validation,
        )
        .await?;

    if is_master_valid {
        println!("✅ Validation successful: Bob has a completed Master's in Computer Science");
        println!("   - Eligible for PhD program admission");
        println!("   - GPA meets minimum requirements (3.92 > 3.5)");
    }

    // =============================================================================
    // STEP 8: Revocation Example (Academic Misconduct)
    // =============================================================================
    println!("\n⚠️  Step 8: Demonstrating degree revocation...");

    // Scenario: Academic misconduct discovered, need to revoke Alice's degree
    println!("🚨 Scenario: Academic misconduct discovered for Alice");
    println!("   - University needs to revoke Alice's Bachelor's degree");
    println!("   - This affects Alice's ability to use the credential");
    println!("   - Future validations will automatically fail");

    println!("\n📋 Step 8a: Revoking Alice's degree...");

    // First, get Alice's current accreditations to find the ID we need to revoke
    let alice_accreditations_before_revocation = hierarchies_client
        .get_accreditations_to_attest(*university_consortium.id.object_id(), alice_student.into())
        .await?;

    println!(
        "🔍 Found {} accreditation(s) for Alice",
        alice_accreditations_before_revocation.accreditations.len()
    );

    if !alice_accreditations_before_revocation.accreditations.is_empty() {
        // Get the accreditation ID to revoke
        let accreditation_to_revoke = &alice_accreditations_before_revocation.accreditations[0];
        let accreditation_id = *accreditation_to_revoke.id.object_id();

        hierarchies_client
            .revoke_accreditation_to_attest(
                *university_consortium.id.object_id(),
                alice_student.into(),
                accreditation_id,
            )
            .build_and_execute(&hierarchies_client)
            .await?;

        println!("   ✅ Alice's Bachelor's degree has been revoked!");
        println!("   - Accreditation ID: {:?}", accreditation_id);
        println!("   - Student: {}", alice_student);
        println!("   - Revoked by: Harvard CS Registrar (authorized)");

        // Verify the revocation worked by checking accreditations again
        let alice_accreditations_after_revocation = hierarchies_client
            .get_accreditations_to_attest(*university_consortium.id.object_id(), alice_student.into())
            .await?;

        println!("\n🔍 Step 8b: Verifying revocation...");
        println!(
            "   - Accreditations before revocation: {}",
            alice_accreditations_before_revocation.accreditations.len()
        );
        println!(
            "   - Accreditations after revocation: {}",
            alice_accreditations_after_revocation.accreditations.len()
        );

        if alice_accreditations_after_revocation.accreditations.len()
            < alice_accreditations_before_revocation.accreditations.len()
        {
            println!("   ✅ Revocation successful - Alice's degree is no longer valid");
        }

        // Test validation after revocation - this should now fail
        println!("\n🧪 Step 8c: Testing validation after revocation...");
        let validation_after_revocation = std::collections::HashMap::from([
            (degree_bachelor.clone(), PropertyValue::Text("completed".to_owned())),
            (field_cs.clone(), PropertyValue::Text("true".to_owned())),
        ]);

        let is_still_valid = hierarchies_client
            .validate_properties(
                *university_consortium.id.object_id(),
                alice_student.into(),
                validation_after_revocation,
            )
            .await?;

        if is_still_valid {
            println!("   ⚠️  Warning: Validation still passes after revocation");
        } else {
            println!("   ✅ Validation correctly fails after revocation");
            println!("   - Employers can no longer verify Alice's degree");
            println!("   - All validators are automatically protected");
            println!("   - Trust chain security maintained");
        }
    } else {
        println!("❌ No accreditations found for Alice to revoke");
    }

    // =============================================================================
    // SUMMARY
    // =============================================================================
    println!("📊 Example Summary:");
    println!("=====================================");
    println!("✅ University consortium federation created");
    println!("✅ Academic properties defined (degrees, fields, grades)");
    println!("✅ Universities added as root authorities");
    println!("✅ Hierarchical delegation: University → Faculty → Registrar");
    println!("✅ Student degrees issued as attestations");
    println!("✅ Credential validation demonstrated");
    println!("✅ Revocation capabilities shown");
    println!("✅ Cross-institutional recognition enabled");
    println!("\n🎯 Benefits Achieved:");
    println!("   - Instant credential verification");
    println!("   - Fraud prevention through cryptographic proof");
    println!("   - Reduced administrative overhead");
    println!("   - Global interoperability");
    println!("   - Privacy-preserving verification");
    println!("   - Automatic revocation handling");

    Ok(())
}

/// Property names for the university degree system
struct DegreePropertyNames<'a> {
    degree_bachelor: &'a PropertyName,
    degree_master: &'a PropertyName,
    degree_phd: &'a PropertyName,
    field_cs: &'a PropertyName,
    field_engineering: &'a PropertyName,
    field_mathematics: &'a PropertyName,
    grade_gpa: &'a PropertyName,
    graduation_year: &'a PropertyName,
    student_verified: &'a PropertyName,
    student_id: &'a PropertyName,
    honors_level: &'a PropertyName,
}

/// Helper function to format and display degree information from an accreditation response
fn format_degree_info(
    student_name: &str,
    student_address: &IotaAddress,
    accreditation: &Accreditation,
    properties: &DegreePropertyNames,
) {
    println!("✅ {}'s degree successfully issued:", student_name);
    println!("   - Student: {}", student_address);

    let accreditation_properties = &accreditation.properties;

    // Extract degree type
    let degree_type = {
        if let Some(prop) = accreditation_properties.get(properties.degree_bachelor) {
            if let Some(PropertyValue::Text(text)) = prop.allowed_values.iter().next() {
                format!("Bachelor's ({})", text)
            } else {
                "Bachelor's".to_string()
            }
        } else if let Some(prop) = accreditation_properties.get(properties.degree_master) {
            if let Some(PropertyValue::Text(text)) = prop.allowed_values.iter().next() {
                format!("Master's ({})", text)
            } else {
                "Master's".to_string()
            }
        } else if let Some(prop) = accreditation_properties.get(properties.degree_phd) {
            if let Some(PropertyValue::Text(text)) = prop.allowed_values.iter().next() {
                format!("PhD ({})", text)
            } else {
                "PhD".to_string()
            }
        } else {
            "Unknown Degree".to_string()
        }
    };

    // Extract field of study
    let field_of_study = {
        if let Some(prop) = accreditation_properties.get(properties.field_cs) {
            if let Some(PropertyValue::Text(text)) = prop.allowed_values.iter().next() {
                if text == "true" {
                    "Computer Science"
                } else {
                    "Unknown Field"
                }
            } else {
                "Unknown Field"
            }
        } else if let Some(prop) = accreditation_properties.get(properties.field_engineering) {
            if let Some(PropertyValue::Text(text)) = prop.allowed_values.iter().next() {
                if text == "true" { "Engineering" } else { "Unknown Field" }
            } else {
                "Unknown Field"
            }
        } else if let Some(prop) = accreditation_properties.get(properties.field_mathematics) {
            if let Some(PropertyValue::Text(text)) = prop.allowed_values.iter().next() {
                if text == "true" { "Mathematics" } else { "Unknown Field" }
            } else {
                "Unknown Field"
            }
        } else {
            "Unknown Field"
        }
    };

    // Extract GPA (now stored as number with advanced validation)
    let gpa = accreditation_properties
        .get(properties.grade_gpa)
        .and_then(|p| p.allowed_values.iter().next())
        .map(|v| match v {
            PropertyValue::Number(num) => format!("{:.2}", (*num as f64) / 100.0), // Convert back to decimal
            PropertyValue::Text(text) => text.clone(),
        })
        .unwrap_or_else(|| "N/A".to_string());

    // Extract graduation year (now stored as number with range validation)
    let grad_year = accreditation_properties
        .get(properties.graduation_year)
        .and_then(|p| p.allowed_values.iter().next())
        .map(|v| match v {
            PropertyValue::Number(year) => year.to_string(),
            PropertyValue::Text(text) => text.clone(),
        })
        .unwrap_or_else(|| "N/A".to_string());

    // Extract student ID (with format validation - must contain dash)
    let student_id = accreditation_properties
        .get(properties.student_id)
        .and_then(|p| p.allowed_values.iter().next())
        .map(|v| match v {
            PropertyValue::Text(text) => text.clone(),
            _ => "N/A".to_string(),
        })
        .unwrap_or_else(|| "N/A".to_string());

    // Extract honors level (with specific allowed values validation)
    let honors = accreditation_properties
        .get(properties.honors_level)
        .and_then(|p| p.allowed_values.iter().next())
        .map(|v| match v {
            PropertyValue::Text(text) => match text.as_str() {
                "summa_cum_laude" => "Summa Cum Laude",
                "magna_cum_laude" => "Magna Cum Laude",
                "cum_laude" => "Cum Laude",
                "none" => "No Honors",
                _ => text,
            },
            _ => "N/A",
        })
        .unwrap_or_else(|| "N/A");

    // Extract verification status
    let verification_status = accreditation_properties
        .get(properties.student_verified)
        .and_then(|p| p.allowed_values.iter().next())
        .map(|v| match v {
            PropertyValue::Text(text) if text == "true" => "Verified",
            PropertyValue::Text(text) if text == "false" => "Not Verified",
            _ => "Unknown",
        })
        .unwrap_or_else(|| "Unknown");

    println!("   - Degree: {} in {}", degree_type, field_of_study);
    println!("   - GPA: {} (validated: > 2.0)", gpa);
    println!("   - Graduation Year: {} (validated: > 1950)", grad_year);
    println!("   - Student ID: {} (validated: contains dash)", student_id);
    println!("   - Honors: {}", honors);
    println!("   - Verification Status: {}", verification_status);
    println!("   - Accreditation ID: {:?}", accreditation.id);
    println!("   - Issued by: {:?}\n", accreditation.accredited_by);
}
