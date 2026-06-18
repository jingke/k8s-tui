/// 日志查看器组件
#[derive(Clone, Debug)]
pub struct LogViewer {
    pub pod_name: String,
    pub namespace: String,
    pub container: Option<String>,
    pub logs: String,
}

impl LogViewer {
    pub fn new() -> Self {
        Self {
            pod_name: String::new(),
            namespace: String::new(),
            container: None,
            logs: String::new(),
        }
    }

    pub fn set_pod_name(&mut self, name: String) {
        self.pod_name = name;
    }

    pub fn set_namespace(&mut self, ns: String) {
        self.namespace = ns;
    }

    pub fn set_container(&mut self, container: Option<String>) {
        self.container = container;
    }

    pub fn set_logs(&mut self, logs: String) {
        self.logs = logs;
    }

    pub fn lines(&self) -> Vec<&str> {
        self.logs.lines().collect()
    }
}
