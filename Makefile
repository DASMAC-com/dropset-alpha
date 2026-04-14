.PHONY: docs
.PHONY: docs-build
.PHONY: docs-links
.PHONY: docs-prettier
.PHONY: docs-prod

# Build the docs locally for development
docs:
	cd docs && npm install \
		&& rm -rf .vitepress/cache .vitepress/dist node_modules/.vite \
		&& npx vitepress dev --open

# Build the docs for production
docs-build:
	cd docs \
		&& rm -rf .vitepress/cache .vitepress/dist node_modules/.vite \
		&& npm ci \
		&& npx vitepress build

# Check for any broken links
docs-links: docs-build
	lychee --config cfg/lychee.toml --include-fragments \
		--root-dir docs/.vitepress/dist 'docs/.vitepress/dist/**/*.html'

# Format docs with Prettier
docs-prettier:
	cd docs && npm install && npx prettier --write .

# Serve docs in production mode 
docs-prod: docs-build
	cd docs && (sleep 1 && open http://localhost:4173 &) && npx vitepress preview
