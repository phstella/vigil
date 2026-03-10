# Plugin Sandbox Model

## Objective
Enable high-performance extensibility through WASM plugins with strict isolation and explicit capabilities.

## Runtime Choice
- Runtime: `wasmtime` in Rust backend.
- Execution model: host-driven function calls with bounded memory and timeouts.

## Manifest Schema (v1)
Required fields:
- `id`
- `name`
- `version`
- `api_version`
- `entry_wasm`
- `capabilities`

Optional fields:
- `description`
- `author`
- `homepage`
- `checksum`

## Capability Set (Initial)
- `workspace:read`
- `workspace:write`
- `editor:read_active`
- `editor:apply_edit`
- `omnibar:register_command`
- `sidebar:register_panel`
- `network:fetch` (off by default)

Policy:
- Deny by default.
- User must explicitly approve requested capabilities at install time.
- Capability grants are persisted per workspace.

## Security Controls
- Plugin memory limit per instance.
- Execution timeout per host call.
- No direct filesystem access outside approved host API.
- Optional network capability gated and auditable.
- Manifest checksum verification before load.

## API Versioning
- Host exposes `plugin_api_version`.
- Plugin must declare matching semver range.
- Incompatible versions are rejected with actionable error.

## Lifecycle
1. Install manifest + wasm artifact.
2. Validate schema, checksum, capability policy.
3. Register plugin metadata in local registry.
4. Enable plugin and initialize runtime instance.
5. Route events/commands through capability-filtered host API.
6. On crash, mark unhealthy and isolate from host process.

## UI Integration
- Omnibar command: `> Install Plugin`.
- Plugin management panel:
  - list installed plugins
  - enable/disable
  - show requested/granted capabilities
  - display runtime health and last error

## Audit Logging
Record plugin actions locally:
- install/upgrade/uninstall events
- capability grants/revocations
- runtime faults/timeouts
- network capability usage

## Non-Goals (Initial)
- Remote code execution outside WASM sandbox.
- Arbitrary native dynamic library loading.
- Background plugin daemons outside host runtime.
