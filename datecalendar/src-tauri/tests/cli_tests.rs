//! CLI 工具集成测试
//! 
//! 运行方式：
//! ```bash
//! cargo test --test cli_tests
//! ```

use std::process::Command;
use std::path::PathBuf;
use std::fs;

/// 获取测试用的 CLI 可执行文件路径
fn get_cli_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap();
    path.push("target");
    path.push("debug");
    path.push("datecalendar-cli.exe");
    
    // 如果 debug 不存在，尝试 release
    if !path.exists() {
        path = std::env::current_dir().unwrap();
        path.push("target");
        path.push("release");
        path.push("datecalendar-cli.exe");
    }
    
    path
}

/// 获取测试用的数据库路径（使用临时数据库）
fn get_test_db_path() -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push("datecalendar_test.db");
    path
}

/// 获取数据库路径的字符串形式
fn get_test_db_string() -> String {
    get_test_db_path().to_string_lossy().to_string()
}

/// 清理测试数据库
fn cleanup_test_db() {
    let db_path = get_test_db_path();
    if db_path.exists() {
        fs::remove_file(&db_path).ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 辅助函数：运行 CLI 命令并返回输出
    fn run_cli(args: &[&str]) -> (bool, String, String) {
        let cli = get_cli_path();
        let db_path = get_test_db_string();
        
        let output = Command::new(&cli)
            .arg("--db-path")
            .arg(&db_path)
            .args(args)
            .output()
            .expect("Failed to execute CLI");
        
        let success = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        (success, stdout, stderr)
    }
    
    #[test]
    fn test_health_command() {
        cleanup_test_db();
        
        let cli = get_cli_path();
        assert!(cli.exists(), "CLI executable not found at {:?}", cli);
        
        let (success, stdout, stderr) = run_cli(&["health"]);
        
        assert!(success, "health command failed: {}", stderr);
        assert!(stdout.contains("\"status\": \"ok\""), 
                "health output should contain status: ok, got: {}", stdout);
    }
    
    #[test]
    fn test_task_create_and_list() {
        cleanup_test_db();
        
        // 创建任务
        let (success, stdout, stderr) = run_cli(&["task", "create", "测试任务", "--priority", "1", "--description", "测试描述"]);
        
        assert!(success, "task create failed: {}", stderr);
        assert!(stdout.contains("\"title\": \"测试任务\""), 
                "create output should contain title, got: {}", stdout);
        assert!(stdout.contains("\"priority\": 1"), 
                "create output should contain priority, got: {}", stdout);
        
        // 列出任务
        let (success, stdout, stderr) = run_cli(&["task", "list", "--format", "json"]);
        
        assert!(success, "task list failed: {}", stderr);
        assert!(stdout.contains("\"title\": \"测试任务\""), 
                "list output should contain created task, got: {}", stdout);
    }
    
    #[test]
    fn test_task_update() {
        cleanup_test_db();
        
        // 先创建任务
        let (success, stdout, stderr) = run_cli(&["task", "create", "待更新任务"]);
        assert!(success, "task create failed: {}", stderr);
        
        // 提取任务 ID
        let id_start = stdout.find("\"id\": \"").unwrap() + 8;
        let id_end = stdout[id_start..].find("\"").unwrap() + id_start;
        let task_id = &stdout[id_start..id_end];
        
        // 更新任务
        let (success, stdout, stderr) = run_cli(&["task", "update", task_id, "--title", "已更新任务", "--status", "in_progress"]);
        
        assert!(success, "task update failed: {}", stderr);
        assert!(stdout.contains("\"title\": \"已更新任务\""), 
                "update output should contain new title, got: {}", stdout);
        assert!(stdout.contains("\"status\": \"in_progress\""), 
                "update output should contain new status, got: {}", stdout);
    }
    
    #[test]
    fn test_task_complete() {
        cleanup_test_db();
        
        // 先创建任务
        let (success, stdout, stderr) = run_cli(&["task", "create", "待完成任务"]);
        assert!(success, "task create failed: {}", stderr);
        
        // 提取任务 ID
        let id_start = stdout.find("\"id\": \"").unwrap() + 8;
        let id_end = stdout[id_start..].find("\"").unwrap() + id_start;
        let task_id = &stdout[id_start..id_end];
        
        // 标记完成
        let (success, stdout, stderr) = run_cli(&["task", "complete", task_id]);
        
        assert!(success, "task complete failed: {}", stderr);
        assert!(stdout.contains("\"status\": \"completed\""), 
                "complete output should contain completed status, got: {}", stdout);
        assert!(stdout.contains("\"completed_at\""), 
                "complete output should contain completed_at, got: {}", stdout);
    }
    
    #[test]
    fn test_task_search() {
        cleanup_test_db();
        
        // 创建两个任务
        run_cli(&["task", "create", "搜索测试任务1"]);
        run_cli(&["task", "create", "其他任务"]);
        
        // 搜索
        let (success, stdout, stderr) = run_cli(&["task", "search", "搜索"]);
        
        assert!(success, "task search failed: {}", stderr);
        assert!(stdout.contains("\"title\": \"搜索测试任务1\""), 
                "search output should contain matching task, got: {}", stdout);
        assert!(!stdout.contains("\"title\": \"其他任务\""), 
                "search output should not contain non-matching task, got: {}", stdout);
    }
    
    #[test]
    fn test_task_delete() {
        cleanup_test_db();
        
        // 先创建任务
        let (success, stdout, stderr) = run_cli(&["task", "create", "待删除任务"]);
        assert!(success, "task create failed: {}", stderr);
        
        // 提取任务 ID
        let id_start = stdout.find("\"id\": \"").unwrap() + 8;
        let id_end = stdout[id_start..].find("\"").unwrap() + id_start;
        let task_id = &stdout[id_start..id_end];
        
        // 删除任务
        let (success, _, stderr) = run_cli(&["task", "delete", task_id]);
        assert!(success, "task delete failed: {}", stderr);
        
        // 验证已删除
        let (success, stdout, _) = run_cli(&["task", "list", "--format", "json"]);
        assert!(success);
        assert!(!stdout.contains(task_id), 
                "list output should not contain deleted task, got: {}", stdout);
    }
    
    #[test]
    fn test_output_formats() {
        cleanup_test_db();
        
        // 创建任务
        run_cli(&["task", "create", "格式测试任务"]);
        
        // 测试 JSON 格式
        let (success, stdout, stderr) = run_cli(&["task", "list", "--format", "json"]);
        assert!(success, "json format failed: {}", stderr);
        assert!(stdout.trim().starts_with("["), "JSON output should start with [, got: {}", stdout);
        
        // 测试 table 格式
        let (success, stdout, stderr) = run_cli(&["task", "list", "--format", "table"]);
        assert!(success, "table format failed: {}", stderr);
        assert!(stdout.contains("title"), "table output should contain column headers, got: {}", stdout);
    }
    
    #[test]
    fn test_help_output() {
        let cli = get_cli_path();
        
        // 测试主帮助
        let output = Command::new(&cli)
            .arg("--help")
            .output()
            .expect("Failed to execute CLI");
        
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("DateCalendar CLI"), "help should contain app name, got: {}", stdout);
        assert!(stdout.contains("task"), "help should contain task command, got: {}", stdout);
        assert!(stdout.contains("schedule"), "help should contain schedule command, got: {}", stdout);
        
        // 测试 task 子命令帮助
        let (success, stdout, stderr) = run_cli(&["task", "--help"]);
        assert!(success, "task help failed: {}", stderr);
        assert!(stdout.contains("create"), "task help should contain create, got: {}", stdout);
        assert!(stdout.contains("list"), "task help should contain list, got: {}", stdout);
    }
}
