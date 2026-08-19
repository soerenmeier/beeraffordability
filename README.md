# Crelte project

This is a crelte project, if you want to know more about crelte
check out the [documentation](https://docs.crelte.com/).

## Local Development

Everything (Craft CMS and the Svelte dev server) runs inside [DDEV](https://ddev.com),
so you don't need PHP or Node.js on your machine.

Once you have cloned the repository you can start the project with the following commands:

### CMS

```bash
# start ddev (from the project root)
ddev start
ddev composer install
ddev import-db --file=dump.sql.gz
# copy assets
```

### Svelte

```bash
ddev npm install
ddev npm run dev
# then run `ddev launch` or open https://<project>.ddev.site
```

## URLs

- **Frontend**: `https://<project>.ddev.site`
- **Craft control panel**: `https://admin.<project>.ddev.site/admin`
