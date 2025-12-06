use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;
use lazy_static::lazy_static;
use crate::vga_buffer::WRITER;

/// Erreurs possibles du shell
#[derive(Debug)]
pub enum ShellError {
    CommandNotFound(String),
    InvalidArguments,
    ExecutionFailed(String),
    IOError,
}

/// Représente une commande parsée
#[derive(Debug, Clone)]
pub struct Command {
    pub program: String,
    pub args: Vec<String>,
    pub stdin: Option<String>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub pipes: Vec<Command>,
}

impl Command {
    pub fn new(program: &str) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            stdin: None,
            stdout: None,
            stderr: None,
            pipes: Vec::new(),
        }
    }

    pub fn add_arg(&mut self, arg: &str) {
        self.args.push(arg.into());
    }
}

/// Gestionnaire du shell
pub struct Shell {
    pub current_dir: String,
    pub env_vars: BTreeMap<String, String>,
    pub history: Vec<String>,
    pub history_index: usize,
}

impl Shell {
    /// Crée une nouvelle instance du shell
    pub fn new() -> Self {
        let mut env_vars = BTreeMap::new();
        env_vars.insert("HOME".into(), "/home".into());
        env_vars.insert("PATH".into(), "/bin:/usr/bin".into());
        env_vars.insert("USER".into(), "root".into());
        env_vars.insert("SHELL".into(), "/bin/bash".into());

        Self {
            current_dir: "/".into(),
            env_vars,
            history: Vec::new(),
            history_index: 0,
        }
    }

    /// Affiche le prompt
    pub fn print_prompt(&self) {
        WRITER.lock().write_string(&format!("{}> ", self.current_dir));
    }

    /// Parse une ligne de commande
    pub fn parse_command(&self, input: &str) -> Result<Command, ShellError> {
        let input = input.trim();
        
        if input.is_empty() {
            return Err(ShellError::InvalidArguments);
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return Err(ShellError::InvalidArguments);
        }

        let mut cmd = Command::new(parts[0]);
        
        for part in &parts[1..] {
            cmd.add_arg(part);
        }

        Ok(cmd)
    }

    /// Exécute une commande
    pub fn execute(&mut self, cmd: Command) -> Result<(), ShellError> {
        match cmd.program.as_str() {
            "cd" => self.builtin_cd(&cmd),
            "pwd" => self.builtin_pwd(&cmd),
            "ls" => self.builtin_ls(&cmd),
            "echo" => self.builtin_echo(&cmd),
            "cat" => self.builtin_cat(&cmd),
            "mkdir" => self.builtin_mkdir(&cmd),
            "rm" => self.builtin_rm(&cmd),
            "cp" => self.builtin_cp(&cmd),
            "mv" => self.builtin_mv(&cmd),
            "exit" => self.builtin_exit(&cmd),
            "help" => self.builtin_help(&cmd),
            "export" => self.builtin_export(&cmd),
            "ps" => self.builtin_ps(&cmd),
            "clear" => self.builtin_clear(&cmd),
            "history" => self.builtin_history(&cmd),
            _ => Err(ShellError::CommandNotFound(cmd.program.clone())),
        }
    }

    /// Ajoute une commande à l'historique
    pub fn add_to_history(&mut self, cmd: &str) {
        self.history.push(cmd.into());
        self.history_index = self.history.len();
    }

    // ============ COMMANDES BUILTINS ============

    /// Commande: cd <répertoire>
    fn builtin_cd(&mut self, cmd: &Command) -> Result<(), ShellError> {
        if cmd.args.is_empty() {
            self.current_dir = self.env_vars.get("HOME")
                .cloned()
                .unwrap_or_else(|| "/".into());
            Ok(())
        } else {
            let path = &cmd.args[0];
            if path == ".." {
                if self.current_dir != "/" {
                    if let Some(pos) = self.current_dir.rfind('/') {
                        if pos == 0 {
                            self.current_dir = "/".into();
                        } else {
                            self.current_dir = self.current_dir[..pos].into();
                        }
                    }
                }
            } else if path == "/" {
                self.current_dir = "/".into();
            } else if path.starts_with('/') {
                self.current_dir = path.into();
            } else {
                if self.current_dir == "/" {
                    self.current_dir = format!("/{}", path);
                } else {
                    self.current_dir = format!("{}/{}", self.current_dir, path);
                }
            }
            Ok(())
        }
    }

    /// Commande: pwd
    fn builtin_pwd(&self, _cmd: &Command) -> Result<(), ShellError> {
        WRITER.lock().write_string(&format!("{}\n", self.current_dir));
        Ok(())
    }

    /// Commande: ls [répertoire]
    fn builtin_ls(&self, cmd: &Command) -> Result<(), ShellError> {
        let dir = if cmd.args.is_empty() {
            self.current_dir.clone()
        } else {
            cmd.args[0].clone()
        };

        WRITER.lock().write_string(&format!("Contenu de {}:\n", dir));
        WRITER.lock().write_string("  .\n");
        WRITER.lock().write_string("  ..\n");
        WRITER.lock().write_string("  file1.txt\n");
        WRITER.lock().write_string("  file2.txt\n");
        
        Ok(())
    }

    /// Commande: echo <texte>
    fn builtin_echo(&self, cmd: &Command) -> Result<(), ShellError> {
        let text = cmd.args.join(" ");
        WRITER.lock().write_string(&format!("{}\n", text));
        Ok(())
    }

    /// Commande: cat <fichier>
    fn builtin_cat(&self, cmd: &Command) -> Result<(), ShellError> {
        if cmd.args.is_empty() {
            return Err(ShellError::InvalidArguments);
        }

        let filename = &cmd.args[0];
        WRITER.lock().write_string(&format!("Contenu de {}:\n", filename));
        WRITER.lock().write_string("(Contenu du fichier)\n");
        
        Ok(())
    }

    /// Commande: mkdir <répertoire>
    fn builtin_mkdir(&self, cmd: &Command) -> Result<(), ShellError> {
        if cmd.args.is_empty() {
            return Err(ShellError::InvalidArguments);
        }

        let dirname = &cmd.args[0];
        WRITER.lock().write_string(&format!("Création du répertoire: {}\n", dirname));
        
        Ok(())
    }

    /// Commande: rm <fichier>
    fn builtin_rm(&self, cmd: &Command) -> Result<(), ShellError> {
        if cmd.args.is_empty() {
            return Err(ShellError::InvalidArguments);
        }

        let filename = &cmd.args[0];
        WRITER.lock().write_string(&format!("Suppression de: {}\n", filename));
        
        Ok(())
    }

    /// Commande: cp <source> <destination>
    fn builtin_cp(&self, cmd: &Command) -> Result<(), ShellError> {
        if cmd.args.len() < 2 {
            return Err(ShellError::InvalidArguments);
        }

        let src = &cmd.args[0];
        let dst = &cmd.args[1];
        WRITER.lock().write_string(&format!("Copie de {} vers {}\n", src, dst));
        
        Ok(())
    }

    /// Commande: mv <source> <destination>
    fn builtin_mv(&self, cmd: &Command) -> Result<(), ShellError> {
        if cmd.args.len() < 2 {
            return Err(ShellError::InvalidArguments);
        }

        let src = &cmd.args[0];
        let dst = &cmd.args[1];
        WRITER.lock().write_string(&format!("Déplacement de {} vers {}\n", src, dst));
        
        Ok(())
    }

    /// Commande: exit
    fn builtin_exit(&self, _cmd: &Command) -> Result<(), ShellError> {
        WRITER.lock().write_string("Au revoir!\n");
        // TODO: Terminer le shell
        Ok(())
    }

    /// Commande: help
    fn builtin_help(&self, _cmd: &Command) -> Result<(), ShellError> {
        WRITER.lock().write_string("Commandes disponibles:\n");
        WRITER.lock().write_string("  cd <dir>      - Changer de répertoire\n");
        WRITER.lock().write_string("  pwd           - Afficher le répertoire courant\n");
        WRITER.lock().write_string("  ls [dir]      - Lister les fichiers\n");
        WRITER.lock().write_string("  echo <text>   - Afficher du texte\n");
        WRITER.lock().write_string("  cat <file>    - Afficher le contenu d'un fichier\n");
        WRITER.lock().write_string("  mkdir <dir>   - Créer un répertoire\n");
        WRITER.lock().write_string("  rm <file>     - Supprimer un fichier\n");
        WRITER.lock().write_string("  cp <s> <d>    - Copier un fichier\n");
        WRITER.lock().write_string("  mv <s> <d>    - Déplacer un fichier\n");
        WRITER.lock().write_string("  exit          - Quitter le shell\n");
        WRITER.lock().write_string("  help          - Afficher cette aide\n");
        WRITER.lock().write_string("  export <var>  - Définir une variable\n");
        WRITER.lock().write_string("  ps            - Lister les processus\n");
        WRITER.lock().write_string("  clear         - Effacer l'écran\n");
        WRITER.lock().write_string("  history       - Afficher l'historique\n");
        
        Ok(())
    }

    /// Commande: export <variable>=<valeur>
    fn builtin_export(&mut self, cmd: &Command) -> Result<(), ShellError> {
        if cmd.args.is_empty() {
            return Err(ShellError::InvalidArguments);
        }

        let arg = &cmd.args[0];
        if let Some(pos) = arg.find('=') {
            let key = &arg[..pos];
            let value = &arg[pos+1..];
            self.env_vars.insert(key.into(), value.into());
            WRITER.lock().write_string(&format!("{}={}\n", key, value));
        } else {
            return Err(ShellError::InvalidArguments);
        }
        
        Ok(())
    }

    /// Commande: ps
    fn builtin_ps(&self, _cmd: &Command) -> Result<(), ShellError> {
        WRITER.lock().write_string("PID  COMMAND\n");
        WRITER.lock().write_string("1    init\n");
        WRITER.lock().write_string("2    shell\n");
        
        Ok(())
    }

    /// Commande: clear
    fn builtin_clear(&self, _cmd: &Command) -> Result<(), ShellError> {
        // TODO: Implémenter l'effacement de l'écran
        WRITER.lock().write_string("\x1b[2J\x1b[H");
        Ok(())
    }

    /// Commande: history
    fn builtin_history(&self, _cmd: &Command) -> Result<(), ShellError> {
        for (i, cmd) in self.history.iter().enumerate() {
            WRITER.lock().write_string(&format!("  {}  {}\n", i + 1, cmd));
        }
        
        Ok(())
    }
}

lazy_static! {
    pub static ref SHELL: Mutex<Shell> = Mutex::new(Shell::new());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_shell_creation() {
        let shell = Shell::new();
        assert_eq!(shell.current_dir, "/");
        assert!(!shell.env_vars.is_empty());
    }

    #[test_case]
    fn test_parse_command() {
        let shell = Shell::new();
        let cmd = shell.parse_command("ls -la /home").unwrap();
        assert_eq!(cmd.program, "ls");
        assert_eq!(cmd.args.len(), 2);
    }

    #[test_case]
    fn test_builtin_cd() {
        let mut shell = Shell::new();
        let cmd = Command {
            program: "cd".into(),
            args: vec!["/home".into()],
            stdin: None,
            stdout: None,
            stderr: None,
            pipes: Vec::new(),
        };
        assert!(shell.execute(cmd).is_ok());
        assert_eq!(shell.current_dir, "/home");
    }
}
