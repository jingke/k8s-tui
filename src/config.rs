//! 配置系统：持久化用户偏好

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 用户配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// 默认 context
    pub context: Option<String>,
    /// 默认 namespace
    pub namespace: Option<String>,
    /// 默认资源标签页
    pub resource_tab: Option<String>,
    /// 自动刷新间隔（秒）
    pub refresh_interval: u64,
    /// 日志行数限制
    pub log_lines: i64,
    /// 主题：dark / light
    pub theme: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            context: None,
            namespace: None,
            resource_tab: None,
            refresh_interval: 5,
            log_lines: 500,
            theme: "dark".to_string(),
        }
    }
}

impl Config {
    /// 加载配置文件
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("读取配置文件失败: {}", path.display()))?;
        let config: Config = toml::from_str(&content)
            .with_context(|| format!("解析配置文件失败: {}", path.display()))?;
        Ok(config)
    }

    /// 保存配置文件
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("创建配置目录失败: {}", parent.display()))?;
        }
        let content = toml::to_string_pretty(self).context("序列化配置失败")?;
        std::fs::write(&path, content)
            .with_context(|| format!("写入配置文件失败: {}", path.display()))?;
        Ok(())
    }

    /// 配置文件路径
    pub fn config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("无法获取 home 目录")?;
        Ok(home.join(".config").join("k8s-tui").join("config.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.refresh_interval, 5);
        assert_eq!(config.log_lines, 500);
        assert_eq!(config.theme, "dark");
        assert!(config.context.is_none());
        assert!(config.namespace.is_none());
    }

    #[test]
    fn test_config_roundtrip() {
        let config = Config {
            context: Some("minikube".to_string()),
            namespace: Some("kube-system".to_string()),
            resource_tab: Some("Pod".to_string()),
            refresh_interval: 10,
            log_lines: 1000,
            theme: "light".to_string(),
        };

        let serialized = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();

        assert_eq!(deserialized.context, Some("minikube".to_string()));
        assert_eq!(deserialized.namespace, Some("kube-system".to_string()));
        assert_eq!(deserialized.resource_tab, Some("Pod".to_string()));
        assert_eq!(deserialized.refresh_interval, 10);
        assert_eq!(deserialized.log_lines, 1000);
        assert_eq!(deserialized.theme, "light");
    }
}
