/// 帮助弹窗组件
#[derive(Clone, Debug)]
pub struct HelpPopup;

impl HelpPopup {
    pub fn new() -> Self {
        Self
    }

    pub fn content() -> Vec<String> {
        vec![
            "快捷键帮助".to_string(),
            "".to_string(),
            "导航:".to_string(),
            "  ↑/↓ 或 j/k    移动选中行".to_string(),
            "  Tab/1/2/3     切换资源标签".to_string(),
            "".to_string(),
            "操作:".to_string(),
            "  Enter         查看资源详情".to_string(),
            "  d             删除资源（需确认）".to_string(),
            "  l             查看 Pod 日志（多容器时可选择）".to_string(),
            "  r             手动刷新".to_string(),
            "".to_string(),
            "日志查看:".to_string(),
            "  c             切换查看的容器".to_string(),
            "  ↑/↓ PgUp/PgDn 滚动 / 翻页".to_string(),
            "".to_string(),
            "切换:".to_string(),
            "  c             切换 K8s Context".to_string(),
            "  n             切换命名空间".to_string(),
            "".to_string(),
            "搜索:".to_string(),
            "  /             进入搜索模式".to_string(),
            "  Esc           退出搜索".to_string(),
            "".to_string(),
            "其他:".to_string(),
            "  ?             打开帮助".to_string(),
            "  q             退出工具".to_string(),
        ]
    }
}
