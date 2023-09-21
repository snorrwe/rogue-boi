// For more comprehensive examples of custom
// commands please read more here:
// https://on.cypress.io/custom-commands
// ***********************************************
Cypress.Commands.add("clickLink", (label) => {
  cy.get("a").contains(label).click();
});
