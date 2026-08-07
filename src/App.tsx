import { useEffect, useState, type ComponentType } from "react";
import {
  AppWindow,
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
} from "@phosphor-icons/react";
import {
  launchProduct,
  listProducts,
  prepareLaunch,
  setModuleEnabled,
  type ProductView,
} from "./api";
import "./styles.css";

type View = "modules" | "diagnostics";
type Icon = ComponentType<{ size?: number; weight?: "regular" | "bold" }>;

const previewProduct: ProductView = {
  profile: {
    id: "codex",
    name: "Codex",
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
    },
  ],
  status: { productId: "codex", phase: "not running", moduleErrors: {} },
};

const phaseLabels: Record<string, string> = {
  "not running": "未运行",
  "running normally": "正常运行",
  stopping: "正在停止",
  starting: "正在启动",
  "connecting to CDP": "正在连接 CDP",
  injecting: "正在注入",
  injected: "已注入",
  "partially failed": "部分失败",
  "launch failed": "启动失败",
};

export function App() {
  const [products, setProducts] = useState<ProductView[]>([]);
  const [view, setView] = useState<View>("modules");
  const [error, setError] = useState("");
  const [saving, setSaving] = useState("");
  const [launching, setLaunching] = useState(false);
  const [restartPending, setRestartPending] = useState(false);
  const preview = import.meta.env.DEV && location.search === "?preview=1";

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

  const product = products[0];

  if (!product) {
    return <div className="loading">{error || "正在加载产品…"}</div>;
  }

  const phase = phaseLabels[product.status.phase] ?? product.status.phase;
  const moduleErrors = Object.entries(product.status.moduleErrors);
  const cdpConnected = ["injecting", "injected", "partially failed"].includes(
    product.status.phase,
  );
  const targetReady = cdpConnected;

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <AppWindow size={19} />
          <span>CDP Injector</span>
        </div>

        <div className="product-picker">
          <Cube size={21} />
          <strong>{product.profile.name}</strong>
          <CaretDown size={15} />
        </div>

        <nav aria-label="主导航">
          <NavButton
            active={view === "modules"}
            icon={PuzzlePiece}
            label="模块"
            onClick={() => setView("modules")}
          />
          <NavButton
            active={view === "diagnostics"}
            icon={Pulse}
            label="诊断"
            onClick={() => setView("diagnostics")}
          />
        </nav>

        <button className="settings-button" type="button" disabled>
          <GearSix size={20} />
          <span>设置</span>
        </button>
      </aside>

      <main className="workspace">
        {view === "modules" ? (
          <>
            <div className="workspace-header">
              <div>
                <h1>模块管理</h1>
                <p>启动前选择要注入 Codex 的模块</p>
              </div>
              <button className="secondary-button" type="button" disabled>
                <DownloadSimple size={17} />
                导入 .cdpmod
              </button>
            </div>

            <section className="module-table" aria-label="已安装模块">
              <div className="table-heading" aria-hidden="true">
                <span>名称</span>
                <span>描述</span>
                <span>状态</span>
              </div>
              {product.modules.map((module) => {
                const enabled = module.enabledFor.includes(product.profile.id);
                const moduleError = product.status.moduleErrors[module.id];

                return (
                  <div className="module-row" key={module.id}>
                    <div className="module-name">
                      <span className="module-icon">
                        <PuzzlePiece size={22} />
                      </span>
                      <span>
                        <strong>{module.name}</strong>
                        <small>v{module.version}</small>
                      </span>
                    </div>
                    <p>{moduleDescription(module.id)}</p>
                    <div className="module-controls">
                      <span className={`module-state ${moduleError ? "error" : ""}`}>
                        <i />
                        {moduleError
                          ? "失败"
                          : enabled && product.status.phase === "injected"
                            ? "运行中"
                            : enabled
                              ? "已就绪"
                              : "未启用"}
                      </span>
                      <label className="switch">
                        <input
                          aria-label={`${enabled ? "停用" : "启用"}${module.name}`}
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
                        aria-label={`查看${module.name}详情`}
                        disabled
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
        ) : (
          <Diagnostics product={product} phase={phase} />
        )}
      </main>

      <aside className="session-panel">
        <h2>运行状态</h2>
        <div className="session-steps">
          <StatusStep icon={AppWindow} label={`Codex ${phase}`} />
          <StatusStep
            icon={Link}
            label={cdpConnected ? "CDP 已连接" : "CDP 未连接"}
          />
          <StatusStep
            icon={Clock}
            label={targetReady ? "目标已连接" : "目标等待中"}
          />
          <StatusStep
            icon={PuzzlePiece}
            label={
              product.status.phase === "injected"
                ? "模块已注入"
                : product.status.phase === "injecting"
                  ? "模块正在注入"
                  : "模块未注入"
            }
          />
        </div>

        {product.modules.some((module) => module.id.includes("taskboard")) && (
          <div className="warning-panel">
            <Warning size={19} weight="bold" />
            <p>
              <strong>任务面板</strong>
              <span>服务将在启动后运行</span>
            </p>
          </div>
        )}

        {moduleErrors.length > 0 && (
          <div className="error-panel">
            <Warning size={19} weight="bold" />
            <span>{moduleErrors[0][1]}</span>
          </div>
        )}

        <div className="launch-area">
          <p>{product.profile.preview.restartMessage}</p>
          <button
            className="primary-button"
            type="button"
            disabled={preview || launching}
            onClick={() => void requestLaunch()}
          >
            <Play size={18} weight="bold" />
            {launching ? "正在启动…" : "启动 Codex"}
          </button>
        </div>
      </aside>

      {restartPending && (
        <div className="dialog-backdrop" role="presentation">
          <section
            className="confirm-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="restart-title"
          >
            <h2 id="restart-title">重新启动 Codex？</h2>
            <p>{product.profile.preview.restartMessage}</p>
            <div className="dialog-actions">
              <button type="button" onClick={() => setRestartPending(false)}>
                取消
              </button>
              <button type="button" onClick={() => void startProduct()}>
                退出并重新启动
              </button>
            </div>
          </section>
        </div>
      )}
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

function StatusStep({ icon: IconComponent, label }: { icon: Icon; label: string }) {
  return (
    <div className="status-step">
      <span>
        <IconComponent size={21} />
      </span>
      <p>{label}</p>
    </div>
  );
}

function Diagnostics({ product, phase }: { product: ProductView; phase: string }) {
  return (
    <section className="diagnostics-view">
      <div className="workspace-header">
        <div>
          <h1>诊断</h1>
          <p>查看 Product Session 与模块错误</p>
        </div>
      </div>
      <div className="diagnostic-list">
        <DiagnosticRow icon={AppWindow} label="Codex" value={phase} />
        <DiagnosticRow icon={Link} label="CDP" value="未连接" />
        <DiagnosticRow
          icon={Package}
          label="已安装模块"
          value={`${product.modules.length} 个`}
        />
        <DiagnosticRow
          icon={TerminalWindow}
          label="模块错误"
          value={`${Object.keys(product.status.moduleErrors).length} 个`}
        />
      </div>
    </section>
  );
}

function DiagnosticRow({ icon: IconComponent, label, value }: {
  icon: Icon;
  label: string;
  value: string;
}) {
  return (
    <div className="diagnostic-row">
      <IconComponent size={20} />
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function moduleDescription(moduleId: string) {
  return moduleId === "dev.cdp-injector.codex-theme"
    ? "为 Codex 提供主题与配色"
    : "本地预编译 CDP 模块";
}
