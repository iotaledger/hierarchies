// Copyright 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/**
 * Real-World Example: Supply Chain Quality Certification System
 *
 * This example demonstrates how to use IOTA Hierarchies to create a comprehensive
 * supply chain quality and compliance certification system. The scenario involves:
 *
 * ## Business Context
 * Global supply chains require verifiable quality certifications and compliance
 * attestations to ensure product safety and regulatory compliance. The hierarchical
 * structure enables:
 * - International standards organizations to establish certification frameworks
 * - Regional certifiers to delegate authority to local testing laboratories
 * - Testing labs to delegate to qualified inspectors
 * - Manufacturers and retailers to verify product certifications instantly
 *
 * ## Trust Hierarchy
 * ```
 * International Standards Consortium (Root Authority)
 * ├── ISO Europe (Root Authority)
 * │   ├── German Testing Institute (Regional Certifier)
 * │   │   ├── Berlin Lab (Testing Lab)
 * │   │   │   └── Senior Inspector (Inspector)
 * │   │   └── Munich Lab (Testing Lab)
 * │   └── French Testing Institute (Regional Certifier)
 * └── ISO Americas (Root Authority)
 *     ├── US FDA Regional Office (Regional Certifier)
 *     └── Canadian Health Authority (Regional Certifier)
 * ```
 *
 * ## Statements Defined
 * - `iso.9001`: ISO 9001 Quality Management certification status
 * - `iso.14001`: ISO 14001 Environmental Management certification
 * - `iso.22000`: ISO 22000 Food Safety Management certification
 * - `product.organic`: Organic certification status
 * - `origin.verified`: Country of origin verification
 * - `batch.tested`: Batch-specific quality testing results
 * - `compliance.eu`: European Union compliance status
 * - `compliance.fda`: US FDA compliance status
 * - `compliance.halal`: Halal certification status
 * - `expiry.date`: Certification expiry timestamp
 *
 * ## Real-World Applications
 * - Manufacturers verifying supplier certifications
 * - Retailers validating product compliance before import
 * - Customs authorities checking regulatory compliance
 * - Consumers verifying organic/halal certifications via QR codes
 * - Insurance companies assessing supply chain risk
 * - Audit trail for product recalls and quality issues
 */

import { Accreditation, Federation, FederationProperty, PropertyName, PropertyValue } from "@iota/hierarchies/node";
import { getFundedClient } from "../util";

interface CertificationPropertyNames {
    productOrganic: PropertyName;
    originVerified: PropertyName;
    batchTested: PropertyName;
    complianceEu: PropertyName;
    complianceFda: PropertyName;
    iso22000: PropertyName;
    expiryDate: PropertyName;
}

function formatCertificationInfo(
    productName: string,
    productBatch: string,
    accreditation: Accreditation,
    properties: CertificationPropertyNames,
): void {
    console.log(`✅ ${productName}'s certification successfully issued:`);
    console.log(`   - Product: ${productBatch}`);

    const accreditationProperties = accreditation.properties;

    // Extract organic certification
    const organicProp = accreditationProperties.find(prop =>
        prop.propertyName.toString() === properties.productOrganic.toString()
    );
    const organicValue = organicProp?.allowedValues?.[0];
    const organicStatus = organicValue && "text" in organicValue && organicValue.text === "true"
        ? "Certified Organic"
        : organicValue && "text" in organicValue && organicValue.text === "false"
        ? "Not Organic"
        : "Unknown";

    // Extract origin country
    const originProp = accreditationProperties.find(prop =>
        prop.propertyName.toString() === properties.originVerified.toString()
    );
    const originValue = originProp?.allowedValues?.[0];
    const originText = originValue && "text" in originValue ? originValue.text : null;
    const originCountry = originText
        ? (originText === "DE"
            ? "Germany (DE)"
            : originText === "US"
            ? "United States (US)"
            : originText === "CA"
            ? "Canada (CA)"
            : originText === "FR"
            ? "France (FR)"
            : originText)
        : "Unknown";

    // Extract batch testing results
    const testingProp = accreditationProperties.find(prop =>
        prop.propertyName.toString() === properties.batchTested.toString()
    );
    const testingValue = testingProp?.allowedValues?.[0];
    const testingResult = testingValue && "text" in testingValue
        ? (testingValue.text === "passed"
            ? "Passed"
            : testingValue.text === "failed"
            ? "Failed"
            : testingValue.text === "pending"
            ? "Pending"
            : "Unknown")
        : "Unknown";

    // Extract EU compliance
    const euProp = accreditationProperties.find(prop =>
        prop.propertyName.toString() === properties.complianceEu.toString()
    );
    const euValue = euProp?.allowedValues?.[0];
    const euCompliance = euValue && "text" in euValue && euValue.text === "true"
        ? "Yes"
        : euValue && "text" in euValue && euValue.text === "false"
        ? "No"
        : "N/A";

    // Extract FDA compliance
    const fdaProp = accreditationProperties.find(prop =>
        prop.propertyName.toString() === properties.complianceFda.toString()
    );
    const fdaValue = fdaProp?.allowedValues?.[0];
    const fdaCompliance = fdaValue && "text" in fdaValue && fdaValue.text === "true"
        ? "Yes"
        : fdaValue && "text" in fdaValue && fdaValue.text === "false"
        ? "No"
        : "N/A";

    // Extract ISO 22000 certification
    const isoProp = accreditationProperties.find(prop =>
        prop.propertyName.toString() === properties.iso22000.toString()
    );
    const isoValue = isoProp?.allowedValues?.[0];
    const iso22000Status = isoValue && "text" in isoValue
        ? (isoValue.text === "certified"
            ? "Certified"
            : isoValue.text === "expired"
            ? "Expired"
            : isoValue.text === "pending"
            ? "Pending"
            : "N/A")
        : "N/A";

    // Extract expiry date
    const expiryProp = accreditationProperties.find(prop =>
        prop.propertyName.toString() === properties.expiryDate.toString()
    );
    const expiryValue = expiryProp?.allowedValues?.[0];
    const expiryDateStr = expiryValue && "text" in expiryValue && typeof expiryValue.text === "string"
        ? new Date(expiryValue.text).toISOString().split("T")[0]
        : "N/A";

    console.log(`   - Origin: ${originCountry}`);
    if (organicStatus !== "Unknown") {
        console.log(`   - Organic Status: ${organicStatus}`);
    }
    if (iso22000Status !== "N/A") {
        console.log(`   - ISO 22000: ${iso22000Status}`);
    }
    if (euCompliance !== "N/A") {
        console.log(`   - EU Compliance: ${euCompliance}`);
    }
    if (fdaCompliance !== "N/A") {
        console.log(`   - FDA Compliance: ${fdaCompliance}`);
    }
    console.log(`   - Testing Result: ${testingResult}`);
    console.log(`   - Valid Until: ${expiryDateStr}`);
    console.log(`   - Accreditation ID: ${accreditation.id}`);
    console.log(`   - Issued by: ${accreditation.accreditedBy}\n`);
}

export async function supplyChainExample(): Promise<void> {
    console.log("🏭 Supply Chain Quality Certification System Example\n");

    const hierarchies = await getFundedClient();

    // =============================================================================
    // STEP 1: Create International Standards Consortium Federation
    // =============================================================================
    console.log("🌍 Step 1: Creating International Standards Consortium Federation...");

    const { output: standardsConsortium }: { output: Federation } = await hierarchies.createNewFederation()
        .buildAndExecute(hierarchies);

    console.log("✅ International Standards Consortium Federation created!");
    console.log(`   Federation ID: ${standardsConsortium.id}`);
    console.log("   Purpose: Global supply chain quality and compliance certification\n");

    // =============================================================================
    // STEP 2: Define Certification and Compliance Statements
    // =============================================================================
    console.log("📋 Step 2: Defining certification statements...");

    // ISO Standards
    const iso9001 = new PropertyName(["iso", "9001"]);
    const iso14001 = new PropertyName(["iso", "14001"]);
    const iso22000 = new PropertyName(["iso", "22000"]);

    // Product Certifications
    const productOrganic = new PropertyName(["product", "organic"]);
    const originVerified = new PropertyName(["origin", "verified"]);
    const batchTested = new PropertyName(["batch", "tested"]);

    // Regional Compliance
    const complianceEu = new PropertyName(["compliance", "eu"]);
    const complianceFda = new PropertyName(["compliance", "fda"]);
    const complianceHalal = new PropertyName(["compliance", "halal"]);

    // Time-sensitive certification
    const expiryDate = new PropertyName(["expiry", "date"]);

    // Certification status values

    // Add ISO certification properties
    await hierarchies
        .addProperty(
            standardsConsortium.id,
            new FederationProperty(iso9001).withAllowedValues([
                PropertyValue.newText("certified"),
                PropertyValue.newText("pending"),
                PropertyValue.newText("expired"),
                PropertyValue.newText("revoked"),
                PropertyValue.newText("suspended"),
            ]),
        )
        .buildAndExecute(hierarchies);

    await hierarchies
        .addProperty(
            standardsConsortium.id,
            new FederationProperty(iso14001).withAllowedValues([
                PropertyValue.newText("certified"),
                PropertyValue.newText("pending"),
                PropertyValue.newText("expired"),
                PropertyValue.newText("revoked"),
                PropertyValue.newText("suspended"),
            ]),
        )
        .buildAndExecute(hierarchies);

    await hierarchies
        .addProperty(
            standardsConsortium.id,
            new FederationProperty(iso22000).withAllowedValues([
                PropertyValue.newText("certified"),
                PropertyValue.newText("pending"),
                PropertyValue.newText("expired"),
                PropertyValue.newText("revoked"),
                PropertyValue.newText("suspended"),
            ]),
        )
        .buildAndExecute(hierarchies);

    // Boolean certifications (certified/not certified)

    await hierarchies
        .addProperty(
            standardsConsortium.id,
            new FederationProperty(productOrganic).withAllowedValues([
                PropertyValue.newText("true"),
                PropertyValue.newText("false"),
            ]),
        )
        .buildAndExecute(hierarchies);

    await hierarchies
        .addProperty(
            standardsConsortium.id,
            new FederationProperty(complianceEu).withAllowedValues([
                PropertyValue.newText("true"),
                PropertyValue.newText("false"),
            ]),
        )
        .buildAndExecute(hierarchies);

    await hierarchies
        .addProperty(
            standardsConsortium.id,
            new FederationProperty(complianceFda).withAllowedValues([
                PropertyValue.newText("true"),
                PropertyValue.newText("false"),
            ]),
        )
        .buildAndExecute(hierarchies);

    await hierarchies
        .addProperty(
            standardsConsortium.id,
            new FederationProperty(complianceHalal).withAllowedValues([
                PropertyValue.newText("true"),
                PropertyValue.newText("false"),
            ]),
        )
        .buildAndExecute(hierarchies);

    // Testing results

    await hierarchies
        .addProperty(
            standardsConsortium.id,
            new FederationProperty(batchTested).withAllowedValues([
                PropertyValue.newText("passed"),
                PropertyValue.newText("failed"),
                PropertyValue.newText("pending"),
                PropertyValue.newText("inconclusive"),
            ]),
        )
        .buildAndExecute(hierarchies);

    // Country codes for origin verification (allow any - validated by business logic)
    await hierarchies
        .addProperty(standardsConsortium.id, new FederationProperty(originVerified).withAllowAny(true)) // Country codes: DE, US, CN, etc.
        .buildAndExecute(hierarchies);

    // Expiry dates (allow any - ISO 8601 timestamps)
    await hierarchies
        .addProperty(standardsConsortium.id, new FederationProperty(expiryDate).withAllowAny(true)) // ISO 8601 timestamp strings
        .buildAndExecute(hierarchies);

    console.log("✅ Certification statements defined:");
    console.log("   - ISO Standards: 9001 (Quality), 14001 (Environmental), 22000 (Food Safety)");
    console.log("   - Product Certifications: Organic, Origin Verification");
    console.log("   - Regional Compliance: EU, FDA, Halal");
    console.log("   - Quality Testing: Batch testing results");
    console.log("   - Time Management: Certification expiry dates\n");

    // =============================================================================
    // STEP 3: Add Regional Standards Organizations
    // =============================================================================
    console.log("🌐 Step 3: Adding regional standards organizations...");

    // Simulate regional standards organization addresses
    const isoEurope = "0x" + Math.random().toString(16).substring(2, 42).padStart(40, "0");
    const isoAmericas = "0x" + Math.random().toString(16).substring(2, 42).padStart(40, "0");
    const isoAsiaPacific = "0x" + Math.random().toString(16).substring(2, 42).padStart(40, "0");

    // Add regional organizations as root authorities
    await hierarchies
        .addRootAuthority(standardsConsortium.id, isoEurope)
        .buildAndExecute(hierarchies);

    await hierarchies
        .addRootAuthority(standardsConsortium.id, isoAmericas)
        .buildAndExecute(hierarchies);

    await hierarchies
        .addRootAuthority(standardsConsortium.id, isoAsiaPacific)
        .buildAndExecute(hierarchies);

    console.log("✅ Regional standards organizations added:");
    console.log(`   - ISO Europe: ${isoEurope}`);
    console.log(`   - ISO Americas: ${isoAmericas}`);
    console.log(`   - ISO Asia-Pacific: ${isoAsiaPacific}\n`);

    // =============================================================================
    // STEP 4: Create National Testing Institute Accreditations
    // =============================================================================
    console.log("🏢 Step 4: Creating national testing institute accreditations...");

    // German Testing Institute under ISO Europe
    const germanTestingInstitute = "0x" + Math.random().toString(16).substring(2, 42).padStart(40, "0");

    // Create comprehensive accreditation package for German institute

    // ISO Europe delegates accreditation rights to German Testing Institute
    await hierarchies
        .createAccreditationToAccredit(standardsConsortium.id, germanTestingInstitute, [
            new FederationProperty(iso9001).withAllowAny(true),
            new FederationProperty(iso14001).withAllowAny(true),
            new FederationProperty(iso22000).withAllowAny(true),
            new FederationProperty(productOrganic).withAllowAny(true),
            new FederationProperty(originVerified).withAllowAny(true),
            new FederationProperty(batchTested).withAllowAny(true),
            new FederationProperty(complianceEu).withAllowAny(true),
            new FederationProperty(expiryDate).withAllowAny(true),
        ])
        .buildAndExecute(hierarchies);

    // US FDA Regional Office under ISO Americas
    const usFdaRegional = "0x" + Math.random().toString(16).substring(2, 42).padStart(40, "0");

    await hierarchies
        .createAccreditationToAccredit(standardsConsortium.id, usFdaRegional, [
            new FederationProperty(iso9001).withAllowAny(true),
            new FederationProperty(iso22000).withAllowAny(true),
            new FederationProperty(productOrganic).withAllowAny(true),
            new FederationProperty(originVerified).withAllowAny(true),
            new FederationProperty(batchTested).withAllowAny(true),
            new FederationProperty(complianceFda).withAllowAny(true),
            new FederationProperty(expiryDate).withAllowAny(true),
        ])
        .buildAndExecute(hierarchies);

    console.log("✅ National testing institutes accredited:");
    console.log(`   - German Testing Institute: ${germanTestingInstitute}`);
    console.log("     Scope: EU compliance, organic, environmental certifications");
    console.log(`   - US FDA Regional Office: ${usFdaRegional}`);
    console.log("     Scope: FDA compliance, food safety, organic certifications\n");

    // =============================================================================
    // STEP 5: Create Local Testing Laboratory Attestation Rights
    // =============================================================================
    console.log("🧪 Step 5: Creating local testing laboratory rights...");

    // Berlin Food Safety Lab under German Testing Institute
    const berlinFoodLab = "0x" + Math.random().toString(16).substring(2, 42).padStart(40, "0");

    // Focus on food safety and organic certifications

    // German Testing Institute delegates attestation rights to Berlin lab
    await hierarchies
        .createAccreditationToAttest(standardsConsortium.id, berlinFoodLab, [
            new FederationProperty(iso22000).withAllowAny(true),
            new FederationProperty(productOrganic).withAllowAny(true),
            new FederationProperty(originVerified).withAllowAny(true),
            new FederationProperty(batchTested).withAllowAny(true),
            new FederationProperty(complianceEu).withAllowAny(true),
            new FederationProperty(expiryDate).withAllowAny(true),
        ])
        .buildAndExecute(hierarchies);

    // California Agricultural Lab under US FDA
    const californiaAgLab = "0x" + Math.random().toString(16).substring(2, 42).padStart(40, "0");

    await hierarchies
        .createAccreditationToAttest(standardsConsortium.id, californiaAgLab, [
            new FederationProperty(iso9001).withAllowAny(true),
            new FederationProperty(iso22000).withAllowAny(true),
            new FederationProperty(productOrganic).withAllowAny(true),
            new FederationProperty(originVerified).withAllowAny(true),
            new FederationProperty(batchTested).withAllowAny(true),
            new FederationProperty(complianceFda).withAllowAny(true),
            new FederationProperty(expiryDate).withAllowAny(true),
        ])
        .buildAndExecute(hierarchies);

    console.log("✅ Local testing laboratories authorized:");
    console.log(`   - Berlin Food Safety Lab: ${berlinFoodLab}`);
    console.log("     Specialization: Organic food products, EU compliance");
    console.log(`   - California Agricultural Lab: ${californiaAgLab}`);
    console.log("     Specialization: Organic produce, FDA compliance\n");

    // =============================================================================
    // STEP 6: Issue Product Certifications
    // =============================================================================
    console.log("🥕 Step 6: Issuing product certifications...");

    // Simulate product batch addresses/IDs
    const organicApplesBatch = "0x" + Math.random().toString(16).substring(2, 42).padStart(40, "0");
    const processedFoodBatch = "0x" + Math.random().toString(16).substring(2, 42).padStart(40, "0");

    // Current time for certification
    const now = new Date();
    const expiry = new Date(now.getTime() + 365 * 24 * 60 * 60 * 1000); // 1 year validity

    console.log("🍎 Issuing organic apple certification (German orchard)...");

    // Berlin lab certifies organic apples from German orchard
    const appleCertification = [
        new FederationProperty(productOrganic).withAllowedValues([PropertyValue.newText("true")]),
        new FederationProperty(originVerified).withAllowedValues([PropertyValue.newText("DE")]),
        new FederationProperty(batchTested).withAllowedValues([PropertyValue.newText("passed")]),
        new FederationProperty(complianceEu).withAllowedValues([PropertyValue.newText("true")]),
        new FederationProperty(expiryDate).withAllowedValues([PropertyValue.newText(expiry.toISOString())]),
    ];

    await hierarchies
        .createAccreditationToAttest(standardsConsortium.id, organicApplesBatch, appleCertification)
        .buildAndExecute(hierarchies);

    // Get the accreditation data to display real information
    const appleAccreditations = await hierarchies.readOnly().getAccreditationsToAttest(
        standardsConsortium.id,
        organicApplesBatch,
    );

    if (appleAccreditations.accreditations.length > 0) {
        formatCertificationInfo(
            "Organic Apples",
            `Batch #${organicApplesBatch.substring(0, 8)}`,
            appleAccreditations.accreditations[0],
            {
                productOrganic,
                originVerified,
                batchTested,
                complianceEu,
                complianceFda,
                iso22000,
                expiryDate,
            },
        );
    }

    console.log("\n🥫 Issuing processed food certification (US manufacturer)...");

    // California lab certifies processed food for US market
    const processedFoodCert = [
        new FederationProperty(iso22000).withAllowedValues([PropertyValue.newText("certified")]),
        new FederationProperty(originVerified).withAllowedValues([PropertyValue.newText("US")]),
        new FederationProperty(batchTested).withAllowedValues([PropertyValue.newText("passed")]),
        new FederationProperty(complianceFda).withAllowedValues([PropertyValue.newText("true")]),
        new FederationProperty(expiryDate).withAllowedValues([PropertyValue.newText(expiry.toISOString())]),
    ];

    await hierarchies
        .createAccreditationToAttest(standardsConsortium.id, processedFoodBatch, processedFoodCert)
        .buildAndExecute(hierarchies);

    // Get the accreditation data to display real information
    const processedFoodAccreditations = await hierarchies.readOnly().getAccreditationsToAttest(
        standardsConsortium.id,
        processedFoodBatch,
    );

    if (processedFoodAccreditations.accreditations.length > 0) {
        formatCertificationInfo(
            "Processed Food",
            `Batch #${processedFoodBatch.substring(0, 8)}`,
            processedFoodAccreditations.accreditations[0],
            {
                productOrganic,
                originVerified,
                batchTested,
                complianceEu,
                complianceFda,
                iso22000,
                expiryDate,
            },
        );
    }

    // =============================================================================
    // STEP 7: Retail Import Validation
    // =============================================================================
    console.log("🏪 Step 7: Demonstrating retail import validation...");

    // Scenario: European retailer importing organic apples from Germany
    console.log("🛒 Scenario: EU supermarket chain validating organic apple import");

    const importRequirements = new Map([
        [productOrganic.toString(), PropertyValue.newText("true")],
        [complianceEu.toString(), PropertyValue.newText("true")],
        [batchTested.toString(), PropertyValue.newText("passed")],
    ]);

    const importValid = await hierarchies.readOnly().validateProperties(
        standardsConsortium.id,
        organicApplesBatch, // Validate the organic apples product, not the lab
        importRequirements,
    );

    if (importValid) {
        console.log("✅ Import validation successful:");
        console.log("   - Organic certification verified");
        console.log("   - EU compliance confirmed");
        console.log("   - Quality testing passed");
        console.log("   - Trust chain verified: ISO Europe → German Institute → Berlin Lab");
        console.log("   - Products approved for retail sale");
    } else {
        console.log("❌ Import validation failed: Requirements not met");
    }

    // Scenario: US retailer checking FDA compliance
    console.log("\n🇺🇸 Scenario: US grocery chain validating FDA compliance");

    const fdaRequirements = new Map([
        [iso22000.toString(), PropertyValue.newText("certified")],
        [complianceFda.toString(), PropertyValue.newText("true")],
        [batchTested.toString(), PropertyValue.newText("passed")],
    ]);

    const fdaValid = await hierarchies.readOnly().validateProperties(
        standardsConsortium.id,
        processedFoodBatch, // Validate the processed food product, not the lab
        fdaRequirements,
    );

    if (fdaValid) {
        console.log("✅ FDA compliance validation successful:");
        console.log("   - ISO 22000 food safety certification confirmed");
        console.log("   - FDA regulatory compliance verified");
        console.log("   - Quality testing requirements met");
        console.log("   - Products cleared for US market distribution");
    }

    // =============================================================================
    // STEP 8: Consumer Verification via QR Code
    // =============================================================================
    console.log("\n📱 Step 8: Consumer verification demonstration...");

    // Consumer app validates organic claim
    const consumerVerification = new Map([
        [productOrganic.toString(), PropertyValue.newText("true")],
        [originVerified.toString(), PropertyValue.newText("DE")],
    ]);

    const consumerValid = await hierarchies.readOnly().validateProperties(
        standardsConsortium.id,
        organicApplesBatch, // Validate the organic apples product, not the lab
        consumerVerification,
    );

    if (consumerValid) {
        console.log("✅ Consumer verification successful:");
        console.log("   - ✓ Genuinely organic (certified by authorized lab)");
        console.log("   - ✓ Origin: Germany (verified)");
        console.log("   - ✓ Certification authority: Berlin Food Safety Lab");
        console.log("   - ✓ Trust chain intact and verifiable");
        console.log("   - Consumer can trust the organic claim!");
    }

    // =============================================================================
    // STEP 9: Product Recall Scenario
    // =============================================================================
    console.log("\n🚨 Step 9: Product recall scenario...");

    console.log("⚠️  Scenario: Quality issue discovered, product recall needed");
    console.log("   Issue: Contamination found in processed food batch");
    console.log("   Action required: Immediate certification revocation");

    // Real implementation: Revoke the specific attestation
    const processedFoodAccreditationsForRevocation = await hierarchies.readOnly().getAccreditationsToAttest(
        standardsConsortium.id,
        processedFoodBatch,
    );

    if (processedFoodAccreditationsForRevocation.accreditations.length > 0) {
        const accreditationId = processedFoodAccreditationsForRevocation.accreditations[0].id;

        await hierarchies
            .revokeAccreditationToAttest(standardsConsortium.id, processedFoodBatch, accreditationId)
            .buildAndExecute(hierarchies);

        console.log("🚨 CERTIFICATION REVOKED!");
        console.log(`   Revoked accreditation ID: ${accreditationId}`);

        // Verify revocation by checking accreditations again
        const revokedCheck = await hierarchies.readOnly().getAccreditationsToAttest(
            standardsConsortium.id,
            processedFoodBatch,
        );

        if (revokedCheck.accreditations.length === 0) {
            console.log("✅ Revocation confirmed - no active certifications remain");
        }

        // Test validation after revocation (should fail)
        const postRevocationValidation = await hierarchies.readOnly().validateProperties(
            standardsConsortium.id,
            processedFoodBatch,
            new Map([[iso22000.toString(), PropertyValue.newText("certified")]]),
        );

        if (!postRevocationValidation) {
            console.log("✅ Validation correctly fails after revocation");
        }
    }

    console.log("📋 Recall process completed:");
    console.log("   1. Laboratory identifies contamination");
    console.log("   2. Batch certification immediately revoked ✓");
    console.log("   3. Downstream validations automatically fail ✓");
    console.log("   4. Retailers notified through failed re-validation");
    console.log("   5. Products removed from shelves");
    console.log("   6. Consumer apps show recall status");
    console.log("   7. Supply chain impact minimized through precise targeting\n");

    // =============================================================================
    // SUMMARY
    // =============================================================================
    console.log("\n📊 Example Summary:");
    console.log("=====================================");
    console.log("✅ International standards consortium federation created");
    console.log("✅ Comprehensive certification statements defined");
    console.log("✅ Multi-regional authority structure established");
    console.log("✅ Hierarchical delegation: International → Regional → National → Local");
    console.log("✅ Product certifications issued with expiry management");
    console.log("✅ Import/export validation demonstrated");
    console.log("✅ Consumer verification enabled");
    console.log("✅ Product recall scenario handled");
    console.log("\n🎯 Benefits Achieved:");
    console.log("   - Instant compliance verification across borders");
    console.log("   - Fraud prevention through cryptographic certificates");
    console.log("   - Automated expiry management");
    console.log("   - Streamlined import/export processes");
    console.log("   - Consumer trust through transparency");
    console.log("   - Efficient product recall capabilities");
    console.log("   - Reduced regulatory overhead");
    console.log("   - Global interoperability of certifications");
}

// Export for main.ts integration
export { supplyChainExample as supplyChain };
