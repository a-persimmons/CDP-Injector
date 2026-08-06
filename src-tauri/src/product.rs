use std::{
    net::TcpListener,
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::{Duration, Instant},
};

use crate::model::{LaunchPreparation, ProductProfile};
pub use crate::model::LaunchMode;

pub fn decide_launch(has_enabled_modules: bool, is_running: bool) -> (LaunchMode, bool) {
    if has_enabled_modules {
        (LaunchMode::Injected, is_running)
    } else {
        (LaunchMode::Normal, false)
    }
}

pub fn prepare_launch(
    profile: &ProductProfile,
    has_enabled_modules: bool,
) -> Result<LaunchPreparation, String> {
    let _ = resolve_application(profile)?;
    let (mode, restart_required) =
        decide_launch(has_enabled_modules, is_product_running(profile)?);
    Ok(LaunchPreparation {
        mode,
        restart_required,
    })
}

pub fn launch_product(
    profile: &ProductProfile,
    mode: LaunchMode,
) -> Result<Option<u16>, String> {
    let application = resolve_application(profile)?;

    if mode == LaunchMode::Normal {
        run_open(&application, &[])?;
        return Ok(None);
    }

    if is_product_running(profile)? {
        request_quit(&application)?;
        wait_for_exit(profile, Duration::from_secs(15))?;
    }

    let port = reserve_loopback_port()?;
    run_open(
        &application,
        &[
            format!("--remote-debugging-port={port}"),
            "--remote-allow-origins=*".into(),
        ],
    )?;
    Ok(Some(port))
}

fn resolve_application(profile: &ProductProfile) -> Result<PathBuf, String> {
    profile
        .application_paths
        .iter()
        .map(PathBuf::from)
        .find(|path| path.is_dir())
        .ok_or_else(|| format!("未找到已安装的 {}", profile.name))
}

fn is_product_running(profile: &ProductProfile) -> Result<bool, String> {
    for process_name in &profile.process_names {
        let status = Command::new("/usr/bin/pgrep")
            .arg("-x")
            .arg(process_name)
            .status()
            .map_err(|error| format!("无法检查 {} 进程：{error}", profile.name))?;
        if status.success() {
            return Ok(true);
        }
    }
    Ok(false)
}

fn request_quit(application: &Path) -> Result<(), String> {
    let name = application
        .file_stem()
        .and_then(|name| name.to_str())
        .ok_or("应用名称无效")?;
    let status = Command::new("/usr/bin/osascript")
        .arg("-e")
        .arg(format!("tell application \"{name}\" to quit"))
        .status()
        .map_err(|error| format!("无法退出 {name}：{error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("无法正常退出 {name}"))
    }
}

fn wait_for_exit(profile: &ProductProfile, timeout: Duration) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if !is_product_running(profile)? {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(250));
    }
    Err(format!("请手动退出 {} 后重试", profile.name))
}

fn reserve_loopback_port() -> Result<u16, String> {
    TcpListener::bind(("127.0.0.1", 0))
        .and_then(|listener| listener.local_addr())
        .map(|address| address.port())
        .map_err(|error| format!("无法分配 CDP 端口：{error}"))
}

fn run_open(application: &Path, cdp_args: &[String]) -> Result<(), String> {
    let mut command = Command::new("/usr/bin/open");
    if !cdp_args.is_empty() {
        command.arg("-n");
    }
    command.arg("-a").arg(application);
    if !cdp_args.is_empty() {
        command.arg("--args");
        for argument in cdp_args {
            command.arg(argument);
        }
    }
    let status = command
        .status()
        .map_err(|error| format!("无法启动应用：{error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("应用启动失败".into())
    }
}

#[cfg(test)]
mod tests {
    use super::{decide_launch, LaunchMode};

    #[test]
    fn launch_decision() {
        assert_eq!(decide_launch(false, false), (LaunchMode::Normal, false));
        assert_eq!(decide_launch(true, false), (LaunchMode::Injected, false));
        assert_eq!(decide_launch(true, true), (LaunchMode::Injected, true));
    }
}
