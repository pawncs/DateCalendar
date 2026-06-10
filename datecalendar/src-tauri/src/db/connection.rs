use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::PathBuf;
use std::sync::Mutex;

/// 数据库连接池的类型别名
pub type DbPool = Pool<SqliteConnectionManager>;

/// 应用数据目录缓存，用于在 Tauri 上下文可用前确定 DB 路径
static DB_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

/// 初始化数据库连接池
///
/// 在 Tauri 应用数据目录下创建 datecalendar.db
/// 连接池大小设为 4（SQLite 写锁特性，更多连接无意义）
pub fn init_pool(app_data_dir: PathBuf) -> Result<DbPool, Box<dyn std::error::Error>> {
    std::fs::create_dir_all(&app_data_dir)?;
    let db_path = app_data_dir.join("datecalendar.db");

    let manager = SqliteConnectionManager::file(&db_path);
    let pool = Pool::builder()
        .max_size(4)
        .build(manager)?;

    // 缓存 DB 路径供 CLI 模式使用
    if let Ok(mut path) = DB_PATH.lock() {
        *path = Some(db_path);
    }

    Ok(pool)
}

/// 获取数据库文件路径（用于 CLI 模式）
pub fn get_db_path() -> Option<PathBuf> {
    DB_PATH.lock().ok()?.clone()
}
