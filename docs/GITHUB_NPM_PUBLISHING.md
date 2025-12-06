# GitHub npm Registry Publishing Design Document

## Overview

This document outlines the architecture and implementation plan for publishing the `token-tracker` CLI to GitHub's private npm registry, enabling users to run:

```bash
npx @0xinevitable/token-tracker [command]
```

## Current Structure (Implemented)

```
token-tracker/
├── package.json              # token-tracker-monorepo (private, workspace root)
├── .npmrc                    # GitHub registry config
├── packages/
│   └── cli/
│       ├── package.json      # @0xinevitable/token-tracker
│       ├── src/              # CLI TypeScript source
│       ├── dist/             # Built CLI (included in publish)
│       └── tsconfig.json
├── core/                     # @0xinevitable/token-tracker-core (napi-rs native)
│   ├── package.json
│   ├── src/                  # Rust source
│   ├── index.js              # Generated loader
│   ├── index.d.ts            # Generated types
│   └── *.node                # Platform binaries (gitignored)
└── frontend/                 # Next.js app (unchanged)
```

### Workspace Configuration

Root `package.json`:
```json
{
  "name": "token-tracker-monorepo",
  "private": true,
  "workspaces": ["core", "packages/*", "frontend", "benchmarks"]
}
```

### Package Scopes

| Package | Scope | Purpose |
|---------|-------|---------|
| `@0xinevitable/token-tracker` | CLI | Main CLI tool, published to GitHub npm |
| `@0xinevitable/token-tracker-core` | Core | Native Rust module with binaries |

---

## Implementation Plan

### Phase 1: Package Configuration

#### 1.1 Update Root package.json

```json
{
  "name": "@0xinevitable/token-tracker",
  "version": "1.0.0",
  "description": "Calculate token prices from AI coding sessions",
  "type": "module",
  "bin": {
    "token-tracker": "./dist/cli.js"
  },
  "files": [
    "dist/**/*",
    "src/**/*"
  ],
  "publishConfig": {
    "registry": "https://npm.pkg.github.com",
    "access": "restricted"
  },
  "repository": {
    "type": "git",
    "url": "https://github.com/0xinevitable/token-usage-tracker.git"
  },
  "dependencies": {
    "@0xinevitable/token-tracker-core": "^0.1.0",
    ...
  }
}
```

**Key Changes:**
- Remove `"private": true`
- Change name to `@0xinevitable/token-tracker`
- Add `publishConfig` for GitHub registry
- Add `files` to control what gets published
- Update core dependency name

#### 1.2 Update core/package.json

```json
{
  "name": "@0xinevitable/token-tracker-core",
  "version": "0.1.0",
  "publishConfig": {
    "registry": "https://npm.pkg.github.com",
    "access": "restricted"
  },
  "repository": {
    "type": "git",
    "url": "https://github.com/0xinevitable/token-usage-tracker.git",
    "directory": "core"
  },
  "files": [
    "index.js",
    "index.d.ts",
    "*.node"
  ]
}
```

### Phase 2: Registry Configuration

#### 2.1 Create .npmrc (Repository Root)

```ini
# GitHub npm registry for @0xinevitable scope
@0xinevitable:registry=https://npm.pkg.github.com

# Auth token (set via environment variable or npm login)
//npm.pkg.github.com/:_authToken=${GITHUB_TOKEN}
```

#### 2.2 User Authentication Requirements

Users need to authenticate to use `npx @0xinevitable/token-tracker`:

**Option A: Environment Variable**
```bash
export GITHUB_TOKEN=ghp_xxxxxxxxxxxx
npx @0xinevitable/token-tracker
```

**Option B: Global .npmrc**
```bash
# ~/.npmrc
@0xinevitable:registry=https://npm.pkg.github.com
//npm.pkg.github.com/:_authToken=ghp_xxxxxxxxxxxx
```

**Option C: npm login**
```bash
npm login --registry=https://npm.pkg.github.com --scope=@0xinevitable
```

### Phase 3: CI/CD Workflow Updates

#### 3.1 Updated build-native.yml (Publish Section)

```yaml
# Publish core package first
publish-core:
  name: Publish Core to GitHub
  if: startsWith(github.ref, 'refs/tags/v')
  runs-on: ubuntu-latest
  needs: [build, test-macos-windows, test-linux]
  permissions:
    contents: read
    packages: write
  steps:
    - uses: actions/checkout@v4

    - name: Setup Node.js
      uses: actions/setup-node@v4
      with:
        node-version: 20
        registry-url: "https://npm.pkg.github.com"
        scope: "@0xinevitable"

    - name: Download all artifacts
      uses: actions/download-artifact@v4
      with:
        path: core/artifacts

    - name: Move artifacts
      working-directory: core
      run: |
        for dir in artifacts/bindings-*/; do
          cp "$dir"*.node . 2>/dev/null || true
        done
        ls -la *.node

    - name: Publish Core
      working-directory: core
      run: npm publish
      env:
        NODE_AUTH_TOKEN: ${{ secrets.GITHUB_TOKEN }}

# Publish CLI package after core
publish-cli:
  name: Publish CLI to GitHub
  if: startsWith(github.ref, 'refs/tags/v')
  runs-on: ubuntu-latest
  needs: [publish-core]
  permissions:
    contents: read
    packages: write
  steps:
    - uses: actions/checkout@v4

    - name: Setup Node.js
      uses: actions/setup-node@v4
      with:
        node-version: 20
        registry-url: "https://npm.pkg.github.com"
        scope: "@0xinevitable"

    - name: Install dependencies
      run: yarn install --frozen-lockfile

    - name: Build CLI
      run: yarn build  # Compile TypeScript to dist/

    - name: Publish CLI
      run: npm publish
      env:
        NODE_AUTH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### Phase 4: Build Script Updates

#### 4.1 Add TypeScript Build

Update `package.json` scripts:

```json
{
  "scripts": {
    "build": "tsc",
    "prepublishOnly": "yarn build"
  }
}
```

Add `tsconfig.json` if not present:

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "outDir": "./dist",
    "rootDir": "./src",
    "declaration": true,
    "strict": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "core", "frontend", "benchmarks"]
}
```

---

## Dependency Resolution

### How CLI Finds Core Package

When a user runs `npx @0xinevitable/token-tracker`:

1. npm downloads `@0xinevitable/token-tracker` from GitHub registry
2. npm sees dependency on `@0xinevitable/token-tracker-core`
3. npm downloads core package (includes platform-specific `.node` binary)
4. napi-rs `index.js` auto-detects platform and loads correct binary

### Platform Binary Loading (core/index.js)

```javascript
// napi-rs generated loader
const { existsSync, readFileSync } = require('fs');
const { join } = require('path');

const { platform, arch } = process;
let nativeBinding = null;

// Platform detection
switch (platform) {
  case 'darwin':
    switch (arch) {
      case 'x64':
        nativeBinding = require('./token-tracker-core.darwin-x64.node');
        break;
      case 'arm64':
        nativeBinding = require('./token-tracker-core.darwin-arm64.node');
        break;
    }
    break;
  case 'linux':
    // Similar logic for linux-x64-gnu, linux-arm64-gnu, etc.
    break;
  case 'win32':
    // Windows binaries
    break;
}

module.exports = nativeBinding;
```

---

## User Experience

### First-Time Setup (One-Time)

```bash
# Create GitHub Personal Access Token with `read:packages` scope
# Then configure npm:
echo "@0xinevitable:registry=https://npm.pkg.github.com" >> ~/.npmrc
echo "//npm.pkg.github.com/:_authToken=YOUR_TOKEN" >> ~/.npmrc
```

### Usage

```bash
# Run directly via npx
npx @0xinevitable/token-tracker

# Or install globally
npm install -g @0xinevitable/token-tracker
token-tracker
```

---

## Versioning Strategy

### Semantic Versioning

- **CLI version**: Independent, follows its own semver
- **Core version**: Independent, follows its own semver
- **Compatibility**: CLI `package.json` specifies compatible core versions

### Release Process

1. **Core-only change**: Bump core version, publish core
2. **CLI-only change**: Bump CLI version, publish CLI
3. **Both**: Bump both, publish core first, then CLI

### Git Tags

```bash
# For combined releases
git tag v1.0.0

# For separate releases (future consideration)
git tag core-v0.1.0
git tag cli-v1.0.0
```

---

## Security Considerations

### Token Scopes Required

| Operation | Required Scope |
|-----------|---------------|
| Read/Install packages | `read:packages` |
| Publish packages | `write:packages` |
| Delete packages | `delete:packages` |

### Repository Visibility

- **Private repo**: Only collaborators can install packages
- **Public repo**: Anyone with valid GitHub token can install

### Recommendations

1. Use fine-grained PATs when available
2. Set token expiration (90 days recommended)
3. Store CI tokens as repository secrets
4. Never commit tokens to `.npmrc`

---

## Rollback Plan

If issues arise:

1. **Unpublish**: `npm unpublish @0xinevitable/token-tracker@version`
2. **Deprecate**: `npm deprecate @0xinevitable/token-tracker@version "message"`
3. **Revert**: Push new version with fixes

---

## Testing Checklist

- [ ] Core package publishes successfully
- [ ] CLI package publishes successfully
- [ ] `npx @0xinevitable/token-tracker --version` works
- [ ] Platform binaries load correctly (macOS, Linux, Windows)
- [ ] New user can authenticate and install
- [ ] Global install works: `npm i -g @0xinevitable/token-tracker`

---

## Files to Modify

| File | Changes |
|------|---------|
| `package.json` | Remove private, rename, add publishConfig, files |
| `core/package.json` | Rename to @0xinevitable scope, add publishConfig |
| `.npmrc` | Create with GitHub registry config |
| `.github/workflows/build-native.yml` | Update publish jobs |
| `tsconfig.json` | Add/update for CLI build |
| `src/native.ts` | Update import path if core renamed |

---

## Timeline Estimate

| Phase | Effort |
|-------|--------|
| Phase 1: Package Config | 1-2 hours |
| Phase 2: Registry Config | 30 min |
| Phase 3: CI/CD Updates | 1-2 hours |
| Phase 4: Build Scripts | 30 min |
| Testing | 1-2 hours |
| **Total** | **4-7 hours** |

---

## User Authentication Guide

### Prerequisites

Users need a GitHub Personal Access Token (PAT) with `read:packages` scope to install packages from the GitHub npm registry.

### Creating a GitHub Token

1. Go to https://github.com/settings/tokens
2. Click "Generate new token (classic)"
3. Select scope: `read:packages`
4. Set expiration (recommended: 90 days)
5. Copy the token (starts with `ghp_`)

### Configuration Options

#### Option 1: Global .npmrc (Recommended)

```bash
# Add to ~/.npmrc
echo "@0xinevitable:registry=https://npm.pkg.github.com" >> ~/.npmrc
echo "//npm.pkg.github.com/:_authToken=ghp_YOUR_TOKEN_HERE" >> ~/.npmrc
```

#### Option 2: Environment Variable

```bash
export GITHUB_TOKEN=ghp_YOUR_TOKEN_HERE
npx @0xinevitable/token-tracker
```

#### Option 3: npm login

```bash
npm login --registry=https://npm.pkg.github.com --scope=@0xinevitable
# Username: your-github-username
# Password: ghp_YOUR_TOKEN_HERE
# Email: your-email@example.com
```

### Usage After Authentication

```bash
# Run directly via npx
npx @0xinevitable/token-tracker

# Or install globally
npm install -g @0xinevitable/token-tracker
token-tracker

# With specific command
npx @0xinevitable/token-tracker graph --year 2025
```

### Troubleshooting

| Error | Solution |
|-------|----------|
| `401 Unauthorized` | Token missing or expired - regenerate PAT |
| `403 Forbidden` | Token lacks `read:packages` scope |
| `404 Not Found` | Package not published yet or wrong scope |
| `UNABLE_TO_GET_ISSUER_CERT_LOCALLY` | Corporate proxy - configure npm proxy settings |

### For Organization Members

If you're a member of the `0xinevitable` organization:
- You automatically have read access to private packages
- Use your regular GitHub PAT with `read:packages` scope

### For External Collaborators

External users need to be granted access to the repository or the package must be made public.
