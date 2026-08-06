import { useEffect, useState } from "react";
import {
  listProducts,
  setModuleEnabled,
  type ProductView,
} from "./api";
import "./styles.css";

export function App() {
  const [products, setProducts] = useState<ProductView[]>([]);
  const [error, setError] = useState("");
  const [saving, setSaving] = useState(false);

  async function reload() {
    try {
      setProducts(await listProducts());
      setError("");
    } catch (reason) {
      setError(String(reason));
    }
  }

  useEffect(() => {
    void reload();
  }, []);

  async function toggleTheme(product: ProductView, enabled: boolean) {
    setSaving(true);
    try {
      await setModuleEnabled(product.profile.id, product.modules[0].id, enabled);
      await reload();
    } catch (reason) {
      setError(String(reason));
    } finally {
      setSaving(false);
    }
  }

  return (
    <main>
      <header>
        <h1>CDP Injector</h1>
        <p>CDP 注入器</p>
      </header>

      {products.map((product) => {
        const theme = product.modules[0];
        const enabled = theme.enabledFor.includes(product.profile.id);

        return (
          <article className="product-card" key={product.profile.id}>
            <div className="product-icon" aria-hidden="true">
              C
            </div>
            <div className="product-details">
              <div className="product-title">
                <h2>{product.profile.name}</h2>
                {!product.profile.preview.supported && (
                  <span className="badge">不支持预览</span>
                )}
              </div>
              <p className="status">状态：{product.status.phase}</p>
              <label className="module-toggle">
                <span>{theme.name}</span>
                <input
                  type="checkbox"
                  checked={enabled}
                  disabled={saving}
                  onChange={(event) =>
                    void toggleTheme(product, event.currentTarget.checked)
                  }
                />
              </label>
            </div>
            <button type="button" disabled>
              启动 Codex
            </button>
          </article>
        );
      })}

      {error && <p className="error">{error}</p>}
    </main>
  );
}
