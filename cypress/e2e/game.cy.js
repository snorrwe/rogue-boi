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
	});
});

describe('basic game tests', () => {
	beforeEach(() => {
		cy.visit('/#newgame');
	});

	it('should have player in the middle of the canvas', () => {
		// player should be in the center
		cy.get('canvas')
			.should('exist')
			.then((canva) => {
				const w = canva.width();
				const h = canva.height();

				const cw = w / 2;
				const ch = h / 2;

				cy.wrap(canva).scrollIntoView().click(cw, ch);
			});

		// assert that the player has been selected
		cy.get('.icon').should('exist');
		cy.get('h3').contains('The player');
		cy.get('h2').contains('Equipment');
		cy.get('h2').contains('Inventory');
		cy.get('h2').contains('Log');
	});
});
