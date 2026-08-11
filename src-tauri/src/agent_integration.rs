use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{model::AgentIntegrationStatus, module_package::AgentIntegrationSpec};

pub struct AgentArtifacts {
    skill_links: Vec<(PathBuf, PathBuf)>,
    command_paths: Vec<PathBuf>,
    command_links: Vec<(PathBuf, PathBuf)>,
}

pub fn activate(
    module_root: &Path,
    spec: &AgentIntegrationSpec,
    skills_root: &Path,
    runtime_bin: &Path,
    command_bin: Option<&Path>,
    node: &Path,
    service_url: Option<&str>,
) -> (AgentArtifacts, AgentIntegrationStatus) {
    let mut artifacts = AgentArtifacts {
        skill_links: vec![],
        command_paths: vec![],
        command_links: vec![],
    };
    let mut errors = Vec::new();
    let mut mounted_skills = 0;
    let mut mounted_commands = 0;
    let mut command_conflicted = false;

    for skill in &spec.skills {
        let source = module_root.join(&skill.path);
        let destination = skills_root.join(&skill.name);
        match mount_skill(&source, &destination) {
            Ok(()) => {
                mounted_skills += 1;
                artifacts.skill_links.push((destination, source));
            }
            Err(error) => errors.push(error),
        }
    }

    for command in &spec.commands {
        let entry = module_root.join(&command.entry);
        match write_command_wrapper(runtime_bin, &command.name, node, &entry, service_url) {
            Ok(path) => {
                if let Some(command_bin) = command_bin {
                    let destination = command_bin.join(&command.name);
                    if let Err(error) = mount_command(&path, &destination) {
                        let _ = fs::remove_file(&path);
                        command_conflicted = true;
                        errors.push(error);
                        continue;
                    }
                    artifacts.command_links.push((destination, path.clone()));
                }
                mounted_commands += 1;
                artifacts.command_paths.push(path);
            }
            Err(error) => errors.push(error),
        }
    }

    let skill_status = artifact_status(spec.skills.len(), mounted_skills, "conflict");
    let command_status = artifact_status(
        spec.commands.len(),
        mounted_commands,
        if command_conflicted {
            "conflict"
        } else {
            "error"
        },
    );
    let status = AgentIntegrationStatus {
        skill_status: skill_status.into(),
        command_status: command_status.into(),
        skills: spec.skills.iter().map(|skill| skill.name.clone()).collect(),
        commands: spec
            .commands
            .iter()
            .map(|command| command.name.clone())
            .collect(),
        error: (!errors.is_empty()).then(|| errors.join("；")),
    };
    (artifacts, status)
}

pub fn deactivate(artifacts: AgentArtifacts) {
    for (link, source) in artifacts.command_links {
        if fs::read_link(&link).ok().as_deref() == Some(source.as_path()) {
            let _ = fs::remove_file(link);
        }
    }
    for path in artifacts.command_paths {
        let _ = fs::remove_file(path);
    }
    for (link, source) in artifacts.skill_links {
        if fs::read_link(&link).ok().as_deref() == Some(source.as_path()) {
            let _ = fs::remove_file(link);
        }
    }
}

fn artifact_status(total: usize, active: usize, failed: &str) -> &'static str {
    if total == 0 {
        "not required"
    } else if total == active {
        "active"
    } else {
        match failed {
            "conflict" => "conflict",
            _ => "error",
        }
    }
}

fn mount_skill(source: &Path, destination: &Path) -> Result<(), String> {
    if !source.join("SKILL.md").is_file() {
        return Err(format!("Skill 缺少 SKILL.md：{}", source.display()));
    }
    if let Ok(target) = fs::read_link(destination) {
        return if target == source {
            Ok(())
        } else {
            Err(format!("Skill 名称冲突：{}", destination.display()))
        };
    }
    if destination.exists() {
        return Err(format!("Skill 名称冲突：{}", destination.display()));
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    create_directory_link(source, destination).map_err(|error| error.to_string())
}

fn mount_command(source: &Path, destination: &Path) -> Result<(), String> {
    if let Ok(target) = fs::read_link(destination) {
        return if target == source {
            Ok(())
        } else {
            Err(format!("命令名称冲突：{}", destination.display()))
        };
    }
    if destination.exists() {
        return Err(format!("命令名称冲突：{}", destination.display()));
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    create_file_link(source, destination).map_err(|error| error.to_string())
}

#[cfg(unix)]
fn create_directory_link(source: &Path, destination: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(source, destination)
}

#[cfg(unix)]
fn create_file_link(source: &Path, destination: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(source, destination)
}

#[cfg(windows)]
fn create_directory_link(source: &Path, destination: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(source, destination)
}

#[cfg(windows)]
fn create_file_link(source: &Path, destination: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(source, destination)
}

fn write_command_wrapper(
    runtime_bin: &Path,
    name: &str,
    node: &Path,
    entry: &Path,
    service_url: Option<&str>,
) -> Result<PathBuf, String> {
    if !entry.is_file() {
        return Err(format!("命令入口不存在：{}", entry.display()));
    }
    fs::create_dir_all(runtime_bin).map_err(|error| error.to_string())?;
    #[cfg(windows)]
    let path = runtime_bin.join(format!("{name}.cmd"));
    #[cfg(not(windows))]
    let path = runtime_bin.join(name);
    let service_url = service_url.unwrap_or_default();

    #[cfg(windows)]
    let contents = format!(
        "@echo off\r\nset \"CDP_MODULE_SERVICE_URL={}\"\r\n\"{}\" \"{}\" %*\r\n",
        service_url.replace('%', "%%"),
        node.display(),
        entry.display(),
    );
    #[cfg(not(windows))]
    let contents = format!(
        "#!/bin/sh\nexport CDP_MODULE_SERVICE_URL={}\nexec {} {} \"$@\"\n",
        shell_quote(service_url),
        shell_quote(&node.to_string_lossy()),
        shell_quote(&entry.to_string_lossy()),
    );
    fs::write(&path, contents).map_err(|error| error.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755))
            .map_err(|error| error.to_string())?;
    }
    Ok(path)
}

#[cfg(not(windows))]
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module_package::{AgentCommandSpec, AgentSkillSpec};

    #[test]
    #[cfg(unix)]
    fn mounts_and_removes_agent_artifacts() {
        let root = tempfile::tempdir().unwrap();
        let module = root.path().join("module");
        let skill = module.join("skills/manage-taskboard");
        let command = module.join("cli/taskctl.mjs");
        fs::create_dir_all(&skill).unwrap();
        fs::create_dir_all(command.parent().unwrap()).unwrap();
        fs::write(skill.join("SKILL.md"), "---\nname: manage-taskboard\n---").unwrap();
        fs::write(&command, "console.log('ok')").unwrap();
        let spec = AgentIntegrationSpec {
            hosts: vec!["codex".into()],
            skills: vec![AgentSkillSpec {
                name: "manage-taskboard".into(),
                path: "skills/manage-taskboard".into(),
            }],
            commands: vec![AgentCommandSpec {
                name: "taskctl".into(),
                entry: "cli/taskctl.mjs".into(),
                runtime: "node".into(),
            }],
        };

        let skills_root = root.path().join("codex-skills");
        let runtime_bin = root.path().join("bin");
        let command_bin = root.path().join("user-bin");
        let (artifacts, status) = activate(
            &module,
            &spec,
            &skills_root,
            &runtime_bin,
            Some(&command_bin),
            Path::new("/usr/bin/node"),
            Some("http://127.0.0.1:47823"),
        );

        assert_eq!(status.skill_status, "active");
        assert_eq!(status.command_status, "active");
        assert!(skills_root.join("manage-taskboard").is_symlink());
        assert!(runtime_bin.join("taskctl").is_file());
        assert!(command_bin.join("taskctl").is_symlink());
        deactivate(artifacts);
        assert!(!skills_root.join("manage-taskboard").exists());
        assert!(!runtime_bin.join("taskctl").exists());
        assert!(!command_bin.join("taskctl").exists());
    }

    #[test]
    #[cfg(unix)]
    fn command_link_never_overwrites_an_existing_command() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("runtime/taskctl");
        let destination = root.path().join("user-bin/taskctl");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::write(&source, "injector").unwrap();
        fs::write(&destination, "user").unwrap();

        assert!(mount_command(&source, &destination).is_err());
        assert_eq!(fs::read_to_string(destination).unwrap(), "user");
    }
}
