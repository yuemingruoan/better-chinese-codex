/// 更新动作：始终引导用户打开 Releases 页面。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateAction {
    OpenReleasePage,
}

impl UpdateAction {
    pub const RELEASE_PAGE_URL: &'static str =
        "https://github.com/yuemingruoan/better-chinese-codex/releases";

    pub fn release_url(self) -> &'static str {
        Self::RELEASE_PAGE_URL
    }
}

#[cfg(not(debug_assertions))]
pub(crate) fn get_update_action() -> Option<UpdateAction> {
    Some(UpdateAction::OpenReleasePage)
}
