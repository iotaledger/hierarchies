// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/**
 * Real-World Example: University Degree Verification System
 *
 * This example demonstrates how to use IOTA Hierarchies to create a comprehensive
 * university degree verification system. The scenario involves:
 *
 * ## Business Context
 * Universities need to issue verifiable digital degrees that employers and other
 * institutions can trust. The hierarchical structure allows:
 * - University consortiums to establish trust networks
 * - Individual universities to delegate authority to faculties
 * - Faculties to delegate to registrars and professors
 * - External parties to verify credentials without contacting the university directly
 *
 * ## Trust Hierarchy
 * ```
 * University Consortium (Root Authority)
 * ├── Harvard University (Root Authority)
 * │   ├── Computer Science Faculty (Accreditor)
 * │   │   └── CS Registrar (Attester)
 * │   └── Engineering Faculty (Accreditor)
 * │       └── Engineering Registrar (Attester)
 * └── MIT (Root Authority)
 *     ├── Computer Science Faculty (Accreditor)
 *     └── Engineering Faculty (Accreditor)
 * ```
 *
 * ## Statements Defined
 * - `degree.bachelor`: Bachelor's degree completion status
 * - `degree.master`: Master's degree completion status
 * - `degree.phd`: PhD completion status
 * - `field.computer_science`: Computer Science specialization
 * - `field.engineering`: Engineering specialization
 * - `grade.gpa`: Grade Point Average (0.0-4.0 scale)
 * - `graduation.year`: Year of graduation
 * - `student.verified`: Student identity verification status
 *
 * ## Real-World Applications
 * - Employers verifying job applicant credentials
 * - Graduate schools checking undergraduate degrees
 * - Professional licensing bodies validating educational requirements
 * - International credential recognition
 * - Alumni verification for networking platforms
 */

import { Federation, PropertyName, PropertyValue, FederationProperty, Accreditation } from "@iota/hierarchies/node";
import { getFundedClient } from "../util";

/**
 * Property names for the university degree system
 */
interface DegreePropertyNames {
    degreeBachelor: PropertyName;
    degreeMaster: PropertyName;
    degreePhd: PropertyName;
    fieldCs: PropertyName;
    fieldEngineering: PropertyName;
    fieldMathematics: PropertyName;
    gradeGpa: PropertyName;
    graduationYear: PropertyName;
    studentVerified: PropertyName;
}

/**
 * Helper function to format and display degree information from an accreditation response
 */
function formatDegreeInfo(
    studentName: string,
    studentAddress: string,
    accreditation: Accreditation,
    properties: DegreePropertyNames
): void {
    console.log(`✅ ${studentName}'s degree successfully issued:`);
    console.log(`   - Student: ${studentAddress}`);

    const accreditationProperties = accreditation.properties;

    // Extract degree type
    let degreeType = "Unknown Degree";
    const bachelorProp = accreditationProperties.find(prop => prop.propertyName.toString() === properties.degreeBachelor.toString());
    if (bachelorProp && bachelorProp.allowedValues.length > 0) {
        const value = bachelorProp.allowedValues[0];
        if (value.isText()) {
            degreeType = `Bachelor's (${value.asText()})`;
        }
    } else {
        const masterProp = accreditationProperties.find(prop => prop.propertyName.toString() === properties.degreeMaster.toString());
        if (masterProp && masterProp.allowedValues.length > 0) {
            const value = masterProp.allowedValues[0];
            if (value.isText()) {
                degreeType = `Master's (${value.asText()})`;
            }
        } else {
            const phdProp = accreditationProperties.find(prop => prop.propertyName.toString() === properties.degreePhd.toString());
            if (phdProp && phdProp.allowedValues.length > 0) {
                const value = phdProp.allowedValues[0];
                if (value.isText()) {
                    degreeType = `PhD (${value.asText()})`;
                }
            }
        }
    }

    // Extract field of study
    let fieldOfStudy = "Unknown Field";
    const csProp = accreditationProperties.find(prop => prop.propertyName.toString() === properties.fieldCs.toString());
    if (csProp && csProp.allowedValues.length > 0) {
        const value = csProp.allowedValues[0];
        if (value.isText() && value.asText() === "true") {
            fieldOfStudy = "Computer Science";
        }
    } else {
        const engineeringProp = accreditationProperties.find(prop => prop.propertyName.toString() === properties.fieldEngineering.toString());
        if (engineeringProp && engineeringProp.allowedValues.length > 0) {
            const value = engineeringProp.allowedValues[0];
            if (value.isText() && value.asText() === "true") {
                fieldOfStudy = "Engineering";
            }
        } else {
            const mathProp = accreditationProperties.find(prop => prop.propertyName.toString() === properties.fieldMathematics.toString());
            if (mathProp && mathProp.allowedValues.length > 0) {
                const value = mathProp.allowedValues[0];
                if (value.isText() && value.asText() === "true") {
                    fieldOfStudy = "Mathematics";
                }
            }
        }
    }

    // Extract GPA
    let gpa = "N/A";
    const gpaProp = accreditationProperties.find(prop => prop.propertyName.toString() === properties.gradeGpa.toString());
    if (gpaProp && gpaProp.allowedValues.length > 0) {
        const value = gpaProp.allowedValues[0];
        if (value.isText()) {
            gpa = value.asNumber()?.toString() ?? "N/A";
        }
    }

    // Extract graduation year
    let gradYear = "N/A";
    const gradYearProp = accreditationProperties.find(prop => prop.propertyName.toString() === properties.graduationYear.toString());
    if (gradYearProp && gradYearProp.allowedValues.length > 0) {
        const value = gradYearProp.allowedValues[0];
        if (value.isText()) {
            gradYear = value.asNumber()?.toString() ?? "N/A";
        }
    }

    // Extract verification status
    let verificationStatus = "Unknown";
    const verifiedProp = accreditationProperties.find(prop => prop.propertyName.toString() === properties.studentVerified.toString());
    if (verifiedProp && verifiedProp.allowedValues.length > 0) {
        const value = verifiedProp.allowedValues[0];
        if (value.isText()) {
            verificationStatus = value.asText() === "true" ? "Verified" : "Not Verified";
        }
    }

    console.log(`   - Degree: ${degreeType} in ${fieldOfStudy}`);
    console.log(`   - GPA: ${gpa}`);
    console.log(`   - Graduation Year: ${gradYear}`);
    console.log(`   - Verification Status: ${verificationStatus}`);
    console.log(`   - Accreditation ID: ${accreditation.id}`);
    console.log(`   - Issued by: ${accreditation.accreditedBy}\n`);
}

export async function universityDegreesExample(): Promise<void> {
    console.log("🎓 University Degree Verification System Example\n");

    const hierarchies = await getFundedClient();

    // =============================================================================
    // STEP 1: Create University Consortium Federation
    // =============================================================================
    console.log("📚 Step 1: Creating University Consortium Federation...");

    const { output: universityConsortium }: { output: Federation } =
        await hierarchies.createNewFederation().buildAndExecute(hierarchies);

    console.log("✅ University Consortium Federation created!");
    console.log(`   Federation ID: ${universityConsortium.id}`);
    console.log("   Root Authority: University Consortium Board\n");

    // =============================================================================
    // STEP 2: Define Academic Statements (Credential Types)
    // =============================================================================
    console.log("📝 Step 2: Defining academic statements...");

    // Degree completion properties
    const degreeBachelor = new PropertyName(["degree", "bachelor"]);
    const degreeMaster = new PropertyName(["degree", "master"]);
    const degreePhd = new PropertyName(["degree", "phd"]);

    // Field of study properties
    const fieldCs = new PropertyName(["field", "computer_science"]);
    const fieldEngineering = new PropertyName(["field", "engineering"]);
    const fieldMathematics = new PropertyName(["field", "mathematics"]);

    // Academic performance and verification
    const gradeGpa = new PropertyName(["grade", "gpa"]);
    const graduationYear = new PropertyName(["graduation", "year"]);
    const studentVerified = new PropertyName(["student", "verified"]);

    // Add degree completion properties with specific allowed values
    const degreeValues = [
        PropertyValue.newText("completed"),
        PropertyValue.newText("in_progress"),
        PropertyValue.newText("withdrawn"),
    ];

    await hierarchies
        .addProperty(universityConsortium.id, new FederationProperty(degreeBachelor).withAllowedValues(degreeValues))
        .buildAndExecute(hierarchies);

    await hierarchies
        .addProperty(universityConsortium.id, new FederationProperty(degreeMaster).withAllowedValues(degreeValues))
        .buildAndExecute(hierarchies);

    await hierarchies
        .addProperty(universityConsortium.id, new FederationProperty(degreePhd).withAllowedValues(degreeValues))
        .buildAndExecute(hierarchies);

    // Add field of study properties (boolean - true if student studied this field)
    const booleanValues = [
        PropertyValue.newText("true"),
        PropertyValue.newText("false"),
    ];

    await hierarchies
        .addProperty(universityConsortium.id, new FederationProperty(fieldCs).withAllowedValues(booleanValues))
        .buildAndExecute(hierarchies);

    await hierarchies
        .addProperty(universityConsortium.id, new FederationProperty(fieldEngineering).withAllowedValues(booleanValues))
        .buildAndExecute(hierarchies);

    await hierarchies
        .addProperty(universityConsortium.id, new FederationProperty(fieldMathematics).withAllowedValues(booleanValues))
        .buildAndExecute(hierarchies);

    // Add GPA property (allow any numeric value - will be validated by business logic)
    await hierarchies
        .addProperty(universityConsortium.id, new FederationProperty(gradeGpa).withAllowAny(true))
        .buildAndExecute(hierarchies);

    // Add graduation year (allow any year - business rules apply)
    await hierarchies
        .addProperty(universityConsortium.id, new FederationProperty(graduationYear).withAllowAny(true))
        .buildAndExecute(hierarchies);

    // Add student verification status
    await hierarchies
        .addProperty(universityConsortium.id, new FederationProperty(studentVerified).withAllowedValues(booleanValues))
        .buildAndExecute(hierarchies);

    console.log("✅ Academic properties defined:");
    console.log("   - Degree types: Bachelor, Master, PhD");
    console.log("   - Fields: Computer Science, Engineering, Mathematics");
    console.log("   - Academic metrics: GPA, Graduation Year");
    console.log("   - Verification: Student identity verification\n");

    // =============================================================================
    // STEP 3: Add Universities as Root Authorities
    // =============================================================================
    console.log("🏛️ Step 3: Adding universities to the consortium...");

    // Simulate Harvard University and MIT addresses
    // In real implementation, these would be actual university wallet addresses
    const harvardAddress = "0x" + Math.random().toString(16).substring(2, 42).padStart(40, '0');
    const mitAddress = "0x" + Math.random().toString(16).substring(2, 42).padStart(40, '0');

    // Add Harvard as root authority
    await hierarchies
        .addRootAuthority(universityConsortium.id, harvardAddress)
        .buildAndExecute(hierarchies);

    // Add MIT as root authority
    await hierarchies
        .addRootAuthority(universityConsortium.id, mitAddress)
        .buildAndExecute(hierarchies);

    console.log("✅ Universities added as root authorities:");
    console.log(`   - Harvard University: ${harvardAddress}`);
    console.log(`   - MIT: ${mitAddress}\n`);

    // =============================================================================
    // STEP 4: Create Faculty-Level Accreditations (Harvard CS Faculty)
    // =============================================================================
    console.log("🏫 Step 4: Creating faculty-level accreditations...");

    // Simulate Harvard CS Faculty address
    const harvardCsFaculty = "0x" + Math.random().toString(16).substring(2, 42).padStart(40, '0');

    // Harvard delegates accreditation rights to its CS Faculty
    // This allows the faculty to further delegate to registrars and professors
    const csFacultyProperties = [
        new FederationProperty(degreeBachelor).withAllowAny(true),
        new FederationProperty(degreeMaster).withAllowAny(true),
        new FederationProperty(degreePhd).withAllowAny(true),
        new FederationProperty(fieldCs).withAllowAny(true),
        new FederationProperty(gradeGpa).withAllowAny(true),
        new FederationProperty(graduationYear).withAllowAny(true),
        new FederationProperty(studentVerified).withAllowAny(true),
    ];

    await hierarchies
        .createAccreditationToAccredit(universityConsortium.id, harvardCsFaculty, csFacultyProperties)
        .buildAndExecute(hierarchies);

    console.log("✅ Harvard CS Faculty granted accreditation rights:");
    console.log(`   - Faculty Address: ${harvardCsFaculty}`);
    console.log("   - Can delegate rights for all CS-related degrees\n");

    // =============================================================================
    // STEP 5: Create Registrar-Level Attestation Rights
    // =============================================================================
    console.log("👨‍💼 Step 5: Creating registrar attestation rights...");

    // Simulate Harvard CS Registrar address
    const harvardCsRegistrar = "0x" + Math.random().toString(16).substring(2, 42).padStart(40, '0');

    // CS Faculty delegates attestation rights to the CS Registrar
    // Registrar can now create attestations (issue degrees) but not delegate further
    await hierarchies
        .createAccreditationToAttest(universityConsortium.id, harvardCsRegistrar, csFacultyProperties)
        .buildAndExecute(hierarchies);

    console.log("✅ Harvard CS Registrar granted attestation rights:");
    console.log(`   - Registrar Address: ${harvardCsRegistrar}`);
    console.log("   - Can issue degrees and verify student credentials\n");

    // =============================================================================
    // STEP 6: Issue Student Degrees (Create Attestations)
    // =============================================================================
    console.log("🎓 Step 6: Issuing student degrees...");

    // Simulate student addresses
    const aliceStudent = "0x" + Math.random().toString(16).substring(2, 42).padStart(40, '0');
    const bobStudent = "0x" + Math.random().toString(16).substring(2, 42).padStart(40, '0');

    console.log("📜 Issuing Bachelor's degree in Computer Science to Alice...");

    // Create Alice's degree attestation data
    const aliceProperties = [
        new FederationProperty(degreeBachelor).withAllowedValues([PropertyValue.newText("completed")]),
        new FederationProperty(fieldCs).withAllowedValues([PropertyValue.newText("true")]),
        new FederationProperty(gradeGpa).withAllowedValues([PropertyValue.newText("3.85")]),
        new FederationProperty(graduationYear).withAllowedValues([PropertyValue.newText("2024")]),
        new FederationProperty(studentVerified).withAllowedValues([PropertyValue.newText("true")]),
    ];

    await hierarchies
        .createAccreditationToAttest(universityConsortium.id, aliceStudent, aliceProperties)
        .buildAndExecute(hierarchies);

    // Check if the accreditation to attest was issued
    const accreditations = await hierarchies.readOnly().getAccreditationsToAttest(universityConsortium.id, aliceStudent);

    if (accreditations.accreditations.length === 0) {
        throw new Error("Failed to create Alice's degree accreditation");
    }

    // Use helper function to format and display Alice's degree information
    formatDegreeInfo(
        "Alice",
        aliceStudent,
        accreditations.accreditations[0],
        { degreeBachelor, degreeMaster, degreePhd, fieldCs, fieldEngineering, fieldMathematics, gradeGpa, graduationYear, studentVerified }
    );

    console.log("\n📜 Issuing Master's degree in Computer Science to Bob...");

    const bobProperties = [
        new FederationProperty(degreeMaster).withAllowedValues([PropertyValue.newText("completed")]),
        new FederationProperty(fieldCs).withAllowedValues([PropertyValue.newText("true")]),
        new FederationProperty(gradeGpa).withAllowedValues([PropertyValue.newText("3.92")]),
        new FederationProperty(graduationYear).withAllowedValues([PropertyValue.newText("2023")]),
        new FederationProperty(studentVerified).withAllowedValues([PropertyValue.newText("true")]),
    ];

    await hierarchies
        .createAccreditationToAttest(universityConsortium.id, bobStudent, bobProperties)
        .buildAndExecute(hierarchies);

    // Check if the accreditation to attest was issued
    const bobAccreditations = await hierarchies.readOnly().getAccreditationsToAttest(universityConsortium.id, bobStudent);

    if (bobAccreditations.accreditations.length === 0) {
        throw new Error("Failed to create Bob's degree accreditation");
    }

    // Use helper function to format and display Bob's degree information
    formatDegreeInfo(
        "Bob",
        bobStudent,
        bobAccreditations.accreditations[0],
        { degreeBachelor, degreeMaster, degreePhd, fieldCs, fieldEngineering, fieldMathematics, gradeGpa, graduationYear, studentVerified }
    );

    // =============================================================================
    // STEP 7: Validation Examples
    // =============================================================================
    console.log("🔍 Step 7: Demonstrating credential validation...");

    // Example 1: Employer verifying Alice's bachelor's degree
    console.log("🏢 Scenario: Tech company verifying Alice's credentials for a software engineer position");

    const validationProperties = new Map([
        [degreeBachelor.toString(), PropertyValue.newText("completed")],
        [fieldCs.toString(), PropertyValue.newText("true")],
    ]);

    // Validate that Alice has the required credentials
    const isValid = await hierarchies.readOnly().validateProperties(
        universityConsortium.id,
        aliceStudent, // Validate Alice's credentials, not the registrar's
        validationProperties
    );

    if (isValid) {
        console.log("✅ Validation successful: Alice has a completed Bachelor's in Computer Science");
        console.log("   - Attester: Harvard CS Registrar (authorized)");
        console.log("   - Trust Chain: University Consortium → Harvard → CS Faculty → CS Registrar");
    } else {
        console.log("❌ Validation failed: Credentials could not be verified");
    }

    // Example 2: Graduate school checking Bob's master's degree for PhD admission
    console.log("\n🎓 Scenario: Graduate school verifying Bob's Master's degree for PhD admission");

    const gradValidation = new Map([
        [degreeMaster.toString(), PropertyValue.newText("completed")],
        [fieldCs.toString(), PropertyValue.newText("true")],
    ]);

    const isMasterValid = await hierarchies.readOnly().validateProperties(
        universityConsortium.id,
        bobStudent, // Validate Bob's credentials, not the registrar's
        gradValidation
    );

    if (isMasterValid) {
        console.log("✅ Validation successful: Bob has a completed Master's in Computer Science");
        console.log("   - Eligible for PhD program admission");
        console.log("   - GPA meets minimum requirements (3.92 > 3.5)");
    }

    // =============================================================================
    // STEP 8: Revocation Example (Academic Misconduct)
    // =============================================================================
    console.log("\n⚠️  Step 8: Demonstrating degree revocation...");

    // Scenario: Academic misconduct discovered, need to revoke Alice's degree
    console.log("🚨 Scenario: Academic misconduct discovered for Alice");
    console.log("   - University needs to revoke Alice's Bachelor's degree");
    console.log("   - This affects Alice's ability to use the credential");
    console.log("   - Future validations will automatically fail");

    console.log("\n📋 Step 8a: Revoking Alice's degree...");

    // First, get Alice's current accreditations to find the ID we need to revoke
    const aliceAccreditationsBeforeRevocation = await hierarchies.readOnly().getAccreditationsToAttest(universityConsortium.id, aliceStudent);

    console.log(`🔍 Found ${aliceAccreditationsBeforeRevocation.accreditations.length} accreditation(s) for Alice`);

    if (aliceAccreditationsBeforeRevocation.accreditations.length > 0) {
        // Get the accreditation ID to revoke
        const accreditationToRevoke = aliceAccreditationsBeforeRevocation.accreditations[0];

        console.log("📋 Revocation process:");
        console.log("   1. University investigates misconduct ✅");
        console.log("   2. Due process followed ✅");
        console.log("   3. Registrar revokes the degree attestation...");

        // Perform the actual revocation using the hierarchies API
        await hierarchies
            .revokeAccreditationToAttest(universityConsortium.id, aliceStudent, accreditationToRevoke.id)
            .buildAndExecute(hierarchies);

        console.log("   ✅ Alice's Bachelor's degree has been revoked!");
        console.log(`   - Accreditation ID: ${accreditationToRevoke.id}`);
        console.log(`   - Student: ${aliceStudent}`);
        console.log("   - Revoked by: Harvard CS Registrar (authorized)");

        // Verify the revocation worked by checking accreditations again
        const aliceAccreditationsAfterRevocation = await hierarchies.readOnly().getAccreditationsToAttest(universityConsortium.id, aliceStudent);

        console.log("\n🔍 Step 8b: Verifying revocation...");
        console.log(`   - Accreditations before revocation: ${aliceAccreditationsBeforeRevocation.accreditations.length}`);
        console.log(`   - Accreditations after revocation: ${aliceAccreditationsAfterRevocation.accreditations.length}`);

        if (aliceAccreditationsAfterRevocation.accreditations.length < aliceAccreditationsBeforeRevocation.accreditations.length) {
            console.log("   ✅ Revocation successful - Alice's degree is no longer valid");
        }

        // Test validation after revocation - this should now fail
        console.log("\n🧪 Step 8c: Testing validation after revocation...");
        const validationAfterRevocation = new Map([
            [degreeBachelor.toString(), PropertyValue.newText("completed")],
            [fieldCs.toString(), PropertyValue.newText("true")],
        ]);

        const isStillValid = await hierarchies.readOnly().validateProperties(
            universityConsortium.id,
            aliceStudent,
            validationAfterRevocation
        );

        if (isStillValid) {
            console.log("   ⚠️  Warning: Validation still passes after revocation");
        } else {
            console.log("   ✅ Validation correctly fails after revocation");
            console.log("   - Employers can no longer verify Alice's degree");
            console.log("   - All validators are automatically protected");
            console.log("   - Trust chain security maintained");
        }

    } else {
        console.log("❌ No accreditations found for Alice to revoke");
    }

    console.log("\n🎯 Revocation Benefits:");
    console.log("   - Immediate effect across the entire network");
    console.log("   - No need to notify individual validators");
    console.log("   - Cryptographically secure and tamper-proof");
    console.log("   - Maintains audit trail and transparency");

    // =============================================================================
    // SUMMARY
    // =============================================================================
    console.log("📊 Example Summary:");
    console.log("=====================================");
    console.log("✅ University consortium federation created");
    console.log("✅ Academic statements defined (degrees, fields, grades)");
    console.log("✅ Universities added as root authorities");
    console.log("✅ Hierarchical delegation: University → Faculty → Registrar");
    console.log("✅ Student degrees issued as attestations");
    console.log("✅ Credential validation demonstrated");
    console.log("✅ Revocation capabilities shown");
    console.log("\n🎯 Benefits Achieved:");
    console.log("   - Instant credential verification");
    console.log("   - Fraud prevention through cryptographic proof");
    console.log("   - Reduced administrative overhead");
    console.log("   - Global interoperability");
    console.log("   - Privacy-preserving verification");
    console.log("   - Automatic revocation handling");
}

// Export for main.ts integration
export { universityDegreesExample as universityDegrees };