use anyhow::{Context as AnyhowContext, Result};
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{ConfigMap, Namespace, Node, Pod, Secret, Service};
use kube::{
    api::{Api, DeleteParams, ListParams, LogParams},
    config::{Config, KubeConfigOptions, Kubeconfig},
    Client,
};
use serde::Serialize;

/// K8s 资源通用表示
#[derive(Clone, Debug, PartialEq)]
pub struct K8sResource {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub age: String,
    pub extra: Vec<String>, // 额外列，根据资源类型不同
}

/// K8s 客户端封装
pub struct K8sClient {
    client: Client,
    pub current_context: String,
    pub contexts: Vec<String>,
    #[allow(dead_code)]
    pub kubeconfig: Kubeconfig,
}

impl K8sClient {
    /// 创建新的 K8s 客户端（使用当前 context）
    pub async fn new() -> Result<Self> {
        let kubeconfig = Kubeconfig::read().context("读取 kubeconfig 失败")?;
        let config = Config::from_kubeconfig(&KubeConfigOptions::default())
            .await
            .context("加载 K8s 配置失败")?;

        let client = Client::try_from(config).context("创建 K8s 客户端失败")?;

        let current_context = kubeconfig
            .current_context
            .clone()
            .unwrap_or_else(|| "unknown".to_string());

        let contexts = kubeconfig
            .contexts
            .iter()
            .map(|c| c.name.clone())
            .collect();

        Ok(Self {
            client,
            current_context,
            contexts,
            kubeconfig,
        })
    }

    /// 切换 Context
    pub async fn switch_context(&mut self, context: &str) -> Result<()> {
        let options = KubeConfigOptions {
            context: Some(context.to_string()),
            ..Default::default()
        };
        let config = Config::from_kubeconfig(&options)
            .await
            .with_context(|| format!("切换到 context '{}' 失败", context))?;

        self.client = Client::try_from(config)?;
        self.current_context = context.to_string();
        Ok(())
    }

    /// 列出命名空间
    pub async fn list_namespaces(&self) -> Result<Vec<String>> {
        let api: Api<Namespace> = Api::all(self.client.clone());
        let list = api.list(&ListParams::default()).await?;
        let mut namespaces: Vec<String> = list
            .items
            .into_iter()
            .filter_map(|ns| ns.metadata.name)
            .collect();
        namespaces.sort();
        Ok(namespaces)
    }

    /// 列出 Pod
    pub async fn list_pods(&self, namespace: Option<String>) -> Result<Vec<K8sResource>> {
        let api = namespace.as_ref().map_or_else(
            || Api::<Pod>::all(self.client.clone()),
            |ns| Api::<Pod>::namespaced(self.client.clone(), ns),
        );

        let list = api.list(&ListParams::default()).await?;
        let resources = list
            .items
            .into_iter()
            .map(|pod| {
                let name = pod.metadata.name.unwrap_or_default();
                let namespace = pod.metadata.namespace.unwrap_or_default();
                let status = pod
                    .status
                    .as_ref()
                    .and_then(|s| s.phase.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                let restarts = pod
                    .status
                    .as_ref()
                    .and_then(|s| s.container_statuses.as_ref())
                    .map(|cs| cs.iter().map(|c| c.restart_count).sum::<i32>().to_string())
                    .unwrap_or_else(|| "0".to_string());
                let age = pod
                    .metadata
                    .creation_timestamp
                    .as_ref()
                    .map(format_age)
                    .unwrap_or_else(|| "?".to_string());
                let node = pod
                    .spec
                    .as_ref()
                    .and_then(|s| s.node_name.clone())
                    .unwrap_or_default();

                K8sResource {
                    name,
                    namespace,
                    status,
                    age,
                    extra: vec![restarts, node],
                }
            })
            .collect();
        Ok(resources)
    }

    /// 列出 ConfigMap
    pub async fn list_configmaps(&self, namespace: Option<String>) -> Result<Vec<K8sResource>> {
        let api = namespace.as_ref().map_or_else(
            || Api::<ConfigMap>::all(self.client.clone()),
            |ns| Api::<ConfigMap>::namespaced(self.client.clone(), ns),
        );

        let list = api.list(&ListParams::default()).await?;
        let resources = list
            .items
            .into_iter()
            .map(|cm| {
                let name = cm.metadata.name.unwrap_or_default();
                let namespace = cm.metadata.namespace.unwrap_or_default();
                let data_count = cm
                    .data
                    .as_ref()
                    .map(|d| d.len().to_string())
                    .unwrap_or_else(|| "0".to_string());
                let age = cm
                    .metadata
                    .creation_timestamp
                    .as_ref()
                    .map(format_age)
                    .unwrap_or_else(|| "?".to_string());

                K8sResource {
                    name,
                    namespace,
                    status: "-".to_string(),
                    age,
                    extra: vec![data_count],
                }
            })
            .collect();
        Ok(resources)
    }

    /// 列出 Secret
    pub async fn list_secrets(&self, namespace: Option<String>) -> Result<Vec<K8sResource>> {
        let api = namespace.as_ref().map_or_else(
            || Api::<Secret>::all(self.client.clone()),
            |ns| Api::<Secret>::namespaced(self.client.clone(), ns),
        );

        let list = api.list(&ListParams::default()).await?;
        let resources = list
            .items
            .into_iter()
            .map(|secret| {
                let name = secret.metadata.name.unwrap_or_default();
                let namespace = secret.metadata.namespace.unwrap_or_default();
                let secret_type = secret.type_.unwrap_or_else(|| "Opaque".to_string());
                let data_count = secret
                    .data
                    .as_ref()
                    .map(|d| d.len().to_string())
                    .unwrap_or_else(|| "0".to_string());
                let age = secret
                    .metadata
                    .creation_timestamp
                    .as_ref()
                    .map(format_age)
                    .unwrap_or_else(|| "?".to_string());

                K8sResource {
                    name,
                    namespace,
                    status: secret_type,
                    age,
                    extra: vec![data_count],
                }
            })
            .collect();
        Ok(resources)
    }

    /// 获取 Pod YAML
    pub async fn get_pod_yaml(&self, name: &str, namespace: &str) -> Result<String> {
        let api: Api<Pod> = Api::namespaced(self.client.clone(), namespace);
        let pod = api.get(name).await?;
        to_yaml(&pod)
    }

    /// 获取 ConfigMap YAML
    pub async fn get_configmap_yaml(&self, name: &str, namespace: &str) -> Result<String> {
        let api: Api<ConfigMap> = Api::namespaced(self.client.clone(), namespace);
        let cm = api.get(name).await?;
        to_yaml(&cm)
    }

    /// 获取 Secret YAML（不暴露 data 内容）
    pub async fn get_secret_yaml(&self, name: &str, namespace: &str) -> Result<String> {
        let api: Api<Secret> = Api::namespaced(self.client.clone(), namespace);
        let secret = api.get(name).await?;
        to_yaml(&secret)
    }

    /// 列出 Pod 内的容器名（包含 Init 容器，Init 容器带 "(init)" 后缀）
    pub async fn get_pod_containers(&self, name: &str, namespace: &str) -> Result<Vec<String>> {
        let api: Api<Pod> = Api::namespaced(self.client.clone(), namespace);
        let pod = api.get(name).await?;
        let spec = pod.spec.context("Pod 缺少 spec")?;
        let mut containers: Vec<String> = Vec::new();
        if let Some(init_containers) = spec.init_containers {
            containers.extend(init_containers.into_iter().map(|c| format!("{} (init)", c.name)));
        }
        containers.extend(spec.containers.into_iter().map(|c| c.name));
        Ok(containers)
    }

    /// 获取 Pod 日志（container 为 None 时使用 Pod 的默认容器）
    pub async fn get_pod_logs(
        &self,
        name: &str,
        namespace: &str,
        container: Option<&str>,
    ) -> Result<String> {
        let api: Api<Pod> = Api::namespaced(self.client.clone(), namespace);
        let params = LogParams {
            container: container.map(str::to_string),
            ..Default::default()
        };
        let logs = api.logs(name, &params).await?;
        Ok(logs)
    }

    /// 列出 Deployment
    pub async fn list_deployments(
        &self,
        namespace: Option<String>,
    ) -> Result<Vec<K8sResource>> {
        let api = namespace.as_ref().map_or_else(
            || Api::<Deployment>::all(self.client.clone()),
            |ns| Api::<Deployment>::namespaced(self.client.clone(), ns),
        );

        let list = api.list(&ListParams::default()).await?;
        let resources = list
            .items
            .into_iter()
            .map(|d| {
                let name = d.metadata.name.unwrap_or_default();
                let namespace = d.metadata.namespace.unwrap_or_default();
                let status = d
                    .status
                    .as_ref()
                    .map(|s| {
                        let ready = s.ready_replicas.unwrap_or(0);
                        let total = s.replicas.unwrap_or(0);
                        format!("{}/{}", ready, total)
                    })
                    .unwrap_or_else(|| "?".to_string());
                let age = d
                    .metadata
                    .creation_timestamp
                    .as_ref()
                    .map(format_age)
                    .unwrap_or_else(|| "?".to_string());
                let strategy = d
                    .spec
                    .as_ref()
                    .and_then(|s| s.strategy.as_ref().and_then(|st| st.type_.as_ref().map(|t| t.to_string())))
                    .unwrap_or_default();

                K8sResource {
                    name,
                    namespace,
                    status,
                    age,
                    extra: vec![strategy],
                }
            })
            .collect();
        Ok(resources)
    }

    /// 列出 Service
    pub async fn list_services(
        &self,
        namespace: Option<String>,
    ) -> Result<Vec<K8sResource>> {
        let api = namespace.as_ref().map_or_else(
            || Api::<Service>::all(self.client.clone()),
            |ns| Api::<Service>::namespaced(self.client.clone(), ns),
        );

        let list = api.list(&ListParams::default()).await?;
        let resources = list
            .items
            .into_iter()
            .map(|svc| {
                let name = svc.metadata.name.unwrap_or_default();
                let namespace = svc.metadata.namespace.unwrap_or_default();
                let cluster_ip = svc
                    .spec
                    .as_ref()
                    .and_then(|s| s.cluster_ip.clone())
                    .unwrap_or_default();
                let svc_type = svc
                    .spec
                    .as_ref()
                    .and_then(|s| s.type_.clone())
                    .unwrap_or_default();
                let age = svc
                    .metadata
                    .creation_timestamp
                    .as_ref()
                    .map(format_age)
                    .unwrap_or_else(|| "?".to_string());

                K8sResource {
                    name,
                    namespace,
                    status: svc_type,
                    age,
                    extra: vec![cluster_ip],
                }
            })
            .collect();
        Ok(resources)
    }

    /// 列出 Node
    pub async fn list_nodes(&self) -> Result<Vec<K8sResource>> {
        let api: Api<Node> = Api::all(self.client.clone());
        let list = api.list(&ListParams::default()).await?;
        let resources = list
            .items
            .into_iter()
            .map(|node| {
                let name = node.metadata.name.unwrap_or_default();
                let status = node
                    .status
                    .as_ref()
                    .and_then(|s| s.conditions.as_ref())
                    .and_then(|conds| {
                        conds
                            .iter()
                            .find(|c| c.type_ == "Ready")
                            .map(|c| c.status.clone())
                    })
                    .unwrap_or_else(|| "Unknown".to_string());
                let version = node
                    .status
                    .as_ref()
                    .and_then(|s| s.node_info.as_ref())
                    .map(|ni| ni.kubelet_version.clone())
                    .unwrap_or_default();
                let age = node
                    .metadata
                    .creation_timestamp
                    .as_ref()
                    .map(format_age)
                    .unwrap_or_else(|| "?".to_string());

                K8sResource {
                    name,
                    namespace: "-".to_string(),
                    status,
                    age,
                    extra: vec![version],
                }
            })
            .collect();
        Ok(resources)
    }

    /// 获取 Deployment YAML
    pub async fn get_deployment_yaml(
        &self,
        name: &str,
        namespace: &str,
    ) -> Result<String> {
        let api: Api<Deployment> = Api::namespaced(self.client.clone(), namespace);
        let d = api.get(name).await?;
        to_yaml(&d)
    }

    /// 获取 Service YAML
    pub async fn get_service_yaml(
        &self,
        name: &str,
        namespace: &str,
    ) -> Result<String> {
        let api: Api<Service> = Api::namespaced(self.client.clone(), namespace);
        let svc = api.get(name).await?;
        to_yaml(&svc)
    }

    /// 获取 Node YAML
    pub async fn get_node_yaml(&self, name: &str) -> Result<String> {
        let api: Api<Node> = Api::all(self.client.clone());
        let node = api.get(name).await?;
        to_yaml(&node)
    }

    /// 删除 Deployment
    pub async fn delete_deployment(
        &self,
        name: &str,
        namespace: &str,
    ) -> Result<()> {
        let api: Api<Deployment> = Api::namespaced(self.client.clone(), namespace);
        api.delete(name, &DeleteParams::default()).await?;
        Ok(())
    }

    /// 删除 Service
    pub async fn delete_service(
        &self,
        name: &str,
        namespace: &str,
    ) -> Result<()> {
        let api: Api<Service> = Api::namespaced(self.client.clone(), namespace);
        api.delete(name, &DeleteParams::default()).await?;
        Ok(())
    }

    /// 删除 Pod
    pub async fn delete_pod(&self, name: &str, namespace: &str) -> Result<()> {
        let api: Api<Pod> = Api::namespaced(self.client.clone(), namespace);
        api.delete(name, &DeleteParams::default()).await?;
        Ok(())
    }

    /// 删除 ConfigMap
    pub async fn delete_configmap(&self, name: &str, namespace: &str) -> Result<()> {
        let api: Api<ConfigMap> = Api::namespaced(self.client.clone(), namespace);
        api.delete(name, &DeleteParams::default()).await?;
        Ok(())
    }

    /// 删除 Secret
    pub async fn delete_secret(&self, name: &str, namespace: &str) -> Result<()> {
        let api: Api<Secret> = Api::namespaced(self.client.clone(), namespace);
        api.delete(name, &DeleteParams::default()).await?;
        Ok(())
    }
}

/// 将对象序列化为 YAML
fn to_yaml<T: Serialize>(obj: &T) -> Result<String> {
    serde_yaml::to_string(obj).context("YAML 序列化失败")
}

/// 格式化年龄（简化版）
fn format_age(ts: &k8s_openapi::apimachinery::pkg::apis::meta::v1::Time) -> String {
    let now = chrono::Utc::now();
    let dt = ts.0;
    let duration = now.signed_duration_since(dt);

    if duration.num_days() > 0 {
        format!("{}d", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{}h", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{}m", duration.num_minutes())
    } else {
        format!("{}s", duration.num_seconds())
    }
}
