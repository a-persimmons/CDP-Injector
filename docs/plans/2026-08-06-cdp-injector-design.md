# CDP Injector Design

## Product name

- English: CDP Injector
- Chinese: CDP 注入器
- macOS application: `CDP Injector.app`
- Installable module package: `.cdpmod`
- Specification name: CDP Injector Module Specification

`Hub` is not part of the product name. It may remain an internal name for the
component that manages product sessions and CDP connections.

## Goal

CDP Injector is a resident macOS launcher that:

- organizes Electron and Chromium applications by product;
- maintains one CDP session for each launched application instance;
- injects multiple selected modules through the product's shared CDP session;
- manages module installation, companion services, injection, and status;
- launches an application normally when no module is enabled.

The first release validates Codex only. The architecture permits additional
products later without claiming that untested products work out of the box.

The first release does not include an online marketplace, automatic dependency
installation, Windows or Linux support, module dependencies, or a complete
security sandbox.

## Technology stack

CDP Injector is a standalone application rather than a new Taskboard feature.
Its first-release stack is:

```text
CDP Injector.app
├── Tauri 2 desktop shell
├── React + TypeScript + Vite user interface
├── Rust core
│   ├── Product and Product Session management
│   ├── application launch and termination
│   ├── CDP discovery, WebSocket transport, and target watching
│   ├── module installation and lifecycle orchestration
│   └── tray, logs, and persistent settings
└── bundled Node runtime
    └── optional module service/index.mjs processes
```

Rust owns every Product-level CDP connection. Modules never connect directly to
the Product's debugging port. The bundled Node runtime exists only for module
services and is not required for CSS- or renderer-only modules.

The initial implementation belongs in a standalone `cdp-injector` repository.
This Taskboard repository remains the source of the first application module
and the reference implementation being migrated to `.cdpmod`.

## Core model

### Product

A Product describes how CDP Injector locates, launches, stops, and connects to
an application. It also defines how renderer targets are classified and whether
the application supports a separate preview instance.

### Product Session

A Product Session represents one running application instance:

```text
Codex Session
├── CDP port
├── CDP connection
├── renderer targets
├── enabled modules
└── module services
```

One application instance owns one CDP port. Modules do not open competing CDP
connections; they share the Product Session managed by CDP Injector.

### Module

A Module is an independently enabled feature injected into a Product, such as
a theme, taskboard, shortcut, or page enhancement.

### Module Project and Module Package

A Module Project is developer-owned source code and may use any build system.
A Module Package is the prebuilt artifact installed by users. Its extension is
`.cdpmod`; it is a ZIP archive with `manifest.json` at its root.

CDP Injector never runs `npm install` or a module build command during install.

## Product Profile

The first release ships Product Profiles with the application. Codex is the
first built-in Profile.

```json
{
  "schemaVersion": 1,
  "id": "codex",
  "name": "Codex",
  "platform": "darwin",
  "icon": "codex.png",
  "application": {
    "paths": [
      "/Applications/ChatGPT.app",
      "/Applications/Codex.app"
    ],
    "processNames": ["ChatGPT", "Codex"]
  },
  "cdp": {
    "contexts": [
      {
        "id": "main",
        "targetType": "page",
        "urlPrefixes": ["app://"],
        "excludeUrlContains": ["global-dictation"]
      }
    ]
  },
  "preview": {
    "supported": false,
    "restartMessage": "将退出并重新启动 Codex"
  }
}
```

Product Profiles cannot contain arbitrary shell commands. CDP Injector owns
application launching, termination, and CDP argument construction. Installable
Product Profiles may be considered after a second product is actually tested.

## Module package layout

A CSS-only module:

```text
theme.cdpmod/
├── manifest.json
├── inject/
│   └── index.css
└── assets/
    └── icon.png
```

A script module:

```text
enhancement.cdpmod/
├── manifest.json
├── inject/
│   ├── index.js
│   └── index.css
└── assets/
    └── icon.png
```

A module with a local web application:

```text
taskboard.cdpmod/
├── manifest.json
├── inject/
│   └── index.js
├── service/
│   ├── index.mjs
│   └── ...
├── web/
│   ├── index.html
│   └── assets/
└── assets/
    └── icon.png
```

Packages cannot contain `node_modules`, install scripts, build scripts,
symbolic links, paths outside the package, or commands that users must run.

## Module manifest

```json
{
  "schemaVersion": 1,
  "id": "dev.dashi.taskboard",
  "name": "任务面板",
  "version": "0.1.0",
  "description": "在 Codex 中显示本地任务面板",
  "icon": "assets/icon.png",
  "hubApi": 1,
  "targets": [
    {
      "product": "codex",
      "context": "main"
    }
  ],
  "inject": {
    "entry": "inject/index.js",
    "styles": [],
    "runAt": "document-start"
  },
  "service": {
    "entry": "service/index.mjs",
    "healthPath": "/health",
    "readyTimeoutMs": 10000
  },
  "capabilities": [
    "renderer-injection",
    "local-service",
    "module-data",
    "csp-bypass"
  ]
}
```

Rules:

- `schemaVersion` is `1` in the first release.
- `id` is stable and globally unique; reverse-domain notation is recommended.
- `version` uses SemVer.
- `hubApi` is `1` in the first release.
- `targets` contains at least one Product and renderer context.
- `inject` contains at least a JavaScript entry or one stylesheet.
- `service` is optional.
- `capabilities` is displayed before installation.

Supported first-release capabilities are:

```text
renderer-injection
local-service
module-data
csp-bypass
external-network
```

Capabilities are declarations and warnings. They do not claim that arbitrary
Node module code is fully sandboxed by the operating system.

## Renderer lifecycle

CDP Injector installs `globalThis.cdpHub` into each matched renderer. A script
module registers itself as follows:

```js
globalThis.cdpHub.register({
  id: "dev.dashi.taskboard",

  async activate(context) {
    const entry = document.createElement("button");
    entry.textContent = "任务面板";
    entry.dataset.cdpHubOwner = context.module.id;
    document.body.append(entry);

    return () => {
      entry.remove();
    };
  }
});
```

The only lifecycle contract is:

```text
activate(context) -> cleanup function
```

```ts
type ModuleContext = {
  module: {
    id: string;
    version: string;
  };
  product: {
    id: string;
  };
  target: {
    id: string;
    url: string;
    title: string;
  };
  serviceUrl: string | null;
};
```

Lifecycle requirements:

- `activate` may be asynchronous and must return a cleanup function.
- Before disable or upgrade, CDP Injector calls the previous cleanup function.
- DOM created by a module uses `data-cdp-hub-owner="<module-id>"`.
- Cleanup removes DOM, event listeners, timers, and observers owned by the module.
- Reloading a renderer performs a new lifecycle activation.
- Modules cannot depend on another module or on injection order.
- Modules must handle Product UI rerenders themselves.
- Product DOM stability is not part of the Hub API guarantee.

CSS-only modules do not need a script lifecycle. CDP Injector inserts and
removes their style elements.

## Local service protocol

If `service` exists, CDP Injector launches its entry with the Node runtime
bundled in the application. It invokes the entry directly without a shell.

The process receives:

```text
CDP_HUB_MODULE_ID
CDP_HUB_MODULE_DIR
CDP_HUB_DATA_DIR
CDP_HUB_HOST=127.0.0.1
CDP_HUB_PORT=<allocated-port>
CDP_HUB_SESSION_TOKEN=<random-token>
CDP_HUB_PRODUCT_ID
```

The service must:

- bind only to the provided loopback host and port;
- return HTTP 200 from its declared health path;
- persist data only under `CDP_HUB_DATA_DIR` by default;
- handle `SIGTERM` and close databases cleanly;
- run without system Node or npm;
- avoid native Node addons in the first package format.

Runtime sequence:

```text
allocate port
-> start service
-> poll health path
-> match renderer
-> inject module
-> pass the opaque serviceUrl to activate()
```

The first Product Session that consumes a module starts its service. Sessions
share one service process for the same module. The last consumer stopping ends
the service. A service crash fails that module only.

## Loopback HTTP security

- Services bind to loopback only.
- API requests validate the random session token.
- Services validate `Host` and applicable `Origin` headers.
- Module iframes use `referrerPolicy="no-referrer"`.
- External network access is declared with `external-network`.
- Web content never receives the Product's CDP WebSocket address.

An imported module is still trusted local code. Its renderer script can read
the target page, and its service process has the current user's OS privileges.
The installer must state this clearly.

## Multiple modules

For a Product Session, CDP Injector:

1. installs its renderer runtime;
2. registers every enabled document-start script;
3. inserts module styles;
4. evaluates modules in the current document;
5. watches renderer and target replacement;
6. repeats matching and injection for new targets.

Injection is deterministic but order is not a public API. A single module
failure is recorded against that module and does not block the remaining
modules.

## Application launch behavior

With no enabled module:

```text
click Product
-> launch or focus it normally
-> do not open a CDP port
```

With enabled modules:

```text
click Product
-> inspect existing process
-> inspect preview capability
-> show restart notice when required
-> terminate the old instance normally
-> wait for process exit
-> allocate an application-level CDP port
-> start required module services
-> launch the Product with CDP flags
-> wait for /json/version
-> connect renderer targets
-> inject enabled modules
```

For a Product without preview support, the UI marks preview unavailable and
uses its configured restart message. CDP Injector does not force-kill a process
that refuses normal termination; it asks the user to quit it manually.

## User interface states

Product states:

```text
not running
running normally
stopping
starting
connecting to CDP
injecting
injected
partially failed
launch failed
```

Module states:

```text
disabled
waiting
starting service
injecting
running
injection failed
service failed
incompatible
```

The first UI contains Product icons, running state, installed modules, enable
switches, a launch button, local `.cdpmod` import, capability disclosure, and
module-specific errors. It excludes marketplace, accounts, ratings, comments,
and automatic recommendations.

## Installation and storage

Recommended paths:

```text
~/Library/Application Support/CDP Injector/
├── Modules/
│   └── dev.dashi.taskboard/
│       └── 0.1.0/
├── Module Data/
│   └── dev.dashi.taskboard/
└── Logs/
```

Install flow:

```text
choose .cdpmod
-> extract to a temporary directory
-> reject unsafe ZIP paths and links
-> validate manifest and contained paths
-> check hubApi and Product targets
-> display capabilities
-> confirm
-> atomically move into Modules
```

An update with the same module ID replaces code but preserves Module Data.
Before updating, CDP Injector stops the old service and runs renderer cleanup.
Uninstall removes code and leaves data by default unless the user explicitly
chooses to remove it.

## Build and release

Developers may use React, Vue, Vite, or plain JavaScript. Publishing performs:

```text
install development dependencies
-> build web UI
-> bundle renderer entry
-> bundle service entry
-> generate manifest
-> validate output
-> create .cdpmod
```

The final renderer entry is directly executable browser JavaScript without bare
npm imports. The service is ESM compatible with the bundled Node runtime. Web
content is prebuilt static output.

Developer tooling should eventually provide:

```bash
cdp-injector validate ./dist-module
cdp-injector pack ./dist-module
```

The first release may also import an unpacked module directory in an explicitly
marked development mode.

## Compatibility

- `schemaVersion` versions the package structure.
- `hubApi` versions the renderer lifecycle API.
- Product and Module IDs are immutable after release.
- Unknown Product or renderer contexts make a module incompatible.
- A mismatched Hub API major version prevents activation.
- Product UI selector breakage is a module compatibility issue.
- Product version ranges and module dependencies are deferred until a real need
  appears.

## Error isolation and logs

CDP Injector reports errors at Product Session, renderer target, and Module
scope. It distinguishes missing applications, CDP startup failure, missing
targets, CSP failure, script exceptions, cleanup exceptions, service startup
failure, health timeout, service exit, and iframe load failure.

Logs contain timestamp, Product, Session, Module, target, lifecycle phase, and
error. One module failure does not terminate the Product or other modules.

## Taskboard migration

The existing Taskboard maps into the specification as follows:

```text
inject/codex-taskboard.user.js
-> inject/index.js
-> register with cdpHub
-> use context.serviceUrl
-> return the existing destroy logic as cleanup

server/ and shared runtime files
-> service/
-> read Hub host, port, token, and data directory

dist/web/
-> web/
```

Generic CDP connection, target discovery, document-start registration, CSP
bypass, renderer watching, and source refresh move into CDP Injector.

Taskboard service supervision, iframe behavior, Codex composer prefilling, and
Taskboard automation remain Taskboard module responsibilities rather than Hub
core behavior.

## First-release acceptance

1. `CDP Injector.app` launches normally on macOS and remains resident.
2. The UI shows the Codex Product.
3. The built-in theme and Taskboard can be enabled independently.
4. Starting Codex with both modules injects both through one Product Session.
5. The theme and Taskboard remain simultaneously usable.
6. Taskboard requires no user-installed Node, npm, build, or manual service.
7. Codex launches normally when no module is enabled.
8. An existing Codex instance receives the configured restart notice.
9. Codex is marked as not supporting preview.
10. Renderer replacement causes automatic reinjection.
11. One failed module does not prevent another module from running.
12. The UI imports a local `.cdpmod` package.
13. Unsafe or incompatible packages are rejected.
14. Other Electron and Chromium applications are not release acceptance targets.

## Future catalog boundary

A future website may display modules by Product, publish versions, and provide
signed `.cdpmod` downloads. It does not connect directly to local CDP endpoints
or execute remote injection code.

One-click installation may later use a custom URL such as:

```text
cdp-injector://install?module=...
```

Marketplace, signing, automatic update, and custom URL handling are outside the
first release.
