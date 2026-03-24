/// <reference path="../pb_data/types.d.ts" />
migrate(
	(app) => {
		// Add encryption_salt to users collection
		const users = app.findCollectionByNameOrId('users');
		users.fields.add(
			new Field({
				type: 'text',
				name: 'encryption_salt'
			})
		);
		app.save(users);

		// Create secrets collection
		const secrets = new Collection({
			name: 'secrets',
			type: 'base',
			listRule: '@request.auth.id != "" && user = @request.auth.id',
			viewRule: '@request.auth.id != "" && user = @request.auth.id',
			createRule: '@request.auth.id != ""',
			updateRule: '@request.auth.id != "" && user = @request.auth.id',
			deleteRule: '@request.auth.id != "" && user = @request.auth.id'
		});

		secrets.fields.add(
			new Field({
				type: 'text',
				name: 'name',
				required: true
			})
		);

		secrets.fields.add(
			new Field({
				type: 'select',
				name: 'type',
				required: true,
				values: ['password', 'api_key', 'env_var', 'ssh_key', 'note']
			})
		);

		secrets.fields.add(
			new Field({
				type: 'text',
				name: 'encrypted_value',
				required: true
			})
		);

		secrets.fields.add(
			new Field({
				type: 'text',
				name: 'iv',
				required: true
			})
		);

		secrets.fields.add(
			new Field({
				type: 'url',
				name: 'url'
			})
		);

		secrets.fields.add(
			new Field({
				type: 'relation',
				name: 'user',
				required: true,
				collectionId: users.id,
				maxSelect: 1
			})
		);

		app.save(secrets);
	},
	(app) => {
		const secrets = app.findCollectionByNameOrId('secrets');
		app.delete(secrets);
	}
);
