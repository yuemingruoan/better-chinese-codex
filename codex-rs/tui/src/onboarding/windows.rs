use std::path::PathBuf;

use codex_core::config::edit::ConfigEditsBuilder;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::Color;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::widgets::WidgetRef;
use ratatui::widgets::Wrap;

use crate::onboarding::onboarding_screen::KeyboardHandler;
use crate::onboarding::onboarding_screen::StepStateProvider;

use super::onboarding_screen::StepState;

pub(crate) const WSL_INSTRUCTIONS: &str = r#"以管理员身份打开 PowerShell 并运行以下命令来安装 WSL2：
    # 使用默认的 Linux 发行版（Ubuntu）安装 WSL。
    # 了解更多信息：https://learn.microsoft.com/en-us/windows/wsl/install
    wsl --install

    # 重启电脑，然后在 Windows Subsystem for Linux 中启动一个 shell
    wsl

    # 在 WSL 中通过 nvm 安装 Node.js
    # 文档：https://learn.microsoft.com/en-us/windows/dev-environment/javascript/nodejs-on-wsl
    curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/master/install.sh | bash && export NVM_DIR="$HOME/.nvm" && \. "$NVM_DIR/nvm.sh"
    nvm install 22

    # 在 WSL 中安装并运行 Codex
    npm install --global @openai/codex
    codex

    # 关于如何在 WSL 中安装并运行 Codex 的更多细节与指南：
    https://developers.openai.com/codex/windows"#;

pub(crate) struct WindowsSetupWidget {
    pub codex_home: PathBuf,
    pub selection: Option<WindowsSetupSelection>,
    pub highlighted: WindowsSetupSelection,
    pub error: Option<String>,
    exit_requested: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowsSetupSelection {
    Continue,
    Install,
}

impl WindowsSetupWidget {
    pub fn new(codex_home: PathBuf) -> Self {
        Self {
            codex_home,
            selection: None,
            highlighted: WindowsSetupSelection::Install,
            error: None,
            exit_requested: false,
        }
    }

    fn handle_continue(&mut self) {
        self.highlighted = WindowsSetupSelection::Continue;
        match ConfigEditsBuilder::new(&self.codex_home)
            .set_windows_wsl_setup_acknowledged(true)
            .apply_blocking()
        {
            Ok(()) => {
                self.selection = Some(WindowsSetupSelection::Continue);
                self.exit_requested = false;
                self.error = None;
            }
            Err(err) => {
                tracing::error!("Failed to persist Windows onboarding acknowledgement: {err:?}");
                self.error = Some(format!("更新配置失败：{err}"));
                self.selection = None;
            }
        }
    }

    fn handle_install(&mut self) {
        self.highlighted = WindowsSetupSelection::Install;
        self.selection = Some(WindowsSetupSelection::Install);
        self.exit_requested = true;
    }

    pub fn exit_requested(&self) -> bool {
        self.exit_requested
    }
}

impl WidgetRef for &WindowsSetupWidget {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let mut lines: Vec<Line> = vec![
            Line::from(vec![
                "> ".into(),
                "若要使用 Codex 的全部功能，建议在 Windows Subsystem for Linux (WSL2) 中运行 Codex"
                    .bold(),
            ]),
            Line::from(vec![
                "  ".into(),
                "WSL 让 Codex 能够在沙箱环境中运行 Agent 模式，并提供更完善的数据保护。".into(),
            ]),
            Line::from(vec![
                "  ".into(),
                "了解更多：https://developers.openai.com/codex/windows".into(),
            ]),
            Line::from(""),
        ];

        let create_option =
            |idx: usize, option: WindowsSetupSelection, text: &str| -> Line<'static> {
                if self.highlighted == option {
                    Line::from(format!("> {}. {text}", idx + 1)).cyan()
                } else {
                    Line::from(format!("  {}. {}", idx + 1, text))
                }
            };

        lines.push(create_option(
            0,
            WindowsSetupSelection::Install,
            "退出并安装 WSL2",
        ));
        lines.push(create_option(
            1,
            WindowsSetupSelection::Continue,
            "继续使用当前环境",
        ));
        lines.push("".into());

        if let Some(error) = &self.error {
            lines.push(Line::from(format!("  {error}")).fg(Color::Red));
            lines.push("".into());
        }

        lines.push(Line::from(vec!["  按 Enter 继续".dim()]));

        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }
}

impl KeyboardHandler for WindowsSetupWidget {
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if key_event.kind == KeyEventKind::Release {
            return;
        }

        match key_event.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.highlighted = WindowsSetupSelection::Install;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.highlighted = WindowsSetupSelection::Continue;
            }
            KeyCode::Char('1') => self.handle_install(),
            KeyCode::Char('2') => self.handle_continue(),
            KeyCode::Enter => match self.highlighted {
                WindowsSetupSelection::Install => self.handle_install(),
                WindowsSetupSelection::Continue => self.handle_continue(),
            },
            _ => {}
        }
    }
}

impl StepStateProvider for WindowsSetupWidget {
    fn get_step_state(&self) -> StepState {
        match self.selection {
            Some(WindowsSetupSelection::Continue) => StepState::Hidden,
            Some(WindowsSetupSelection::Install) => StepState::Complete,
            None => StepState::InProgress,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn windows_step_hidden_after_continue() {
        let temp_dir = TempDir::new().expect("temp dir");
        let mut widget = WindowsSetupWidget::new(temp_dir.path().to_path_buf());

        assert_eq!(widget.get_step_state(), StepState::InProgress);

        widget.handle_continue();

        assert_eq!(widget.get_step_state(), StepState::Hidden);
        assert!(!widget.exit_requested());
    }

    #[test]
    fn windows_step_complete_after_install_selection() {
        let temp_dir = TempDir::new().expect("temp dir");
        let mut widget = WindowsSetupWidget::new(temp_dir.path().to_path_buf());

        widget.handle_install();

        assert_eq!(widget.get_step_state(), StepState::Complete);
        assert!(widget.exit_requested());
    }
}
