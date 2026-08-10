import { cp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const websiteDir = dirname(fileURLToPath(import.meta.url));
const repoDir = dirname(websiteDir);
const outputDir = join(websiteDir, "dist");
const { version } = JSON.parse(await readFile(join(repoDir, "package.json"), "utf8"));
const repository = "https://github.com/a-persimmons/CDP-Injector";
const latestRelease = `${repository}/releases/latest`;

const copy = {
  zh: {
    locale: "zh-CN",
    brand: "CDP注入器",
    nav: { home: "首页", docs: "使用文档", modules: "模块开发", download: "下载" },
    language: "EN",
    github: "GitHub",
    theme: "切换主题",
    footer: "本地优先的 Electron 模块启动器",
  },
  en: {
    locale: "en",
    brand: "CDP Injector",
    nav: { home: "Home", docs: "User guide", modules: "Module guide", download: "Download" },
    language: "中文",
    github: "GitHub",
    theme: "Toggle theme",
    footer: "A local-first module launcher for Electron apps",
  },
};

const escapeHtml = (value) =>
  value.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");

const codeBlock = (language, value) =>
  `<div class="code-block"><div class="code-label">${language}</div><pre><code>${escapeHtml(value.trim())}</code></pre></div>`;

function paths(lang, page) {
  const prefix = page === "home" ? "../" : "../../";
  const other = lang === "zh" ? "en" : "zh";
  const pagePath = page === "home" ? "" : `${page}/`;
  return {
    prefix,
    home: `${prefix}${lang}/`,
    docs: `${prefix}${lang}/docs/`,
    modules: `${prefix}${lang}/modules/`,
    download: `${prefix}${lang}/download/`,
    alternate: `${prefix}${other}/${pagePath}`,
    css: `${prefix}assets/site.css`,
    js: `${prefix}assets/site.js`,
    icon: `${prefix}assets/icon.png`,
    screenshot: `${prefix}assets/app.png`,
  };
}

function icon(name) {
  const icons = {
    cube: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="m12 2 8 4.5v9L12 20l-8-4.5v-9L12 2Z"/><path d="m4.5 6.8 7.5 4.3 7.5-4.3M12 11v8.5"/></svg>',
    arrow: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12h14m-5-5 5 5-5 5"/></svg>',
    download: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3v12m-4-4 4 4 4-4M4 17v3h16v-3"/></svg>',
    check: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="m5 12 4 4L19 6"/></svg>',
    code: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="m8 5-6 7 6 7m8-14 6 7-6 7M14 3l-4 18"/></svg>',
    link: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M10 13a5 5 0 0 0 7.5.5l2-2a5 5 0 0 0-7-7l-1.2 1.2M14 11a5 5 0 0 0-7.5-.5l-2 2a5 5 0 0 0 7 7l1.2-1.2"/></svg>',
    server: '<svg viewBox="0 0 24 24" aria-hidden="true"><rect x="3" y="4" width="18" height="6" rx="2"/><rect x="3" y="14" width="18" height="6" rx="2"/><path d="M7 7h.01M7 17h.01"/></svg>',
    shield: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3 4.5 6v5.5c0 4.7 3.2 7.8 7.5 9.5 4.3-1.7 7.5-4.8 7.5-9.5V6L12 3Z"/><path d="m9 12 2 2 4-5"/></svg>',
  };
  return icons[name];
}

function header(lang, page, p) {
  const t = copy[lang];
  const nav = [
    ["home", t.nav.home, p.home],
    ["docs", t.nav.docs, p.docs],
    ["modules", t.nav.modules, p.modules],
    ["download", t.nav.download, p.download],
  ];
  return `<header class="site-header">
    <a class="brand" href="${p.home}" aria-label="${t.brand}">${icon("cube")}<span>${t.brand}</span></a>
    <nav class="main-nav" aria-label="Primary navigation">
      ${nav.map(([id, label, href]) => `<a ${id === page ? 'aria-current="page"' : ""} href="${href}">${label}</a>`).join("")}
    </nav>
    <div class="header-actions">
      <button class="icon-button" type="button" data-theme-toggle aria-label="${t.theme}"><span class="sun">☼</span><span class="moon">◐</span></button>
      <a class="language-link" href="${p.alternate}">${t.language}</a>
      <a class="github-link" href="${repository}" target="_blank" rel="noreferrer">${t.github}<span aria-hidden="true">↗</span></a>
      <button class="menu-button" type="button" data-menu-toggle aria-label="Menu"><span></span><span></span></button>
    </div>
  </header>`;
}

function footer(lang, p) {
  const t = copy[lang];
  return `<footer class="site-footer">
    <div><a class="brand footer-brand" href="${p.home}">${icon("cube")}<span>${t.brand}</span></a><p>${t.footer}</p></div>
    <div class="footer-links"><a href="${p.docs}">${t.nav.docs}</a><a href="${p.modules}">${t.nav.modules}</a><a href="${latestRelease}">${t.nav.download}</a><a href="${repository}">GitHub</a></div>
    <p class="copyright">© 2026 CDP Injector · Open source</p>
  </footer>`;
}

function layout({ lang, page, title, description, content, doc = false }) {
  const t = copy[lang];
  const p = paths(lang, page);
  const fullTitle = page === "home" ? `${t.brand} — ${title}` : `${title} · ${t.brand}`;
  return `<!doctype html>
<html lang="${t.locale}" data-theme="dark">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <meta name="description" content="${description}" />
  <meta name="color-scheme" content="dark light" />
  <meta name="theme-color" content="#0d0c0b" />
  <meta property="og:title" content="${fullTitle}" />
  <meta property="og:description" content="${description}" />
  <meta property="og:type" content="website" />
  <title>${fullTitle}</title>
  <link rel="icon" href="${p.icon}" />
  <link rel="stylesheet" href="${p.css}" />
  <script>try{const v=localStorage.getItem('cdp-theme');if(v)document.documentElement.dataset.theme=v;else document.documentElement.dataset.theme=matchMedia('(prefers-color-scheme: light)').matches?'light':'dark'}catch{}</script>
</head>
<body class="${doc ? "docs-page" : "marketing-page"}">
  ${header(lang, page, p)}
  <main>${content(p)}</main>
  ${footer(lang, p)}
  <script src="${p.js}" defer></script>
</body>
</html>`;
}

function homeZh(p) {
  return `<section class="hero section-shell">
    <div class="hero-copy reveal">
      <a class="version-pill" href="${latestRelease}"><span></span>v${version} 已发布 ${icon("arrow")}</a>
      <p class="eyebrow">为 Electron 应用而生</p>
      <h1>让你的应用，拥有<span>可组合的模块能力</span></h1>
      <p class="hero-lead">通过 Chrome DevTools Protocol 启动并连接 Electron 应用，在同一个 Product Session 中注入主题、面板与本地服务模块。</p>
      <div class="hero-actions"><a class="button primary" href="${latestRelease}">${icon("download")}下载 v${version}</a><a class="button secondary" href="${p.docs}">开始使用 ${icon("arrow")}</a></div>
      <div class="support-line"><span>${icon("check")}首个验收产品：Codex</span><span>macOS 12+</span><span>开源</span></div>
    </div>
    <div class="hero-visual reveal delay-1">
      <div class="orb"></div><div class="app-frame"><div class="app-titlebar"><i></i><i></i><i></i><span>CDP注入器</span></div><img src="${p.screenshot}" alt="CDP注入器模块管理与运行状态界面" /></div>
      <div class="floating-card module-card"><span class="status-dot"></span><div><strong>任务看板</strong><small>模块已注入 · 本地服务运行中</small></div></div>
      <div class="floating-card cdp-card"><span>CDP</span><div><strong>已连接</strong><small>共享 Product Session</small></div></div>
    </div>
  </section>
  <section class="trust-strip"><div class="section-shell"><span>一个应用</span><strong>一个 CDP 端口</strong><i></i><span>多个模块</span><strong>共享同一会话</strong><i></i><span>本地优先</span><strong>数据留在设备</strong></div></section>
  <section class="feature-section section-shell" id="features"><div class="section-heading"><p class="eyebrow">核心能力</p><h2>模块化，不侵入原应用</h2><p>启动、连接、注入和本地服务由一个轻量桌面启动器统一管理。</p></div>
    <div class="feature-grid">
      <article class="feature-card accent-card"><div class="feature-icon">${icon("link")}</div><span>01</span><h3>共享 Product Session</h3><p>每个应用实例只使用一个 CDP 端口。所有已启用模块复用同一连接，不重复启动目标应用。</p></article>
      <article class="feature-card"><div class="feature-icon">${icon("code")}</div><span>02</span><h3>预编译模块包</h3><p>导入本地 <code>.cdpmod</code> 即可安装。用户侧不执行 <code>npm install</code>，也不运行构建脚本。</p></article>
      <article class="feature-card"><div class="feature-icon">${icon("server")}</div><span>03</span><h3>内置 Node 服务</h3><p>需要 Web 或 API 的模块由 Hub 分配 loopback 端口，并使用随应用打包的 Node 运行时启动。</p></article>
      <article class="feature-card"><div class="feature-icon">${icon("shield")}</div><span>04</span><h3>清晰的运行状态</h3><p>区分普通启动与 CDP 启动，显示目标连接、注入数量、模块服务端口和诊断错误。</p></article>
    </div>
  </section>
  <section class="workflow-section"><div class="section-shell workflow-grid"><div class="workflow-copy"><p class="eyebrow">工作方式</p><h2>从选择模块到真实注入，只需一次启动</h2><p>无模块时保持原始启动；启用模块后，CDP注入器负责重启目标应用、建立会话并依次激活模块。</p><a class="text-link" href="${p.docs}">查看完整使用流程 ${icon("arrow")}</a></div><ol class="steps"><li><b>01</b><div><h3>选择应用与模块</h3><p>启用主题、面板或导入的本地模块。</p></div></li><li><b>02</b><div><h3>通过 CDP 启动</h3><p>启动器分配端口并连接目标 renderer。</p></div></li><li><b>03</b><div><h3>激活与诊断</h3><p>共享上下文、启动服务，并持续展示真实状态。</p></div></li></ol></div></section>
  <section class="developer-section section-shell"><div class="developer-panel"><div><p class="eyebrow">开放模块协议</p><h2>用你熟悉的工具开发，交付一个预编译包</h2><p>模块源码可使用任意前端或 Node 工具链。CDP注入器只关心稳定的 manifest、注入入口和可选服务入口。</p><div class="capability-list"><span>renderer-injection</span><span>local-service</span><span>module-data</span><span>csp-bypass</span></div><a class="button secondary" href="${p.modules}">阅读模块开发指南 ${icon("arrow")}</a></div>${codeBlock("manifest.json", `{
  "schemaVersion": 1,
  "id": "dev.example.focus-mode",
  "version": "1.0.0",
  "hubApi": 1,
  "targets": [{ "product": "codex", "context": "main" }],
  "inject": {
    "entry": "inject/index.js",
    "styles": ["inject/index.css"],
    "runAt": "document-start"
  }
}`)}</div></section>
  <section class="download-cta section-shell"><div><p class="eyebrow">从 Codex 开始</p><h2>把第一个模块装进你的 Electron 应用</h2><p>macOS + Codex 是当前完整验收路径。Windows 与 Linux 提供构建包，注入兼容仍在持续验证。</p></div><div><a class="button primary" href="${p.download}">${icon("download")}选择下载版本</a><a class="button secondary" href="${repository}">查看源代码 ↗</a></div></section>`;
}

function homeEn(p) {
  return `<section class="hero section-shell">
    <div class="hero-copy reveal"><a class="version-pill" href="${latestRelease}"><span></span>v${version} is available ${icon("arrow")}</a><p class="eyebrow">Built for Electron apps</p><h1>Give your app a<span>composable module layer</span></h1><p class="hero-lead">Launch and connect to Electron apps through the Chrome DevTools Protocol, then inject themes, panels, and local-service modules into one shared Product Session.</p><div class="hero-actions"><a class="button primary" href="${latestRelease}">${icon("download")}Download v${version}</a><a class="button secondary" href="${p.docs}">Get started ${icon("arrow")}</a></div><div class="support-line"><span>${icon("check")}First verified product: Codex</span><span>macOS 12+</span><span>Open source</span></div></div>
    <div class="hero-visual reveal delay-1"><div class="orb"></div><div class="app-frame"><div class="app-titlebar"><i></i><i></i><i></i><span>CDP Injector</span></div><img src="${p.screenshot}" alt="CDP Injector module and runtime status interface" /></div><div class="floating-card module-card"><span class="status-dot"></span><div><strong>Taskboard</strong><small>Injected · local service running</small></div></div><div class="floating-card cdp-card"><span>CDP</span><div><strong>Connected</strong><small>Shared Product Session</small></div></div></div>
  </section>
  <section class="trust-strip"><div class="section-shell"><span>One app</span><strong>One CDP port</strong><i></i><span>Many modules</span><strong>One shared session</strong><i></i><span>Local first</span><strong>Data stays on device</strong></div></section>
  <section class="feature-section section-shell"><div class="section-heading"><p class="eyebrow">Core capabilities</p><h2>Modular, without modifying the app</h2><p>A focused desktop launcher coordinates app startup, connection, injection, and local services.</p></div><div class="feature-grid"><article class="feature-card accent-card"><div class="feature-icon">${icon("link")}</div><span>01</span><h3>Shared Product Session</h3><p>Each app instance uses one CDP port. All enabled modules reuse that connection without launching duplicate app instances.</p></article><article class="feature-card"><div class="feature-icon">${icon("code")}</div><span>02</span><h3>Prebuilt packages</h3><p>Install a local <code>.cdpmod</code>. The user never runs <code>npm install</code> or a module build step.</p></article><article class="feature-card"><div class="feature-icon">${icon("server")}</div><span>03</span><h3>Bundled Node services</h3><p>The Hub allocates a loopback port and runs Web or API modules with the Node runtime bundled in the app.</p></article><article class="feature-card"><div class="feature-icon">${icon("shield")}</div><span>04</span><h3>Truthful runtime status</h3><p>See normal versus CDP launch mode, target connection, injected count, service ports, and diagnostics.</p></article></div></section>
  <section class="workflow-section"><div class="section-shell workflow-grid"><div class="workflow-copy"><p class="eyebrow">How it works</p><h2>From module selection to real injection in one launch</h2><p>With no modules, the app opens normally. Once modules are enabled, CDP Injector restarts the product, establishes a session, and activates each module.</p><a class="text-link" href="${p.docs}">Read the complete workflow ${icon("arrow")}</a></div><ol class="steps"><li><b>01</b><div><h3>Select an app and modules</h3><p>Enable a theme, panel, or imported local module.</p></div></li><li><b>02</b><div><h3>Launch through CDP</h3><p>The launcher allocates a port and connects to the target renderer.</p></div></li><li><b>03</b><div><h3>Activate and diagnose</h3><p>Share context, start services, and display live status.</p></div></li></ol></div></section>
  <section class="developer-section section-shell"><div class="developer-panel"><div><p class="eyebrow">Open module protocol</p><h2>Build with your tools. Ship one precompiled package.</h2><p>Module projects may use any frontend or Node toolchain. CDP Injector only consumes a stable manifest, an injection entry, and an optional service entry.</p><div class="capability-list"><span>renderer-injection</span><span>local-service</span><span>module-data</span><span>csp-bypass</span></div><a class="button secondary" href="${p.modules}">Read the module guide ${icon("arrow")}</a></div>${codeBlock("manifest.json", `{
  "schemaVersion": 1,
  "id": "dev.example.focus-mode",
  "version": "1.0.0",
  "hubApi": 1,
  "targets": [{ "product": "codex", "context": "main" }],
  "inject": {
    "entry": "inject/index.js",
    "styles": ["inject/index.css"],
    "runAt": "document-start"
  }
}`)}</div></section>
  <section class="download-cta section-shell"><div><p class="eyebrow">Start with Codex</p><h2>Install your first module into an Electron app</h2><p>macOS + Codex is the fully verified path today. Windows and Linux packages are available while injection compatibility remains in progress.</p></div><div><a class="button primary" href="${p.download}">${icon("download")}Choose a download</a><a class="button secondary" href="${repository}">View source ↗</a></div></section>`;
}

function docShell({ eyebrow, title, lead, toc, body }) {
  return `<section class="doc-hero section-shell"><p class="eyebrow">${eyebrow}</p><h1>${title}</h1><p>${lead}</p></section><div class="doc-layout section-shell"><aside class="doc-toc"><strong>ON THIS PAGE</strong>${toc.map(([id, label]) => `<a href="#${id}">${label}</a>`).join("")}</aside><article class="doc-prose">${body}</article></div>`;
}

function usageZh(p) {
  return docShell({ eyebrow: `使用文档 · v${version}`, title: "从安装到第一次注入", lead: "这份指南覆盖当前已验收的 macOS + Codex 路径，以及模块导入、运行状态与常见问题。", toc: [["install","安装"],["product","目标应用"],["enable","启用模块"],["launch","启动与重启"],["status","理解状态"],["import","导入模块"],["troubleshooting","故障排查"]], body: `
    <section id="install"><h2>安装</h2><p>从 GitHub Release 下载与你的 Mac 架构匹配的安装包。Apple Silicon 选择 ARM64，Intel Mac 选择 x64。</p><div class="callout warning"><strong>当前验收范围</strong><p>完整注入路径以 macOS + Codex 为准。Windows 与 Linux 构建包可下载，但应用发现、启动和进程探测仍在适配。</p></div><a class="button primary compact" href="${latestRelease}">${icon("download")}打开最新 Release</a></section>
    <section id="product"><h2>确认目标应用</h2><p>v${version} 只内置 Codex Product Profile，打开后会直接显示 Codex 的模块与运行状态。当前版本还不支持手动添加其他 Electron 应用。</p><p>启动器会在 <code>/Applications/Codex.app</code> 与兼容的 <code>/Applications/ChatGPT.app</code> 路径中发现应用。若界面无法识别 Codex，请先确认应用安装在系统“应用程序”目录。</p></section>
    <section id="enable"><h2>启用模块</h2><ol><li>进入<strong>模块</strong>页面。</li><li>打开需要的模块开关。内置模块包括 Codex 主题、橙色光框和任务看板。</li><li>如果目标应用已经运行，模块开关会进入“已就绪”，下一次通过 CDP 重启时生效；已连接时则可按当前状态注入或移除。</li></ol><p>模块状态灯含义：灰色为未启用，黄色为已就绪，绿色为运行中，红色表示模块或本地服务出错。</p></section>
    <section id="launch"><h2>启动与重启</h2><div class="state-table"><div><strong>未选择模块</strong><span>普通方式打开应用，不启用 CDP。</span></div><div><strong>已选择模块</strong><span>退出已有实例，通过 CDP 重启并注入启用的模块。</span></div><div><strong>已经 CDP 运行</strong><span>按钮显示“打开”或“重新启动”，根据模块变更决定是否重启。</span></div></div><p>Codex 已运行时，启动器会明确提示“将退出并重新启动 Codex”。未保存的应用内状态应先自行确认。</p></section>
    <section id="status"><h2>理解运行状态</h2><p>右侧运行状态从上到下表示：</p><ul><li><strong>应用：</strong>未启动、正常运行或 CDP 连接中/已连接。</li><li><strong>CDP：</strong>是否能通过探针连接已分配端口。</li><li><strong>目标：</strong>是否已经匹配到 Codex main renderer。</li><li><strong>模块：</strong>当前真实注入数量；带服务的模块还会显示 loopback 端口和浏览器入口。</li></ul><p>诊断页使用同一份状态数据。即使注入器重启，也会优先用 CDP 探针恢复仍在运行的会话状态。</p></section>
    <section id="import"><h2>导入本地 .cdpmod</h2><ol><li>点击模块页右上角<strong>导入 .cdpmod</strong>。</li><li>选择本地预编译模块包。</li><li>核对名称、版本、目标应用、能力声明以及是否包含本地服务。</li><li>确认安装后打开模块开关，再通过 CDP 启动或重启目标应用。</li></ol><div class="callout"><strong>安装边界</strong><p>安装器只解压、校验并复制模块文件，不执行 npm、安装脚本或构建命令。模块包最大解压体积为 100 MB，并拒绝符号链接、<code>node_modules</code> 和越界路径。</p></div></section>
    <section id="troubleshooting"><h2>故障排查</h2><details><summary>应用显示正常运行，但 CDP 未启用</summary><p>说明应用是普通方式启动。启用至少一个模块，然后点击“以 CDP 重启”。</p></details><details><summary>CDP 已连接，但目标一直等待中</summary><p>目标 renderer 尚未匹配。先确认使用的是受支持的 Codex 版本，再在诊断页查看目标 URL 和错误。</p></details><details><summary>任务看板一直显示正在启动</summary><p>在模块详情或运行状态中检查本地服务端口。浏览器打开服务 URL；健康检查失败或页面 404 时，重新启动模块并查看诊断错误。</p></details><details><summary>退出注入器后 Codex 仍在运行</summary><p>这是预期行为。重新打开注入器后，CDP 探针会尝试恢复连接；模块本地服务则需要由注入器重新管理。</p></details></section>` });
}

function usageEn(p) {
  return docShell({ eyebrow: `User guide · v${version}`, title: "From installation to your first injection", lead: "The current verified flow covers macOS + Codex, local module imports, runtime status, and troubleshooting.", toc: [["install","Install"],["product","Target app"],["enable","Enable modules"],["launch","Launch modes"],["status","Runtime status"],["import","Import a module"],["troubleshooting","Troubleshooting"]], body: `
    <section id="install"><h2>Install</h2><p>Download the package that matches your Mac architecture from GitHub Releases: ARM64 for Apple Silicon or x64 for Intel.</p><div class="callout warning"><strong>Current validation scope</strong><p>The complete injection path is verified on macOS + Codex. Windows and Linux packages are available, while product discovery, launch, and process detection remain in progress.</p></div><a class="button primary compact" href="${latestRelease}">${icon("download")}Open the latest release</a></section>
    <section id="product"><h2>Confirm the target app</h2><p>v${version} includes only the Codex Product Profile and opens directly on its modules and runtime status. Manually adding other Electron apps is not available yet.</p><p>The launcher discovers <code>/Applications/Codex.app</code> and the compatible <code>/Applications/ChatGPT.app</code> path. If Codex is not detected, confirm that it is installed in the system Applications directory.</p></section>
    <section id="enable"><h2>Enable modules</h2><ol><li>Open the <strong>Modules</strong> page.</li><li>Turn on the modules you need. Built-ins include Codex Theme, Orange Glow, and Taskboard.</li><li>If the app is already running normally, the module becomes ready and applies after a CDP restart.</li></ol><p>Status colors: gray is disabled, amber is ready, green is active, and red signals a module or service error.</p></section>
    <section id="launch"><h2>Launch and restart</h2><div class="state-table"><div><strong>No modules selected</strong><span>Open the app normally without CDP.</span></div><div><strong>Modules selected</strong><span>Quit the existing instance, relaunch through CDP, and inject enabled modules.</span></div><div><strong>Already on CDP</strong><span>The action becomes Open or Restart depending on pending module changes.</span></div></div><p>When Codex is already running, the launcher warns that it will quit and relaunch Codex. Confirm any unsaved app state first.</p></section>
    <section id="status"><h2>Understand runtime status</h2><ul><li><strong>App:</strong> not running, running normally, or connecting/connected through CDP.</li><li><strong>CDP:</strong> whether the probe can reach the assigned port.</li><li><strong>Target:</strong> whether the Codex main renderer is matched.</li><li><strong>Modules:</strong> the real injected count; service-backed modules also show a loopback port and browser entry.</li></ul><p>The Diagnostics page uses the same state. When Injector restarts, its CDP probe attempts to recover a still-running session.</p></section>
    <section id="import"><h2>Import a local .cdpmod</h2><ol><li>Select <strong>Import .cdpmod</strong> on the Modules page.</li><li>Choose a prebuilt local package.</li><li>Review its identity, version, targets, capability declarations, and optional service.</li><li>Confirm, enable it, then launch or restart the product through CDP.</li></ol><div class="callout"><strong>Install boundary</strong><p>The installer only validates, extracts, and copies files. It never runs npm, install scripts, or build commands. Packages are capped at 100 MB extracted and cannot contain symlinks, <code>node_modules</code>, or traversal paths.</p></div></section>
    <section id="troubleshooting"><h2>Troubleshooting</h2><details><summary>The app is running, but CDP is not enabled</summary><p>The app was opened normally. Enable at least one module, then choose “Restart with CDP”.</p></details><details><summary>CDP is connected, but the target is waiting</summary><p>No renderer matched the profile. Confirm a supported Codex version and inspect the target URL and error in Diagnostics.</p></details><details><summary>Taskboard stays on “Starting”</summary><p>Check the local service port in module details or runtime status. Open the service URL in a browser; restart the module if health fails or the page returns 404.</p></details><details><summary>Codex remains open after Injector exits</summary><p>This is expected. When Injector reopens, the CDP probe tries to recover the session. Local module services must be managed again by Injector.</p></details></section>` });
}

const manifestExample = `{
  "schemaVersion": 1,
  "id": "dev.example.focus-mode",
  "name": "Focus Mode",
  "version": "1.0.0",
  "description": "Reduce visual noise in Codex",
  "icon": "assets/icon.png",
  "hubApi": 1,
  "targets": [
    { "product": "codex", "context": "main" }
  ],
  "inject": {
    "entry": "inject/index.js",
    "styles": ["inject/index.css"],
    "runAt": "document-start"
  },
  "capabilities": ["renderer-injection"]
}`;

const lifecycleExample = `globalThis.cdpHub.register({
  id: "dev.example.focus-mode",

  async activate(context) {
    const badge = document.createElement("button");
    badge.textContent = "Focus";
    badge.dataset.cdpHubOwner = context.module.id;
    document.body.append(badge);

    return () => {
      badge.remove();
    };
  },
});`;

const serviceExample = `import { createServer } from "node:http";

const host = process.env.CDP_HUB_HOST;
const port = Number(process.env.CDP_HUB_PORT);
const token = process.env.CDP_HUB_SESSION_TOKEN;

const server = createServer((request, response) => {
  const url = new URL(request.url, \`http://\${host}:\${port}\`);
  if (url.pathname === "/health") {
    response.writeHead(200).end("ok");
    return;
  }
  if (url.searchParams.get("sessionToken") !== token) {
    response.writeHead(401).end("unauthorized");
    return;
  }
  response.writeHead(200, { "content-type": "text/html" });
  response.end("<h1>Hello from a CDP module</h1>");
});

server.listen(port, host);
process.on("SIGTERM", () => server.close());`;

function modulesZh() {
  return docShell({ eyebrow: "Hub API 1", title: "开发一个 .cdpmod 模块", lead: "模块项目可使用任意工具链；交付物必须是已经构建完成、可直接运行的 ZIP 包。", toc: [["contract","边界与结构"],["manifest","Manifest"],["renderer","Renderer 生命周期"],["service","本地服务"],["package","打包"],["test","测试与发布"]], body: `
    <section id="contract"><h2>边界与目录结构</h2><p><code>.cdpmod</code> 是扩展名不同的 ZIP 文件，<code>manifest.json</code> 必须位于压缩包根目录。第一版只接受 Codex 的 <code>main</code> renderer，并在 <code>document-start</code> 注入。</p>${codeBlock("text", `focus-mode.cdpmod/
├── manifest.json
├── inject/
│   ├── index.js
│   └── index.css
└── assets/
    └── icon.png`)}<p>需要 Web/API 的模块可额外包含 <code>service/index.mjs</code> 和已经构建完成的静态资源。不要包含 <code>node_modules</code>、符号链接、安装脚本或要求用户执行的命令。</p></section>
    <section id="manifest"><h2>Manifest</h2>${codeBlock("json", manifestExample)}<h3>必填规则</h3><ul><li><code>schemaVersion</code> 与 <code>hubApi</code> 当前都必须为 <code>1</code>。</li><li><code>id</code> 仅能包含英文字母、数字、点和连字符，推荐反向域名。</li><li><code>version</code> 必须是 SemVer。</li><li><code>targets</code> 至少包含 <code>{ product: "codex", context: "main" }</code>。</li><li><code>inject</code> 至少提供 JavaScript 入口或一个样式文件，<code>runAt</code> 必须为 <code>document-start</code>。</li><li><code>icon</code>、入口、样式和服务路径都必须真实存在于包内。</li></ul><h3>能力声明</h3><div class="capability-list docs"><span>renderer-injection</span><span>local-service</span><span>module-data</span><span>csp-bypass</span><span>external-network</span></div><p>能力会在安装前展示给用户。它们是权限披露，不代表 Node 模块已经获得操作系统级完全沙箱。</p></section>
    <section id="renderer"><h2>Renderer 生命周期</h2><p>JavaScript 入口需要用 <code>globalThis.cdpHub.register</code> 注册同一个模块 ID：</p>${codeBlock("js", lifecycleExample)}<p><code>activate(context)</code> 可以是异步函数，必须返回 cleanup 函数。cleanup 负责移除 DOM、监听器、定时器和 observer；模块创建的 DOM 应添加 <code>data-cdp-hub-owner</code>。</p>${codeBlock("ts", `type ModuleContext = {
  module: { id: string; version: string };
  product: { id: "codex" };
  target: { url: string; title: string };
  serviceUrl: string | null;
};`)}<div class="callout warning"><strong>不要依赖 Codex DOM 稳定</strong><p>模块需要自行处理 Product UI 的重新渲染，也不能依赖其他模块或注入顺序。</p></div></section>
    <section id="service"><h2>可选本地服务</h2><p>声明 <code>service</code> 时，能力列表必须包含 <code>local-service</code>：</p>${codeBlock("json", `"service": {
  "entry": "service/index.mjs",
  "healthPath": "/health",
  "readyTimeoutMs": 10000
}`)}<p>Hub 使用内置 Node 直接执行入口，不经过 shell，并注入以下环境变量：</p>${codeBlock("text", `CDP_HUB_MODULE_ID
CDP_HUB_MODULE_DIR
CDP_HUB_DATA_DIR
CDP_HUB_HOST=127.0.0.1
CDP_HUB_PORT=<allocated-port>
CDP_HUB_SESSION_TOKEN=<random-token>
CDP_HUB_PRODUCT_ID=codex`)}${codeBlock("js", serviceExample)}<p>服务必须绑定给定 loopback 地址与端口，在健康路径返回 HTTP 200，把持久数据写入 <code>CDP_HUB_DATA_DIR</code>，并在 <code>SIGTERM</code> 时清理资源。第一版避免 native Node addons。</p></section>
    <section id="package"><h2>构建与打包</h2><ol><li>在开发仓库中完成前端/服务构建。</li><li>复制运行所需的最小文件到一个 staging 目录。</li><li>保证 <code>manifest.json</code> 位于 staging 根目录。</li><li>从 staging 目录内部创建 ZIP，再将扩展名设为 <code>.cdpmod</code>。</li></ol>${codeBlock("sh", `cd staging
zip -r ../dev.example.focus-mode-1.0.0.cdpmod . \\
  -x "*.DS_Store" "__MACOSX/*" "node_modules/*"`)}<p>不要把源代码仓库外层目录一起压入包中，否则安装器无法在根目录找到 manifest。解压后总体积不能超过 100 MB，文件数不能超过 10,000。</p></section>
    <section id="test"><h2>测试与发布检查</h2><ol class="checklist"><li>包能在模块页成功预览并显示正确能力。</li><li>启用后通过 CDP 重启 Codex，运行状态中的注入数量增加。</li><li>刷新或切换 Codex renderer 后模块会重新激活。</li><li>关闭开关后 cleanup 完整，不留下 DOM、监听器或服务进程。</li><li>本地服务健康检查成功，浏览器入口可访问，数据只写入模块数据目录。</li><li>升级相同 ID 的新 SemVer 后设置和模块数据仍可用。</li></ol><p>当前没有线上模块市场。请通过 GitHub Release 等可信渠道分发预编译 <code>.cdpmod</code>，并同时提供源码与变更说明。</p></section>` });
}

function modulesEn() {
  return docShell({ eyebrow: "Hub API 1", title: "Build a .cdpmod module", lead: "Use any toolchain in your module project. The delivered ZIP must already be built and ready to run.", toc: [["contract","Contract and layout"],["manifest","Manifest"],["renderer","Renderer lifecycle"],["service","Local service"],["package","Packaging"],["test","Test and release"]], body: `
    <section id="contract"><h2>Contract and layout</h2><p>A <code>.cdpmod</code> is a ZIP with a different extension. <code>manifest.json</code> must be at its root. Hub API 1 accepts the Codex <code>main</code> renderer and injects at <code>document-start</code>.</p>${codeBlock("text", `focus-mode.cdpmod/
├── manifest.json
├── inject/
│   ├── index.js
│   └── index.css
└── assets/
    └── icon.png`)}<p>Web/API modules may also contain <code>service/index.mjs</code> and prebuilt static assets. Never ship <code>node_modules</code>, symlinks, install scripts, or commands the user must run.</p></section>
    <section id="manifest"><h2>Manifest</h2>${codeBlock("json", manifestExample)}<h3>Required rules</h3><ul><li><code>schemaVersion</code> and <code>hubApi</code> must both be <code>1</code>.</li><li><code>id</code> accepts ASCII letters, numbers, dots, and hyphens; reverse-domain notation is recommended.</li><li><code>version</code> must follow SemVer.</li><li><code>targets</code> must include <code>{ product: "codex", context: "main" }</code>.</li><li><code>inject</code> needs a script entry or at least one stylesheet; <code>runAt</code> must be <code>document-start</code>.</li><li>Every referenced icon, entry, style, and service path must exist in the archive.</li></ul><h3>Capability declarations</h3><div class="capability-list docs"><span>renderer-injection</span><span>local-service</span><span>module-data</span><span>csp-bypass</span><span>external-network</span></div><p>Capabilities are disclosed before installation. They are warnings, not a claim of complete OS-level sandboxing for arbitrary Node code.</p></section>
    <section id="renderer"><h2>Renderer lifecycle</h2><p>The JavaScript entry registers the same module ID with <code>globalThis.cdpHub.register</code>:</p>${codeBlock("js", lifecycleExample)}<p><code>activate(context)</code> may be async and must return a cleanup function. Cleanup removes DOM, listeners, timers, and observers. Add <code>data-cdp-hub-owner</code> to module-owned DOM.</p>${codeBlock("ts", `type ModuleContext = {
  module: { id: string; version: string };
  product: { id: "codex" };
  target: { url: string; title: string };
  serviceUrl: string | null;
};`)}<div class="callout warning"><strong>Do not rely on stable Codex DOM</strong><p>Modules must handle Product UI rerenders and cannot depend on another module or injection order.</p></div></section>
    <section id="service"><h2>Optional local service</h2><p>When <code>service</code> exists, capabilities must include <code>local-service</code>:</p>${codeBlock("json", `"service": {
  "entry": "service/index.mjs",
  "healthPath": "/health",
  "readyTimeoutMs": 10000
}`)}<p>The Hub invokes the entry directly with its bundled Node runtime and provides:</p>${codeBlock("text", `CDP_HUB_MODULE_ID
CDP_HUB_MODULE_DIR
CDP_HUB_DATA_DIR
CDP_HUB_HOST=127.0.0.1
CDP_HUB_PORT=<allocated-port>
CDP_HUB_SESSION_TOKEN=<random-token>
CDP_HUB_PRODUCT_ID=codex`)}${codeBlock("js", serviceExample)}<p>Bind to the supplied loopback host and port, return HTTP 200 from the health path, persist under <code>CDP_HUB_DATA_DIR</code>, and clean up on <code>SIGTERM</code>. Avoid native Node addons in the first package format.</p></section>
    <section id="package"><h2>Build and package</h2><ol><li>Complete all frontend and service builds in the developer project.</li><li>Copy only runtime files into a staging directory.</li><li>Place <code>manifest.json</code> at the staging root.</li><li>Create a ZIP from inside staging, then use the <code>.cdpmod</code> extension.</li></ol>${codeBlock("sh", `cd staging
zip -r ../dev.example.focus-mode-1.0.0.cdpmod . \\
  -x "*.DS_Store" "__MACOSX/*" "node_modules/*"`)}<p>Do not wrap staging in another directory. Extracted size is capped at 100 MB and packages may contain at most 10,000 files.</p></section>
    <section id="test"><h2>Test and release checklist</h2><ol class="checklist"><li>The Modules page previews the package and its capabilities correctly.</li><li>After a CDP restart, the injected count increases.</li><li>The module reactivates after a Codex renderer refresh or replacement.</li><li>Disabling runs complete cleanup with no leftover DOM, listeners, or service process.</li><li>The health check and browser entry work; persistent files stay in the module data directory.</li><li>A higher SemVer with the same ID keeps settings and data intact.</li></ol><p>There is no online module marketplace yet. Distribute prebuilt <code>.cdpmod</code> files through a trusted channel such as GitHub Releases, alongside source code and release notes.</p></section>` });
}

function downloadPage(lang) {
  const zh = lang === "zh";
  const labels = zh ? {
    eyebrow: `下载 · v${version}`, title: "选择适合你的安装包", lead: "所有安装包由 GitHub Actions 从同一份源代码构建。macOS + Codex 是当前完整验收路径。",
    verified: "已验收", preview: "兼容适配中", open: "在 GitHub 下载", note: "发布说明", noteBody: "macOS 当前使用 ad-hoc 签名。首次打开如遇系统拦截，请在系统设置的“隐私与安全性”中确认。正式代码签名与自动更新将在后续版本加入。",
  } : {
    eyebrow: `Download · v${version}`, title: "Choose your package", lead: "Every installer is built by GitHub Actions from the same source. macOS + Codex is the currently verified path.",
    verified: "Verified", preview: "Compatibility in progress", open: "Download on GitHub", note: "Release note", noteBody: "macOS builds currently use ad-hoc signing. If the first launch is blocked, confirm it in Privacy & Security settings. Production signing and auto-update are planned for a later release.",
  };
  const cards = [
    ["macOS", "Apple Silicon · ARM64", labels.verified, "verified"],
    ["macOS", "Intel · x64", labels.verified, "verified"],
    ["Windows", "x64 · NSIS", labels.preview, "preview"],
    ["Linux", "x64 · AppImage / DEB", labels.preview, "preview"],
  ];
  return `<section class="doc-hero download-hero section-shell"><p class="eyebrow">${labels.eyebrow}</p><h1>${labels.title}</h1><p>${labels.lead}</p></section><section class="download-grid section-shell">${cards.map(([os, arch, state, kind]) => `<article class="download-card"><div class="platform-mark">${os.slice(0, 2)}</div><div><span class="support-tag ${kind}">${state}</span><h2>${os}</h2><p>${arch}</p></div><a href="${latestRelease}">${labels.open} ${icon("arrow")}</a></article>`).join("")}</section><section class="section-shell release-note"><div class="feature-icon">${icon("shield")}</div><div><h2>${labels.note}</h2><p>${labels.noteBody}</p><a class="text-link" href="${repository}/releases/tag/v${version}">v${version} changelog ${icon("arrow")}</a></div></section>`;
}

const pages = [
  ["zh", "home", "为 Electron 应用注入本地模块", "通过共享 Product Session 为 Codex 注入主题、任务面板与本地服务模块。", homeZh],
  ["en", "home", "Local modules for Electron apps", "Inject themes, panels, and local services into Codex through one shared Product Session.", homeEn],
  ["zh", "docs", "使用文档", "CDP注入器安装、应用管理、模块启用、状态诊断与故障排查指南。", usageZh],
  ["en", "docs", "User guide", "Install CDP Injector, manage apps and modules, understand status, and troubleshoot injection.", usageEn],
  ["zh", "modules", "模块开发指南", "为 CDP注入器开发预编译 .cdpmod 模块与本地 Node 服务。", modulesZh],
  ["en", "modules", "Module development guide", "Build precompiled .cdpmod modules and optional local Node services for CDP Injector.", modulesEn],
  ["zh", "download", "下载", "下载适用于 macOS、Windows 和 Linux 的 CDP注入器安装包。", () => downloadPage("zh")],
  ["en", "download", "Download", "Download CDP Injector packages for macOS, Windows, and Linux.", () => downloadPage("en")],
];

await rm(outputDir, { recursive: true, force: true });
await mkdir(join(outputDir, "assets"), { recursive: true });
await cp(join(websiteDir, "site.css"), join(outputDir, "assets/site.css"));
await cp(join(websiteDir, "site.js"), join(outputDir, "assets/site.js"));
await cp(join(repoDir, "src-tauri/icons/128x128.png"), join(outputDir, "assets/icon.png"));
await cp(join(repoDir, "docs/plans/assets/2026-08-06-cdp-injector-ui-implementation.png"), join(outputDir, "assets/app.png"));

for (const [lang, page, title, description, content] of pages) {
  const directory = page === "home" ? join(outputDir, lang) : join(outputDir, lang, page);
  await mkdir(directory, { recursive: true });
  await writeFile(join(directory, "index.html"), layout({ lang, page, title, description, content, doc: page !== "home" }));
}

await writeFile(join(outputDir, "index.html"), `<!doctype html><html lang="zh-CN"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>CDP注入器</title><script>const lang=navigator.language?.toLowerCase().startsWith('zh')?'zh':'en';location.replace('./'+lang+'/')</script><noscript><meta http-equiv="refresh" content="0;url=./zh/"></noscript></head><body><a href="./zh/">CDP注入器</a> · <a href="./en/">CDP Injector</a></body></html>`);
await writeFile(join(outputDir, ".nojekyll"), "");

console.log(`Built CDP Injector website v${version} in ${outputDir}`);
