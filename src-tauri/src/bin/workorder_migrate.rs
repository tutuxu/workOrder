//! 独立数据目录迁移工具（v1.0 → 当前版本升级）。

use std::env;
use std::path::PathBuf;

use workorder_lib::services::migration_service;

fn main() {
    let data_dir = parse_data_dir().unwrap_or_else(|msg| {
        eprintln!("{msg}");
        print_usage();
        std::process::exit(1);
    });

    match migration_service::run_data_migrations(&data_dir) {
        Ok(summary) => {
            println!("迁移完成。\n{}", summary.format_report());
        }
        Err(err) => {
            eprintln!("迁移失败：{err}");
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!();
    eprintln!("workOrder 数据迁移工具 — 从 v1.0 升级到当前版本");
    eprintln!();
    eprintln!("用法:");
    eprintln!("  workOrder-migrate.exe --data-dir <数据目录>");
    eprintln!();
    eprintln!("示例:");
    eprintln!("  workOrder-migrate.exe --data-dir .\\data");
    eprintln!();
    eprintln!("说明: 也可直接启动新版 workOrder.exe，首次运行会自动执行相同迁移。");
}

fn parse_data_dir() -> Result<PathBuf, String> {
    let mut args = env::args().skip(1);
    let mut data_dir: Option<PathBuf> = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--data-dir" => {
                let value = args
                    .next()
                    .ok_or_else(|| "缺少 --data-dir 参数值".to_string())?;
                data_dir = Some(PathBuf::from(value));
            }
            "--help" | "-h" => return Err("显示帮助".into()),
            other => return Err(format!("未知参数：{other}")),
        }
    }
    data_dir.ok_or_else(|| "必须指定 --data-dir".into())
}
