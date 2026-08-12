# CDP注入器 / CDP Injector

CDP注入器是一个常驻桌面启动器，通过共享的 Product Session 为 Codex 注入本地 `.cdpmod` 模块。

当前运行时验收范围是 macOS + Codex。仓库可以在 macOS Intel/ARM、Windows x64 和 Linux x64 上构建安装包，但 Windows/Linux 的 Electron 应用发现、启动与进程探测仍属于后续兼容工作，当前构建不代表这些系统已经具备完整注入能力。

已包含：

- 内置 Codex 主题、橙色光框和任务看板；
- 安全检查、能力确认和本地安装 `.cdpmod`；
- 一个 Codex 实例共享一个 CDP 端口与 Product Session；
- 带本地服务的模块由内置 Node 运行时启动，无需系统 Node 或 `npm install`；
- 设置页支持手动检查、下载并安装签名更新；
- 关闭主窗口后驻留托盘，退出时停止模块服务。

## 开发

```bash
pnpm install
pnpm prepare:node
pnpm tauri dev
```

Node 二进制位于 `src-tauri/resources/node/`，只在本地或 CI 构建时从 Node.js 官方发行包下载并校验 SHA-256，不提交到 Git。

## 自动发布

- 推送到 `main` 或创建 Pull Request：构建四个平台组合并验证可打包。
- 推送形如 `v0.1.0` 的标签：自动创建 GitHub Release，上传四个平台安装包、更新签名与 `latest.json`。

```bash
git tag v0.1.0
git push origin v0.1.0
```

macOS 当前使用 ad-hoc 签名。正式分发时可再为 GitHub Actions 配置 Apple Developer 与 Windows 代码签名凭据。

自动更新使用 Tauri updater 签名。GitHub Actions 需要配置 `TAURI_SIGNING_PRIVATE_KEY`，对应私钥必须离线备份；丢失后，已安装版本无法信任用新密钥签名的更新。

## 官网与文档

官网、使用文档和模块开发指南位于 `website/`，使用 Node 标准库生成静态页面，不需要额外安装文档框架：

```bash
pnpm site:build
python3 -m http.server 4175 --directory website/dist
```

推送官网或版本号变更到 `main` 后，`.github/workflows/pages.yml` 会构建并发布 GitHub Pages。默认入口会根据浏览器语言进入中文或英文页面。
