/// <reference path="../pb_data/types.d.ts" />
migrate(
	(app) => {
		const secrets = app.findCollectionByNameOrId('secrets');
		secrets.fields.add(
			new Field({
				type: 'text',
				name: 'username'
			})
		);
		app.save(secrets);
	},
	(app) => {
		const secrets = app.findCollectionByNameOrId('secrets');
		secrets.fields.removeByName('username');
		app.save(secrets);
	}
);
