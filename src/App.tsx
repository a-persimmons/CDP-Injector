import { useEffect, useState, type ComponentType } from "react";
import {
  AppWindow,
  ArrowSquareOut,
  Browser,
  CaretDown,
  CaretRight,
  Clock,
  Cube,
  DownloadSimple,
  GearSix,
  Link,
  Package,
  Play,
  Pulse,
  PuzzlePiece,
  TerminalWindow,
  Warning,
  X,
} from "@phosphor-icons/react";
import { getVersion } from "@tauri-apps/api/app";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { check, type Update } from "@tauri-apps/plugin-updater";
import {
  inspectModulePackage,
  installModulePackage,
  launchProduct,
  listProducts,
  openModuleService,
  prepareLaunch,
  restartAfterUpdate,
  setModuleEnabled,
  type ModulePackagePreview,
  type ProductView,
} from "./api";
import "./styles.css";

type View = "modules" | "diagnostics" | "settings";
type Language = "zh" | "en";
type Theme = "auto" | "light" | "dark";
type Icon = ComponentType<{ size?: number; weight?: "regular" | "bold" }>;

const translations = {
  zh: {
    appName: "CDP注入器",
    loading: "正在加载产品…",
    mainNavigation: "主导航",
    modules: "模块",
    diagnostics: "诊断",
    settings: "设置",
    moduleManagement: "模块管理",
    moduleSubtitle: "启动前选择要注入 Codex 的模块",
    importModule: "导入 .cdpmod",
    importTitle: "确认安装模块",
    capabilityWarning: "模块可能在 Codex renderer、本地服务或 Agent 环境中运行。安装过程不会执行代码或构建依赖。",
    targets: "目标",
    capabilities: "能力",
    applicationType: "应用类型",
    electronApp: "Electron 应用",
    codexAgent: "Codex Agent",
    agentIntegration: "Agent 集成",
    agentSkills: "Agent Skills",
    cliCommands: "CLI 命令",
    notRequired: "不需要",
    active: "已生效",
    conflict: "名称冲突",
    integrationWaiting: "等待应用启动",
    installModule: "安装模块",
    installing: "正在安装…",
    installedModules: "已安装模块",
    name: "名称",
    description: "描述",
    status: "状态",
    failed: "失败",
    running: "运行中",
    ready: "已就绪",
    disabled: "未启用",
    enable: "启用",
    disable: "停用",
    viewDetails: "查看{name}详情",
    moduleDetails: "模块详情",
    close: "关闭",
    version: "版本",
    moduleId: "模块 ID",
    targetApplication: "目标应用",
    serviceStatus: "后台服务",
    noService: "无后台服务",
    browserAccess: "浏览器访问",
    supported: "支持",
    unsupported: "不支持",
    runtimeStatus: "运行状态",
    serviceStartsLater: "服务将在启动后运行",
    openInBrowser: "在浏览器中打开",
    restartTitle: "重新启动 Codex？",
    restartMessage: "将退出并重新启动 Codex",
    cancel: "取消",
    quitAndRestart: "退出并重新启动",
    diagnosticsSubtitle: "查看 Product Session 与模块错误",
    installedModuleCount: "已安装模块",
    moduleErrors: "模块错误",
    service: "服务",
    waitingToStart: "等待启动",
    settingsSubtitle: "调整界面语言、外观与软件更新",
    language: "语言",
    languageHint: "选择 CDP注入器 的界面语言",
    chinese: "中文",
    english: "English",
    theme: "主题",
    themeHint: "自动跟随 macOS 外观设置",
    automatic: "自动",
    light: "亮色",
    dark: "暗色",
    softwareUpdate: "软件更新",
    softwareUpdateHint: "当前版本 v{version}",
    checkForUpdates: "检查更新",
    checkingForUpdates: "正在检查…",
    upToDate: "已经是最新版本",
    updateAvailable: (version: string) => `发现新版本 v${version}`,
    installUpdate: "更新并重新启动",
    downloadingUpdate: (progress: number) => `正在下载 ${progress}%`,
    updateFailed: "检查更新失败",
    updateUnavailableInPreview: "预览模式不可检查更新",
    normalLaunch: "将以普通方式启动 Codex",
    cdpLaunch: (count: number) => `将通过 CDP 启动并注入 ${count} 个模块`,
    processing: "正在处理 Codex Product Session",
    runningWithCdp: "Codex 当前通过 CDP 运行",
    restartWithCdp: "将退出并通过 CDP 重新启动 Codex",
    runningNormally: "Codex 当前正常运行",
    retryLaunch: "重试启动 Codex",
    launchWithCdp: "以 CDP 启动 Codex",
    launch: "启动 Codex",
    restart: "重新启动 Codex",
    restartViaCdp: "以 CDP 重启 Codex",
    open: "打开 Codex",
    runtime: {
      "not running": "Codex 未运行",
      stopping: "Codex 正在停止",
      starting: "Codex 正在启动",
      "launch failed": "Codex 启动失败",
      default: "Codex 正常运行",
    },
    phase: {
      "not running": "未运行",
      "running normally": "正常运行",
      stopping: "正在停止",
      starting: "正在启动",
      "connecting to CDP": "正在连接 CDP",
      injecting: "正在注入",
      injected: "已注入",
      "partially failed": "部分失败",
      "launch failed": "启动失败",
    },
    cdp: {
      "not used": "CDP 未启用",
      connecting: "CDP 连接中",
      connected: "CDP 已连接",
      disconnected: "CDP 未连接",
    },
    targetConnected: "目标已连接",
    targetWaiting: "目标等待中",
    targetDisabled: "目标未启用",
    targetDisconnected: "目标未连接",
    modulesInjected: "模块已注入",
    modulesInjecting: "模块正在注入",
    modulesNotInjected: "模块未注入",
    count: (count: number) => `${count} 个`,
  },
  en: {
    appName: "CDP Injector",
    loading: "Loading products…",
    mainNavigation: "Main navigation",
    modules: "Modules",
    diagnostics: "Diagnostics",
    settings: "Settings",
    moduleManagement: "Module management",
    moduleSubtitle: "Choose modules to inject into Codex before launch",
    importModule: "Import .cdpmod",
    importTitle: "Confirm module installation",
    capabilityWarning: "This module may run in the Codex renderer, a local service, or the Agent environment. Installation does not execute code or build dependencies.",
    targets: "Targets",
    capabilities: "Capabilities",
    applicationType: "Application type",
    electronApp: "Electron app",
    codexAgent: "Codex Agent",
    agentIntegration: "Agent integration",
    agentSkills: "Agent Skills",
    cliCommands: "CLI commands",
    notRequired: "Not required",
    active: "Active",
    conflict: "Name conflict",
    integrationWaiting: "Waiting for app launch",
    installModule: "Install module",
    installing: "Installing…",
    installedModules: "Installed modules",
    name: "Name",
    description: "Description",
    status: "Status",
    failed: "Failed",
    running: "Running",
    ready: "Ready",
    disabled: "Disabled",
    enable: "Enable ",
    disable: "Disable ",
    viewDetails: "View {name} details",
    moduleDetails: "Module details",
    close: "Close",
    version: "Version",
    moduleId: "Module ID",
    targetApplication: "Target application",
    serviceStatus: "Background service",
    noService: "No background service",
    browserAccess: "Browser access",
    supported: "Supported",
    unsupported: "Not supported",
    runtimeStatus: "Runtime status",
    serviceStartsLater: "Service starts with the module",
    openInBrowser: "Open in browser",
    restartTitle: "Restart Codex?",
    restartMessage: "Codex will quit and restart",
    cancel: "Cancel",
    quitAndRestart: "Quit and restart",
    diagnosticsSubtitle: "View Product Session and module errors",
    installedModuleCount: "Installed modules",
    moduleErrors: "Module errors",
    service: " service",
    waitingToStart: "Waiting to start",
    settingsSubtitle: "Adjust language, appearance, and software updates",
    language: "Language",
    languageHint: "Choose the language used by CDP Injector",
    chinese: "中文",
    english: "English",
    theme: "Theme",
    themeHint: "Automatic follows the macOS appearance",
    automatic: "Automatic",
    light: "Light",
    dark: "Dark",
    softwareUpdate: "Software update",
    softwareUpdateHint: "Current version v{version}",
    checkForUpdates: "Check for updates",
    checkingForUpdates: "Checking…",
    upToDate: "You're up to date",
    updateAvailable: (version: string) => `Version ${version} is available`,
    installUpdate: "Update and restart",
    downloadingUpdate: (progress: number) => `Downloading ${progress}%`,
    updateFailed: "Update check failed",
    updateUnavailableInPreview: "Updates are unavailable in preview mode",
    normalLaunch: "Codex will launch normally",
    cdpLaunch: (count: number) => `Codex will launch through CDP and inject ${count} module${count === 1 ? "" : "s"}`,
    processing: "Processing the Codex Product Session",
    runningWithCdp: "Codex is running through CDP",
    restartWithCdp: "Codex will quit and restart through CDP",
    runningNormally: "Codex is running normally",
    retryLaunch: "Retry Codex",
    launchWithCdp: "Launch Codex with CDP",
    launch: "Launch Codex",
    restart: "Restart Codex",
    restartViaCdp: "Restart Codex with CDP",
    open: "Open Codex",
    runtime: {
      "not running": "Codex not running",
      stopping: "Stopping Codex",
      starting: "Starting Codex",
      "launch failed": "Codex launch failed",
      default: "Codex running normally",
    },
    phase: {
      "not running": "Not running",
      "running normally": "Running normally",
      stopping: "Stopping",
      starting: "Starting",
      "connecting to CDP": "Connecting to CDP",
      injecting: "Injecting",
      injected: "Injected",
      "partially failed": "Partially failed",
      "launch failed": "Launch failed",
    },
    cdp: {
      "not used": "CDP disabled",
      connecting: "CDP connecting",
      connected: "CDP connected",
      disconnected: "CDP disconnected",
    },
    targetConnected: "Target connected",
    targetWaiting: "Target waiting",
    targetDisabled: "Target disabled",
    targetDisconnected: "Target disconnected",
    modulesInjected: "Modules injected",
    modulesInjecting: "Injecting modules",
    modulesNotInjected: "Modules not injected",
    count: (count: number) => `${count}`,
  },
};

type Translation = (typeof translations)[Language];

const previewProduct: ProductView = {
  profile: {
    id: "codex",
    name: "Codex",
    applicationType: "codex-agent",
    applicationPaths: [],
    processNames: [],
    contexts: [],
    preview: { supported: false, restartMessage: "将退出并重新启动 Codex" },
  },
  modules: [
    {
      id: "dev.cdp-injector.codex-theme",
      name: "Codex 主题",
      version: "0.1.0",
      enabledFor: ["codex"],
      hasService: false,
      browserAccessible: false,
      agentSkills: [],
      agentCommands: [],
      description: "为 Codex 提供主题与配色",
      capabilities: ["renderer-injection", "csp-bypass"],
    },
    {
      id: "dev.cdp-injector.codex-orange-glow",
      name: "Codex 橙色光框",
      version: "0.1.0",
      enabledFor: ["codex"],
      hasService: false,
      browserAccessible: false,
      agentSkills: [],
      agentCommands: [],
      description: "为 Codex 窗口添加橙色发光边框",
      capabilities: ["renderer-injection", "csp-bypass"],
    },
    {
      id: "dev.dashi.taskboard",
      name: "任务看板",
      version: "0.1.0",
      enabledFor: [],
      hasService: true,
      browserAccessible: true,
      agentSkills: ["manage-taskboard"],
      agentCommands: ["taskctl"],
      description: "在 Codex 中管理本地任务与工作流",
      capabilities: ["renderer-injection", "local-service", "module-data", "csp-bypass"],
    },
  ],
  services: [],
  status: {
    productId: "codex",
    phase: "not running",
    launchMode: "injected",
    cdpStatus: "connected",
    moduleErrors: {},
    agentIntegrations: {},
  },
};

export function App() {
  const [products, setProducts] = useState<ProductView[]>([]);
  const [view, setView] = useState<View>("modules");
  const [language, setLanguage] = useState<Language>(() =>
    localStorage.getItem("cdp-injector-language") === "en" ? "en" : "zh",
  );
  const [theme, setTheme] = useState<Theme>(() => {
    const stored = localStorage.getItem("cdp-injector-theme");
    return stored === "light" || stored === "dark" ? stored : "auto";
  });
  const [error, setError] = useState("");
  const [saving, setSaving] = useState("");
  const [launching, setLaunching] = useState(false);
  const [restartPending, setRestartPending] = useState(false);
  const [detailModuleId, setDetailModuleId] = useState<string | null>(null);
  const [pendingImport, setPendingImport] = useState<{
    path: string;
    module: ModulePackagePreview;
  } | null>(null);
  const [installing, setInstalling] = useState(false);
  const preview = import.meta.env.DEV && location.search === "?preview=1";
  const text = translations[language];

  useEffect(() => {
    const title = text.appName;
    localStorage.setItem("cdp-injector-language", language);
    document.documentElement.lang = language === "zh" ? "zh-CN" : "en";
    document.title = title;
    if (!preview) void getCurrentWindow().setTitle(title).catch(() => {});
  }, [language, preview, text.appName]);

  useEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const applyTheme = () => {
      document.documentElement.dataset.theme =
        theme === "auto" ? (media.matches ? "dark" : "light") : theme;
    };
    applyTheme();
    localStorage.setItem("cdp-injector-theme", theme);
    media.addEventListener("change", applyTheme);
    if (!preview) {
      void getCurrentWindow()
        .setTheme(theme === "auto" ? null : theme)
        .catch(() => {});
    }
    return () => media.removeEventListener("change", applyTheme);
  }, [preview, theme]);

  useEffect(() => {
    if (!detailModuleId) return;
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") setDetailModuleId(null);
    };
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, [detailModuleId]);

  async function reload() {
    try {
      setProducts(preview ? [previewProduct] : await listProducts());
      setError("");
    } catch (reason) {
      setError(String(reason));
    }
  }

  useEffect(() => {
    void reload();
    if (preview) return;
    const timer = window.setInterval(() => {
      void listProducts().then(setProducts).catch(() => {});
    }, 1500);
    return () => window.clearInterval(timer);
  }, []);

  async function toggleModule(
    product: ProductView,
    moduleId: string,
    enabled: boolean,
  ) {
    if (preview) {
      setProducts((current) =>
        current.map((item) => ({
          ...item,
          modules: item.modules.map((module) =>
            module.id === moduleId
              ? {
                  ...module,
                  enabledFor: enabled ? [product.profile.id] : [],
                }
              : module,
          ),
        })),
      );
      return;
    }

    setSaving(moduleId);
    try {
      await setModuleEnabled(product.profile.id, moduleId, enabled);
      await reload();
    } catch (reason) {
      setError(String(reason));
    } finally {
      setSaving("");
    }
  }

  async function startProduct() {
    setLaunching(true);
    setRestartPending(false);
    let launchError = "";
    const refreshTimer = window.setInterval(() => void reload(), 500);
    try {
      await launchProduct(product.profile.id);
    } catch (reason) {
      launchError = String(reason);
    } finally {
      window.clearInterval(refreshTimer);
      await reload();
      if (launchError) setError(launchError);
      setLaunching(false);
    }
  }

  async function requestLaunch() {
    setLaunching(true);
    try {
      const preparation = await prepareLaunch(product.profile.id);
      if (preparation.restartRequired) {
        setRestartPending(true);
        return;
      }
      await startProduct();
    } catch (reason) {
      setError(String(reason));
    } finally {
      setLaunching(false);
    }
  }

  async function openService(moduleId: string) {
    try {
      await openModuleService(moduleId);
      setError("");
    } catch (reason) {
      setError(String(reason));
    }
  }

  async function chooseModulePackage() {
    if (preview) return;
    try {
      const path = await openFileDialog({
        multiple: false,
        directory: false,
        filters: [{ name: "CDP Module", extensions: ["cdpmod"] }],
      });
      if (typeof path !== "string") return;
      const module = await inspectModulePackage(path);
      setPendingImport({ path, module });
      setError("");
    } catch (reason) {
      setError(String(reason));
    }
  }

  async function confirmModuleInstall() {
    if (!pendingImport) return;
    setInstalling(true);
    setError("");
    try {
      await installModulePackage(pendingImport.path);
      setPendingImport(null);
      await reload();
      setError("");
    } catch (reason) {
      setError(String(reason));
    } finally {
      setInstalling(false);
    }
  }

  const product = products[0];

  if (!product) {
    return <div className="loading">{error || text.loading}</div>;
  }

  const moduleErrors = Object.entries(product.status.moduleErrors);
  const enabledModuleCount = product.modules.filter((module) =>
    module.enabledFor.includes(product.profile.id),
  ).length;
  const enabledServiceModules = product.modules.filter(
    (module) =>
      module.hasService && module.enabledFor.includes(product.profile.id),
  );
  const enabledAgentModules = product.modules.filter(
    (module) =>
      (module.agentSkills.length > 0 || module.agentCommands.length > 0) &&
      module.enabledFor.includes(product.profile.id),
  );
  const injectedModuleCount =
    product.status.phase === "injected"
      ? enabledModuleCount
      : 0;
  const launchBusy =
    launching ||
    ["stopping", "starting", "connecting to CDP", "injecting"].includes(
      product.status.phase,
    );
  const launchLabel = launchBusy
    ? (text.phase[product.status.phase as keyof typeof text.phase] ?? text.phase.starting)
    : ["launch failed", "partially failed"].includes(product.status.phase)
      ? text.retryLaunch
      : product.status.phase === "not running"
        ? enabledModuleCount > 0
          ? text.launchWithCdp
          : text.launch
        : enabledModuleCount > 0
          ? product.status.launchMode === "injected"
            ? text.restart
            : text.restartViaCdp
          : text.open;
  const launchHint = launchBusy
    ? text.processing
    : product.status.phase === "not running"
      ? enabledModuleCount > 0
        ? text.cdpLaunch(enabledModuleCount)
        : text.normalLaunch
      : product.status.launchMode === "injected"
        ? text.runningWithCdp
        : enabledModuleCount > 0
          ? text.restartWithCdp
          : text.runningNormally;
  const cdpConnected = product.status.cdpStatus === "connected";
  const cdpLabel = text.cdp[product.status.cdpStatus];
  const runtimeLabel =
    text.runtime[product.status.phase as keyof typeof text.runtime] ??
    text.runtime.default;
  const targetLabel = cdpConnected
    ? text.targetConnected
    : product.status.cdpStatus === "connecting"
      ? text.targetWaiting
      : product.status.cdpStatus === "not used"
        ? text.targetDisabled
        : text.targetDisconnected;

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <AppWindow size={19} />
          <span>{text.appName}</span>
        </div>

        <div className="product-picker">
          <Cube size={21} />
          <span className="product-picker-copy">
            <strong>{product.profile.name}</strong>
            <small>{applicationTypeLabel(product.profile.applicationType, text)}</small>
          </span>
          <CaretDown size={15} />
        </div>

        <nav aria-label={text.mainNavigation}>
          <NavButton
            active={view === "modules"}
            icon={PuzzlePiece}
            label={text.modules}
            onClick={() => setView("modules")}
          />
          <NavButton
            active={view === "diagnostics"}
            icon={Pulse}
            label={text.diagnostics}
            onClick={() => setView("diagnostics")}
          />
        </nav>

        <button
          className={view === "settings" ? "settings-button active" : "settings-button"}
          type="button"
          aria-current={view === "settings" ? "page" : undefined}
          onClick={() => setView("settings")}
        >
          <GearSix size={20} />
          <span>{text.settings}</span>
        </button>
      </aside>

      <main className="workspace">
        {view === "modules" ? (
          <>
            <div className="workspace-header">
              <div>
                <h1>{text.moduleManagement}</h1>
                <p>{text.moduleSubtitle}</p>
              </div>
              <button
                className="secondary-button"
                type="button"
                disabled={preview}
                onClick={() => void chooseModulePackage()}
              >
                <DownloadSimple size={17} />
                {text.importModule}
              </button>
            </div>

            <section className="module-table" aria-label={text.installedModules}>
              <div className="table-heading" aria-hidden="true">
                <span>{text.name}</span>
                <span>{text.description}</span>
                <span>{text.status}</span>
              </div>
              {product.modules.map((module) => {
                const enabled = module.enabledFor.includes(product.profile.id);
                const moduleError = product.status.moduleErrors[module.id];
                const displayName = moduleName(module.id, language, module.name);
                const moduleState = moduleError
                  ? "error"
                  : !enabled
                    ? "disabled"
                    : product.status.phase === "injected"
                      ? "running"
                      : "ready";

                return (
                  <div className="module-row" key={module.id}>
                    <div className="module-name">
                      <span className="module-icon">
                        <PuzzlePiece size={22} />
                      </span>
                      <span>
                        <strong>{displayName}</strong>
                        <small>v{module.version}</small>
                      </span>
                    </div>
                    <p>{moduleDescription(module.id, language, module.description)}</p>
                    <div className="module-controls">
                      <span className={`module-state ${moduleState}`}>
                        <i />
                        {moduleError
                          ? text.failed
                          : enabled && product.status.phase === "injected"
                            ? text.running
                            : enabled
                              ? text.ready
                              : text.disabled}
                      </span>
                      <label className="switch">
                        <input
                          aria-label={`${enabled ? text.disable : text.enable}${displayName}`}
                          type="checkbox"
                          checked={enabled}
                          disabled={saving === module.id}
                          onChange={(event) =>
                            void toggleModule(
                              product,
                              module.id,
                              event.currentTarget.checked,
                            )
                          }
                        />
                        <span />
                      </label>
                      <button
                        className="disclosure"
                        type="button"
                        aria-label={text.viewDetails.replace("{name}", displayName)}
                        onClick={() => setDetailModuleId(module.id)}
                      >
                        <CaretRight size={17} />
                      </button>
                    </div>
                    {moduleError && <p className="module-error">{moduleError}</p>}
                  </div>
                );
              })}
            </section>

            {error && <p className="error-message">{error}</p>}
          </>
        ) : view === "diagnostics" ? (
          <Diagnostics
            product={product}
            language={language}
            text={text}
            runtimeLabel={runtimeLabel.replace(/^Codex\s/, "")}
            cdpLabel={cdpLabel.replace(/^CDP\s/, "")}
          />
        ) : (
          <Settings
            language={language}
            theme={theme}
            text={text}
            preview={preview}
            onLanguageChange={setLanguage}
            onThemeChange={setTheme}
          />
        )}
      </main>

      <aside className="session-panel">
        <h2>{text.runtimeStatus}</h2>
        <div className="session-steps">
          <StatusStep
            icon={AppWindow}
            label={runtimeLabel}
            badge={product.status.launchMode === "injected" ? "CDP" : undefined}
          />
          <StatusStep icon={Link} label={cdpLabel} />
          <StatusStep icon={Clock} label={targetLabel} />
          <StatusStep
            icon={PuzzlePiece}
            badge={injectedModuleCount}
            label={
              product.status.phase === "injected"
                ? text.modulesInjected
                : product.status.phase === "injecting"
                  ? text.modulesInjecting
                  : text.modulesNotInjected
            }
          />
        </div>

        {enabledServiceModules.map((module) => {
          const service = product.services.find(
            (candidate) => candidate.moduleId === module.id,
          );
          return (
            <div
              className={service ? "service-panel running" : "service-panel"}
              key={module.id}
            >
              <Browser size={19} />
              <p>
                <strong>{moduleName(module.id, language, module.name)}</strong>
                <span>
                  {service
                    ? `${service.host}:${service.port}`
                    : text.serviceStartsLater}
                </span>
              </p>
              {service && module.browserAccessible && (
                <button
                  className="service-open-button"
                  type="button"
                  aria-label={`${text.openInBrowser}: ${moduleName(module.id, language, module.name)}`}
                  title={text.openInBrowser}
                  onClick={() => void openService(module.id)}
                >
                  <ArrowSquareOut size={16} />
                </button>
              )}
            </div>
          );
        })}

        {enabledAgentModules.map((module) => {
          const integration = product.status.agentIntegrations[module.id];
          return (
            <div
              className={integration && !integration.error ? "service-panel running" : "service-panel"}
              key={`${module.id}-agent`}
            >
              <TerminalWindow size={19} />
              <p>
                <strong>{text.agentIntegration}</strong>
                <span>
                  {integration
                    ? [
                        module.agentSkills.length > 0
                          ? `${text.agentSkills}: ${integrationStatusLabel(integration.skillStatus, text)}`
                          : "",
                        module.agentCommands.length > 0
                          ? `${text.cliCommands}: ${integrationStatusLabel(integration.commandStatus, text)}`
                          : "",
                      ].filter(Boolean).join(" · ")
                    : text.integrationWaiting}
                </span>
              </p>
            </div>
          );
        })}

        {moduleErrors.length > 0 && (
          <div className="error-panel">
            <Warning size={19} weight="bold" />
            <span>{moduleErrors[0][1]}</span>
          </div>
        )}

        <div className="launch-area">
          <p>{launchHint}</p>
          <button
            className="primary-button"
            type="button"
            disabled={preview || launchBusy}
            onClick={() => void requestLaunch()}
          >
            <Play size={18} weight="bold" />
            {launchLabel}
          </button>
        </div>
      </aside>

      {detailModuleId && (
        <ModuleDetailsDialog
          module={product.modules.find((module) => module.id === detailModuleId)}
          product={product}
          language={language}
          text={text}
          onClose={() => setDetailModuleId(null)}
          onOpenService={openService}
        />
      )}

      {restartPending && (
        <div className="dialog-backdrop" role="presentation">
          <section
            className="confirm-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="restart-title"
          >
            <h2 id="restart-title">{text.restartTitle}</h2>
            <p>{text.restartMessage}</p>
            <div className="dialog-actions">
              <button type="button" onClick={() => setRestartPending(false)}>
                {text.cancel}
              </button>
              <button type="button" onClick={() => void startProduct()}>
                {text.quitAndRestart}
              </button>
            </div>
          </section>
        </div>
      )}

      {pendingImport && (
        <div className="dialog-backdrop" role="presentation">
          <section
            className="confirm-dialog import-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="import-title"
          >
            <button
              className="dialog-close-button import-dialog-close"
              type="button"
              aria-label={text.close}
              onClick={() => setPendingImport(null)}
            >
              <X size={18} />
            </button>
            <h2 id="import-title">{text.importTitle}</h2>
            <div className="import-module-heading">
              <span className="module-icon"><PuzzlePiece size={22} /></span>
              <div>
                <strong>{pendingImport.module.name}</strong>
                <small>v{pendingImport.module.version} · {pendingImport.module.id}</small>
              </div>
            </div>
            <p>{pendingImport.module.description}</p>
            <p className="capability-warning">{text.capabilityWarning}</p>
            <dl className="import-metadata">
              <div><dt>{text.targets}</dt><dd>{pendingImport.module.targets.join(", ")}</dd></div>
              <div><dt>{text.capabilities}</dt><dd>{pendingImport.module.capabilities.join(", ")}</dd></div>
              {pendingImport.module.agentSkills.length > 0 && (
                <div><dt>{text.agentSkills}</dt><dd>{pendingImport.module.agentSkills.join(", ")}</dd></div>
              )}
              {pendingImport.module.agentCommands.length > 0 && (
                <div><dt>{text.cliCommands}</dt><dd>{pendingImport.module.agentCommands.join(", ")}</dd></div>
              )}
            </dl>
            {error && <p className="module-detail-error">{error}</p>}
            <div className="dialog-actions">
              <button type="button" disabled={installing} onClick={() => setPendingImport(null)}>
                {text.cancel}
              </button>
              <button type="button" disabled={installing} onClick={() => void confirmModuleInstall()}>
                {installing ? text.installing : text.installModule}
              </button>
            </div>
          </section>
        </div>
      )}
    </div>
  );
}

function ModuleDetailsDialog({
  module,
  product,
  language,
  text,
  onClose,
  onOpenService,
}: {
  module: ProductView["modules"][number] | undefined;
  product: ProductView;
  language: Language;
  text: Translation;
  onClose: () => void;
  onOpenService: (moduleId: string) => Promise<void>;
}) {
  if (!module) return null;

  const enabled = module.enabledFor.includes(product.profile.id);
  const error = product.status.moduleErrors[module.id];
  const service = product.services.find(
    (candidate) => candidate.moduleId === module.id,
  );
  const integration = product.status.agentIntegrations[module.id];
  const state = error
    ? "error"
    : !enabled
      ? "disabled"
      : product.status.phase === "injected"
        ? "running"
        : "ready";
  const stateLabel = error
    ? text.failed
    : state === "running"
      ? text.running
      : state === "ready"
        ? text.ready
        : text.disabled;
  const displayName = moduleName(module.id, language, module.name);

  return (
    <div
      className="dialog-backdrop"
      role="presentation"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) onClose();
      }}
    >
      <section
        className="confirm-dialog module-detail-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="module-detail-title"
        aria-describedby="module-detail-description"
      >
        <header className="module-detail-header">
          <span className="module-icon">
            <PuzzlePiece size={22} />
          </span>
          <span>
            <small>{text.moduleDetails}</small>
            <h2 id="module-detail-title">{displayName}</h2>
          </span>
          <button
            className="dialog-close-button"
            type="button"
            aria-label={text.close}
            autoFocus
            onClick={onClose}
          >
            <X size={17} />
          </button>
        </header>

        <p id="module-detail-description" className="module-detail-description">
          {moduleDescription(module.id, language, module.description)}
        </p>

        <dl className="module-detail-list">
          <div>
            <dt>{text.status}</dt>
            <dd className={`module-state ${state}`}>
              <i />
              {stateLabel}
            </dd>
          </div>
          <div>
            <dt>{text.version}</dt>
            <dd>v{module.version}</dd>
          </div>
          <div>
            <dt>{text.moduleId}</dt>
            <dd className="monospace-value">{module.id}</dd>
          </div>
          <div>
            <dt>{text.targetApplication}</dt>
            <dd>{product.profile.name} · {applicationTypeLabel(product.profile.applicationType, text)}</dd>
          </div>
          <div>
            <dt>{text.serviceStatus}</dt>
            <dd>
              {service
                ? `${service.host}:${service.port}`
                : module.hasService && enabled
                  ? text.waitingToStart
                  : module.hasService
                    ? text.disabled
                    : text.noService}
            </dd>
          </div>
          <div>
            <dt>{text.browserAccess}</dt>
            <dd>{module.browserAccessible ? text.supported : text.unsupported}</dd>
          </div>
          {module.agentSkills.length > 0 && (
            <div>
              <dt>{text.agentSkills}</dt>
              <dd>{module.agentSkills.join(", ")} · {integration ? integrationStatusLabel(integration.skillStatus, text) : text.integrationWaiting}</dd>
            </div>
          )}
          {module.agentCommands.length > 0 && (
            <div>
              <dt>{text.cliCommands}</dt>
              <dd>{module.agentCommands.join(", ")} · {integration ? integrationStatusLabel(integration.commandStatus, text) : text.integrationWaiting}</dd>
            </div>
          )}
        </dl>

        {error && <p className="module-detail-error">{error}</p>}

        <div className="dialog-actions">
          {service && module.browserAccessible && (
            <button type="button" onClick={() => void onOpenService(module.id)}>
              <ArrowSquareOut size={16} />
              {text.openInBrowser}
            </button>
          )}
          <button type="button" onClick={onClose}>
            {text.close}
          </button>
        </div>
      </section>
    </div>
  );
}

function NavButton({
  active,
  icon: IconComponent,
  label,
  onClick,
}: {
  active: boolean;
  icon: Icon;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      className={active ? "nav-button active" : "nav-button"}
      type="button"
      aria-current={active ? "page" : undefined}
      onClick={onClick}
    >
      <IconComponent size={20} />
      <span>{label}</span>
    </button>
  );
}

function StatusStep({
  icon: IconComponent,
  label,
  badge,
}: {
  icon: Icon;
  label: string;
  badge?: number | string;
}) {
  return (
    <div className="status-step">
      <span>
        <IconComponent size={21} />
        {badge !== undefined && <small>{badge}</small>}
      </span>
      <p>{label}</p>
    </div>
  );
}

function Diagnostics({
  product,
  language,
  text,
  runtimeLabel,
  cdpLabel,
}: {
  product: ProductView;
  language: Language;
  text: Translation;
  runtimeLabel: string;
  cdpLabel: string;
}) {
  return (
    <section className="diagnostics-view">
      <div className="workspace-header">
        <div>
          <h1>{text.diagnostics}</h1>
          <p>{text.diagnosticsSubtitle}</p>
        </div>
      </div>
      <div className="diagnostic-list">
        <DiagnosticRow
          icon={Cube}
          label={text.applicationType}
          value={applicationTypeLabel(product.profile.applicationType, text)}
        />
        <DiagnosticRow
          icon={AppWindow}
          label="Codex"
          value={runtimeLabel}
          badge={product.status.launchMode === "injected" ? "CDP" : undefined}
        />
        <DiagnosticRow icon={Link} label="CDP" value={cdpLabel} />
        <DiagnosticRow
          icon={Package}
          label={text.installedModuleCount}
          value={text.count(product.modules.length)}
        />
        <DiagnosticRow
          icon={TerminalWindow}
          label={text.moduleErrors}
          value={text.count(Object.keys(product.status.moduleErrors).length)}
        />
        {product.modules
          .filter((module) => module.hasService)
          .map((module) => {
            const service = product.services.find(
              (candidate) => candidate.moduleId === module.id,
            );
            const enabled = module.enabledFor.includes(product.profile.id);
            return (
              <DiagnosticRow
                icon={Browser}
                key={module.id}
                label={`${moduleName(module.id, language, module.name)}${text.service}`}
                value={
                  service
                    ? `${service.host}:${service.port}`
                    : enabled
                      ? text.waitingToStart
                      : text.disabled
                }
              />
            );
          })}
        {product.modules
          .filter((module) => module.agentSkills.length > 0 || module.agentCommands.length > 0)
          .map((module) => {
            const integration = product.status.agentIntegrations[module.id];
            const enabled = module.enabledFor.includes(product.profile.id);
            return (
              <DiagnosticRow
                icon={TerminalWindow}
                key={`${module.id}-agent`}
                label={`${moduleName(module.id, language, module.name)} ${text.agentIntegration}`}
                value={
                  integration
                    ? integration.error ?? text.active
                    : enabled
                      ? text.integrationWaiting
                      : text.disabled
                }
              />
            );
          })}
      </div>
    </section>
  );
}

function Settings({
  language,
  theme,
  text,
  preview,
  onLanguageChange,
  onThemeChange,
}: {
  language: Language;
  theme: Theme;
  text: Translation;
  preview: boolean;
  onLanguageChange: (language: Language) => void;
  onThemeChange: (theme: Theme) => void;
}) {
  const [currentVersion, setCurrentVersion] = useState("…");
  const [availableUpdate, setAvailableUpdate] = useState<Update | null>(null);
  const [updateStatus, setUpdateStatus] = useState<
    "idle" | "checking" | "current" | "available" | "downloading" | "failed"
  >("idle");
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [updateError, setUpdateError] = useState("");

  useEffect(() => {
    if (preview) {
      setCurrentVersion("0.1.3");
      return;
    }
    void getVersion().then(setCurrentVersion).catch(() => setCurrentVersion("—"));
  }, [preview]);

  const checkForUpdates = async () => {
    if (preview) {
      setUpdateStatus("failed");
      setUpdateError(text.updateUnavailableInPreview);
      return;
    }
    setUpdateStatus("checking");
    setUpdateError("");
    try {
      const update = await check();
      setAvailableUpdate(update);
      setUpdateStatus(update ? "available" : "current");
    } catch (reason) {
      setUpdateStatus("failed");
      setUpdateError(String(reason));
    }
  };

  const installUpdate = async () => {
    if (!availableUpdate) return;
    setUpdateStatus("downloading");
    setDownloadProgress(0);
    setUpdateError("");
    let downloaded = 0;
    let total = 0;
    try {
      await availableUpdate.downloadAndInstall((event) => {
        if (event.event === "Started") total = event.data.contentLength ?? 0;
        if (event.event === "Progress") {
          downloaded += event.data.chunkLength;
          if (total > 0) setDownloadProgress(Math.min(100, Math.round((downloaded / total) * 100)));
        }
        if (event.event === "Finished") setDownloadProgress(100);
      });
      await restartAfterUpdate();
    } catch (reason) {
      setUpdateStatus("failed");
      setUpdateError(String(reason));
    }
  };

  const updateMessage = updateStatus === "current"
    ? text.upToDate
    : updateStatus === "available" && availableUpdate
      ? text.updateAvailable(availableUpdate.version)
      : updateStatus === "downloading"
        ? text.downloadingUpdate(downloadProgress)
        : updateStatus === "failed"
          ? updateError || text.updateFailed
          : "";

  return (
    <section className="settings-view">
      <div className="workspace-header">
        <div>
          <h1>{text.settings}</h1>
          <p>{text.settingsSubtitle}</p>
        </div>
      </div>
      <div className="settings-list">
        <label className="setting-row">
          <span>
            <strong>{text.language}</strong>
            <small>{text.languageHint}</small>
          </span>
          <select
            value={language}
            onChange={(event) => onLanguageChange(event.currentTarget.value as Language)}
          >
            <option value="zh">{text.chinese}</option>
            <option value="en">{text.english}</option>
          </select>
        </label>
        <label className="setting-row">
          <span>
            <strong>{text.theme}</strong>
            <small>{text.themeHint}</small>
          </span>
          <select
            value={theme}
            onChange={(event) => onThemeChange(event.currentTarget.value as Theme)}
          >
            <option value="auto">{text.automatic}</option>
            <option value="light">{text.light}</option>
            <option value="dark">{text.dark}</option>
          </select>
        </label>
        <div className="setting-row">
          <span>
            <strong>{text.softwareUpdate}</strong>
            <small>{updateMessage || text.softwareUpdateHint.replace("{version}", currentVersion)}</small>
          </span>
          <button
            className="setting-action"
            type="button"
            disabled={updateStatus === "checking" || updateStatus === "downloading"}
            onClick={() => void (updateStatus === "available" ? installUpdate() : checkForUpdates())}
          >
            {updateStatus === "checking"
              ? text.checkingForUpdates
              : updateStatus === "available"
                ? text.installUpdate
                : updateStatus === "downloading"
                  ? text.downloadingUpdate(downloadProgress)
                  : text.checkForUpdates}
          </button>
        </div>
      </div>
    </section>
  );
}

function DiagnosticRow({ icon: IconComponent, label, value, badge }: {
  icon: Icon;
  label: string;
  value: string;
  badge?: string;
}) {
  return (
    <div className="diagnostic-row">
      <IconComponent size={20} />
      <span>{label}</span>
      <span className="diagnostic-value">
        <strong>{value}</strong>
        {badge && <small>{badge}</small>}
      </span>
    </div>
  );
}

function moduleName(moduleId: string, language: Language, fallback: string) {
  const names: Record<string, [string, string]> = {
    "dev.cdp-injector.codex-theme": ["Codex 主题", "Codex Theme"],
    "dev.cdp-injector.codex-orange-glow": ["Codex 橙色光框", "Codex Orange Glow"],
    "dev.dashi.taskboard": ["任务看板", "Taskboard"],
  };
  return names[moduleId]?.[language === "zh" ? 0 : 1] ?? fallback;
}

function moduleDescription(moduleId: string, language: Language, fallback: string) {
  const english = language === "en";
  if (moduleId === "dev.cdp-injector.codex-theme") {
    return english ? "Provides themes and colors for Codex" : "为 Codex 提供主题与配色";
  }
  if (moduleId === "dev.cdp-injector.codex-orange-glow") {
    return english
      ? "Adds a glowing orange border to the Codex window"
      : "为 Codex 窗口添加橙色发光边框";
  }
  if (moduleId === "dev.dashi.taskboard") {
    return english
      ? "Manages local tasks and workflows in Codex"
      : "在 Codex 中管理本地任务与工作流";
  }
  return fallback || (english ? "Local precompiled CDP module" : "本地预编译 CDP 模块");
}

function applicationTypeLabel(
  applicationType: ProductView["profile"]["applicationType"],
  text: Translation,
) {
  return applicationType === "codex-agent" ? text.codexAgent : text.electronApp;
}

function integrationStatusLabel(status: string, text: Translation) {
  if (status === "active") return text.active;
  if (status === "conflict") return text.conflict;
  if (status === "not required") return text.notRequired;
  return text.failed;
}
