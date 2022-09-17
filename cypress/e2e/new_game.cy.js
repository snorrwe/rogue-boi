describe('newgame loads', () => {
	it('newgame button visits #newgame', () => {
		cy.visit('/');
		cy.clickLink('New Game');
		cy.url().should('include', '#game');
	});
	it('loads', () => {
		cy.visit('/#newgame');
		cy.url().should('include', '#game');
		cy.get('.game-ui').should('exist');
		cy.get('canvas').should('exist');
	});
});
