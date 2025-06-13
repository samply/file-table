# Development

Your new bare-bones project includes minimal organization with a single `main.rs` file and a few assets.

```
project/
├─ assets/ # Any assets that are used by the app should be placed here
├─ src/
│  ├─ main.rs # main.rs is the entry point to your application and currently contains all components for the app
├─ Cargo.toml # The Cargo.toml file defines the dependencies and feature flags for your project
```

### Tailwind

1. Install npm: https://docs.npmjs.com/downloading-and-installing-node-js-and-npm
2. Install the Tailwind CSS CLI: https://tailwindcss.com/docs/installation
3. Run the following command in the root of the project to start the Tailwind CSS compiler:

```bash
npx @tailwindcss/cli -i ./input.css -o ./assets/tailwind.css --watch
```

### Serving Your App

Run the following command in the root of your project to start developing:

```bash
dx serve
```

### Pre-commit hook

I'm currently trialing this `.git/hooks/pre-commit` hook:

```bash
#!/bin/bash

# Save unstaged changes
git diff > /tmp/unstaged_changes.patch
git restore .

# Format only staged files
git diff --name-only --cached | xargs cargo fmt --
git diff --name-only --cached | xargs -n 1 dx fmt --file

# Restage only the originally staged files
git diff --name-only --cached | xargs git add

# Restore unstaged changes
if ! git apply --allow-empty /tmp/unstaged_changes.patch; then
    echo "Pre-commit hook failed to restore your unstaged changes. You can find your changes at /tmp/unstaged_changes.patch"
    exit 1
fi
```