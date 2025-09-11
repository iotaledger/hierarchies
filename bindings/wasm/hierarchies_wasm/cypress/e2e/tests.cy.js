const { _ } = Cypress;

describe(
    "Test Examples",
    () => {
        const examples = [
            "01_create_federation",
            "02_add_root_authority",
            "03_add_properties",
            "04_create_accreditation_to_attest",
            "05_revoke_accreditation_to_attest",
            "06_create_accreditation_to_accredit",
            "07_revoke_accreditation_to_accredit",
            "08_revoke_root_authority",
            "09_reinstate_root_authority",
            "01_get_accreditations",
            "02_validate_properties",
            "03_get_properties",
        ];

        _.each(examples, (example) => {
            it(example, () => {
                cy.visit("/", {
                    onBeforeLoad(win) {
                        cy.stub(win.console, "log").as("consoleLog");
                    },
                });
                cy.get("@consoleLog").should("be.calledWith", "init");
                cy.window().then(win => win.runTest(example));
                cy.get("@consoleLog").should("be.calledWith", "success");
            });
        });
    },
);
