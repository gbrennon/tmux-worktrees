#![allow(dead_code)]
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use tmux_worktrees::core::error::Result;
use tmux_worktrees::core::ports::TmuxPort;

pub struct StubTmx;
impl TmuxPort for StubTmx {
    fn show_error(&self, _msg: &str) -> Result<()> {
        unimplemented!()
    }
    fn display_popup(&self, _w: &str, _h: &str, _d: &str, _c: &str) -> Result<()> {
        unimplemented!()
    }
    fn resolve_workspace_dir(&self) -> String {
        unimplemented!()
    }
    fn resolve_shell_command(&self) -> String {
        unimplemented!()
    }
    fn select_or_create_window(&self, _n: &str, _p: &str, _c: &str) -> Result<()> {
        unimplemented!()
    }
    fn get_option(&self, _k: &str) -> Option<String> {
        unimplemented!()
    }
    fn show_environment(&self, _v: &str) -> Result<Option<String>> {
        Ok(None)
    }
    fn current_pane_path(&self) -> Result<String> {
        Ok(".".to_string())
    }
    fn run(&self, _a: &[&str]) -> Result<(i32, String, String)> {
        unimplemented!()
    }
    fn kill_window(&self, _n: &str) -> Result<()> {
        unimplemented!()
    }
}

pub struct FakeTmxShowError;
impl TmuxPort for FakeTmxShowError {
    fn show_error(&self, _msg: &str) -> Result<()> {
        Ok(())
    }
    fn display_popup(&self, _w: &str, _h: &str, _d: &str, _c: &str) -> Result<()> {
        unimplemented!()
    }
    fn resolve_workspace_dir(&self) -> String {
        unimplemented!()
    }
    fn resolve_shell_command(&self) -> String {
        unimplemented!()
    }
    fn select_or_create_window(&self, _n: &str, _p: &str, _c: &str) -> Result<()> {
        unimplemented!()
    }
    fn get_option(&self, _k: &str) -> Option<String> {
        unimplemented!()
    }
    fn show_environment(&self, _v: &str) -> Result<Option<String>> {
        Ok(None)
    }
    fn current_pane_path(&self) -> Result<String> {
        Ok(".".to_string())
    }
    fn run(&self, _a: &[&str]) -> Result<(i32, String, String)> {
        unimplemented!()
    }
    fn kill_window(&self, _n: &str) -> Result<()> {
        unimplemented!()
    }
}

#[derive(Clone)]
pub struct FakeTmxNonTty {
    pub show_error_called: std::rc::Rc<std::cell::Cell<bool>>,
}
impl FakeTmxNonTty {
    pub fn new() -> Self {
        Self {
            show_error_called: std::rc::Rc::new(std::cell::Cell::new(false)),
        }
    }
}
impl TmuxPort for FakeTmxNonTty {
    fn display_popup(&self, _w: &str, _h: &str, _d: &str, _c: &str) -> Result<()> {
        Ok(())
    }
    fn show_error(&self, _msg: &str) -> Result<()> {
        self.show_error_called.set(true);
        Ok(())
    }
    fn resolve_workspace_dir(&self) -> String {
        ".".to_string()
    }
    fn resolve_shell_command(&self) -> String {
        "/bin/sh".to_string()
    }
    fn select_or_create_window(&self, _n: &str, _p: &str, _c: &str) -> Result<()> {
        unimplemented!()
    }
    fn get_option(&self, _k: &str) -> Option<String> {
        unimplemented!()
    }
    fn show_environment(&self, _v: &str) -> Result<Option<String>> {
        Ok(None)
    }
    fn current_pane_path(&self) -> Result<String> {
        Ok(".".to_string())
    }
    fn run(&self, _a: &[&str]) -> Result<(i32, String, String)> {
        unimplemented!()
    }
    fn kill_window(&self, _n: &str) -> Result<()> {
        unimplemented!()
    }
}

pub struct FakeTmxCreateSuccess;
impl TmuxPort for FakeTmxCreateSuccess {
    fn show_error(&self, _msg: &str) -> Result<()> {
        Ok(())
    }
    fn resolve_workspace_dir(&self) -> String {
        ".worktrees".into()
    }
    fn resolve_shell_command(&self) -> String {
        "/bin/bash".into()
    }
    fn select_or_create_window(&self, _n: &str, _p: &str, _c: &str) -> Result<()> {
        Ok(())
    }
    fn show_environment(&self, _v: &str) -> Result<Option<String>> {
        Ok(None)
    }
    fn current_pane_path(&self) -> Result<String> {
        Err(tmux_worktrees::core::error::Error::new("no pane"))
    }
    fn display_popup(&self, _w: &str, _h: &str, _d: &str, _c: &str) -> Result<()> {
        unimplemented!()
    }
    fn get_option(&self, _k: &str) -> Option<String> {
        unimplemented!()
    }
    fn run(&self, _a: &[&str]) -> Result<(i32, String, String)> {
        unimplemented!()
    }
    fn kill_window(&self, _n: &str) -> Result<()> {
        unimplemented!()
    }
}

pub struct FakeTmxWithWindow;
impl TmuxPort for FakeTmxWithWindow {
    fn run(&self, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["list-windows", "-F", "{window_name}"] => Ok((0, "ws-feat/x\n".into(), String::new())),
            ["kill-window", "-t", _] => Ok((0, String::new(), String::new())),
            _ => unimplemented!(),
        }
    }
    fn kill_window(&self, branch: &str) -> Result<()> {
        let win = format!("ws-{branch}");
        let (status, out, _) = self.run(&["list-windows", "-F", "{window_name}"])?;
        if status == 0 && out.lines().any(|l| l.trim() == win) {
            self.run(&["kill-window", "-t", &win])?;
        }
        Ok(())
    }
    fn show_error(&self, _msg: &str) -> Result<()> {
        unimplemented!()
    }
    fn display_popup(&self, _w: &str, _h: &str, _d: &str, _c: &str) -> Result<()> {
        unimplemented!()
    }
    fn resolve_workspace_dir(&self) -> String {
        unimplemented!()
    }
    fn resolve_shell_command(&self) -> String {
        unimplemented!()
    }
    fn select_or_create_window(&self, _n: &str, _p: &str, _c: &str) -> Result<()> {
        unimplemented!()
    }
    fn get_option(&self, _k: &str) -> Option<String> {
        unimplemented!()
    }
    fn show_environment(&self, _v: &str) -> Result<Option<String>> {
        Ok(None)
    }
    fn current_pane_path(&self) -> Result<String> {
        Ok(".".to_string())
    }
}

pub struct FakeTmxNoWindow;
impl TmuxPort for FakeTmxNoWindow {
    fn run(&self, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["list-windows", "-F", "{window_name}"] => {
                Ok((0, "other-window\n".into(), String::new()))
            }
            ["kill-window", "-t", _] => Ok((0, String::new(), String::new())),
            _ => unimplemented!(),
        }
    }
    fn kill_window(&self, branch: &str) -> Result<()> {
        let win = format!("ws-{branch}");
        let (status, out, _) = self.run(&["list-windows", "-F", "{window_name}"])?;
        if status == 0 && out.lines().any(|l| l.trim() == win) {
            self.run(&["kill-window", "-t", &win])?;
        }
        Ok(())
    }
    fn show_error(&self, _msg: &str) -> Result<()> {
        unimplemented!()
    }
    fn display_popup(&self, _w: &str, _h: &str, _d: &str, _c: &str) -> Result<()> {
        unimplemented!()
    }
    fn resolve_workspace_dir(&self) -> String {
        unimplemented!()
    }
    fn resolve_shell_command(&self) -> String {
        unimplemented!()
    }
    fn select_or_create_window(&self, _n: &str, _p: &str, _c: &str) -> Result<()> {
        unimplemented!()
    }
    fn get_option(&self, _k: &str) -> Option<String> {
        unimplemented!()
    }
    fn show_environment(&self, _v: &str) -> Result<Option<String>> {
        Ok(None)
    }
    fn current_pane_path(&self) -> Result<String> {
        Ok(".".to_string())
    }
}

#[derive(Clone)]
pub struct FakeTmxPopupOk {
    pub display_called: std::rc::Rc<std::cell::Cell<bool>>,
}
impl FakeTmxPopupOk {
    pub fn new() -> Self {
        Self {
            display_called: std::rc::Rc::new(std::cell::Cell::new(false)),
        }
    }
}
impl TmuxPort for FakeTmxPopupOk {
    fn display_popup(&self, _w: &str, _h: &str, _d: &str, _c: &str) -> Result<()> {
        self.display_called.set(true);
        Ok(())
    }
    fn show_environment(&self, _v: &str) -> Result<Option<String>> {
        Ok(None)
    }
    fn current_pane_path(&self) -> Result<String> {
        Err(tmux_worktrees::core::error::Error::new("no pane"))
    }
    fn show_error(&self, _msg: &str) -> Result<()> {
        unimplemented!()
    }
    fn resolve_workspace_dir(&self) -> String {
        unimplemented!()
    }
    fn resolve_shell_command(&self) -> String {
        unimplemented!()
    }
    fn select_or_create_window(&self, _n: &str, _p: &str, _c: &str) -> Result<()> {
        unimplemented!()
    }
    fn get_option(&self, _k: &str) -> Option<String> {
        unimplemented!()
    }
    fn run(&self, _a: &[&str]) -> Result<(i32, String, String)> {
        unimplemented!()
    }
    fn kill_window(&self, _n: &str) -> Result<()> {
        unimplemented!()
    }
}

#[derive(Clone)]
pub struct FakeTmxWindowFail {
    pub show_error_called: std::rc::Rc<std::cell::Cell<bool>>,
}
impl FakeTmxWindowFail {
    pub fn new() -> Self {
        Self {
            show_error_called: std::rc::Rc::new(std::cell::Cell::new(false)),
        }
    }
}
impl TmuxPort for FakeTmxWindowFail {
    fn show_error(&self, _msg: &str) -> Result<()> {
        self.show_error_called.set(true);
        Ok(())
    }
    fn resolve_workspace_dir(&self) -> String {
        ".worktrees".into()
    }
    fn resolve_shell_command(&self) -> String {
        "/bin/bash".into()
    }
    fn select_or_create_window(&self, _n: &str, _p: &str, _c: &str) -> Result<()> {
        Err(tmux_worktrees::core::error::Error::new(
            "window creation failed",
        ))
    }
    fn show_environment(&self, _v: &str) -> Result<Option<String>> {
        Ok(None)
    }
    fn current_pane_path(&self) -> Result<String> {
        Err(tmux_worktrees::core::error::Error::new("no pane"))
    }
    fn display_popup(&self, _w: &str, _h: &str, _d: &str, _c: &str) -> Result<()> {
        unimplemented!()
    }
    fn get_option(&self, _k: &str) -> Option<String> {
        unimplemented!()
    }
    fn run(&self, _a: &[&str]) -> Result<(i32, String, String)> {
        unimplemented!()
    }
    fn kill_window(&self, _n: &str) -> Result<()> {
        unimplemented!()
    }
}

pub struct FakeTmxTest {
    pub kill_called: Rc<Cell<bool>>,
    pub show_error_called: Rc<Cell<bool>>,
    pub show_error_msg: Rc<RefCell<String>>,
    pub display_msgs: Rc<RefCell<Vec<String>>>,
    pub pane_path: Option<String>,
    pub env_value: Option<String>,
    pub worktree_dir: String,
    pub shell_cmd: String,
    pub auto_fetch: Option<String>,
}
impl FakeTmxTest {
    pub fn new(repo: &str) -> Self {
        Self {
            kill_called: Rc::new(Cell::new(false)),
            show_error_called: Rc::new(Cell::new(false)),
            show_error_msg: Rc::new(RefCell::new(String::new())),
            display_msgs: Rc::new(RefCell::new(Vec::new())),
            pane_path: None,
            env_value: Some(repo.to_string()),
            worktree_dir: ".worktrees".into(),
            shell_cmd: "/bin/bash".into(),
            auto_fetch: Some("false".into()),
        }
    }
}
impl TmuxPort for FakeTmxTest {
    fn show_error(&self, msg: &str) -> Result<()> {
        self.show_error_called.set(true);
        *self.show_error_msg.borrow_mut() = msg.to_string();
        Ok(())
    }
    fn resolve_workspace_dir(&self) -> String {
        self.worktree_dir.clone()
    }
    fn resolve_shell_command(&self) -> String {
        self.shell_cmd.clone()
    }
    fn select_or_create_window(&self, _n: &str, _p: &str, _c: &str) -> Result<()> {
        Ok(())
    }
    fn show_environment(&self, _v: &str) -> Result<Option<String>> {
        Ok(self.env_value.clone())
    }
    fn current_pane_path(&self) -> Result<String> {
        match &self.pane_path {
            Some(p) => Ok(p.clone()),
            None => Err(tmux_worktrees::core::error::Error::new("no pane")),
        }
    }
    fn display_popup(&self, _w: &str, _h: &str, _d: &str, _c: &str) -> Result<()> {
        unimplemented!()
    }
    fn get_option(&self, _k: &str) -> Option<String> {
        self.auto_fetch.clone()
    }
    fn run(&self, args: &[&str]) -> Result<(i32, String, String)> {
        if args.len() >= 2 && args[0] == "display-message" {
            self.display_msgs.borrow_mut().push(args[1].to_string());
        }
        Ok((0, String::new(), String::new()))
    }
    fn kill_window(&self, _n: &str) -> Result<()> {
        self.kill_called.set(true);
        Ok(())
    }
}
