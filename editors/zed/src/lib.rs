use zed_extension_api::{self as zed, Command, LanguageServerId, Result};

struct RplExtension;

impl zed::Extension for RplExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Command> {
        let binary_path = worktree
            .which("rpl")
            .unwrap_or_else(|| "C:\\Program Files\\RunningPseudoLanguage\\rpl.exe".to_string());

        Ok(Command {
            command: binary_path,
            args: vec!["lsp".to_string()],
            env: Default::default(),
        })
    }
}

zed::register_extension!(RplExtension);
