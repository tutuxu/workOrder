# workOrder Cursor 规则说明

本文档说明从 `xiamen-bicycle-server` 本地化复制到本项目的 Cursor 规则，以及使用方式。

## 文件位置

```
.cursor/
├── rules/
│   ├── auto-git-track-new-files.mdc   # 新建 .java/.xml/.sql 自动 git add
│   ├── design-options.mdc             # @design-options 多方案对比（手动触发）
│   ├── java-debug-friendly.mdc        # @java-debug-friendly 可维护/可调试写法
│   ├── java-javadoc.mdc               # @java-javadoc JavaDoc 与分段注释
│   ├── java-naming.mdc                # @java-naming 命名规范
│   └── java-package-abbrev-whitelist.mdc  # @java-package-abbrev-whitelist 缩写白名单
└── skills/
    └── naming-translate/
        └── SKILL.md                   # @naming-translate 中英命名翻译
```

**来源**：`E:\timeloit\2026.0319_0919\workSpace\xiamen-bicycle-server\xiamen-bicycle-server\.cursor\`

## 本地化调整摘要

| 项 | xiamen-bicycle-server | workOrder |
|----|----------------------|-----------|
| globs | `loit-service/loit-bicycle/**/*.java` | `src/main/java/**/*.java` |
| 包根路径 | `com.loit.modules.*` | `com.workorder.*` |
| 技术栈引用 | Spring + MyBatis | Spring Boot 3 + Vaadin + JPA + SQLite + Flyway |
| 分层 | Controller / Service / Dao / XML | View / Service / Repository / Entity |
| git add 后缀 | `.java`、`.xml` | `.java`、`.xml`、`.sql`（Flyway） |
| 示例实体 | Bicycle*、RegionMgr* | WorkOrder、ProgressLog |

## 规则速查

| 触发方式 | 用途 |
|----------|------|
| （自动）修改 `src/main/java/**/*.java` | `@java-javadoc`、`@java-naming`、`@java-debug-friendly` 按 glob 生效 |
| （始终）新建源码文件 | `auto-git-track-new-files` 自动 `git add` |
| `@design-options` | 多方案对比，选型前禁止写代码 |
| `@naming-translate` | 中英命名翻译 Skill |
| `白名单添加 xxx` | 维护缩写白名单 |
| `白名单初始化` | 按 `com.workorder` 包扫描重建业务缩写 |

## 与 plan 文档关系

- 编码阶段应同时遵循 `plan/需求文档.md`、`plan/技术选型.md`、`plan/实现计划.md`
- 命名与注释规范以 `.cursor/rules/` 为准，Agent 修改 Java 代码时自动对齐

## 后续建议

1. **P1 脚手架完成后**执行一次 `白名单初始化`，扫描 `com.workorder` 包更新业务缩写表
2. 若 Vaadin 视图类命名惯例稳定，可在 `java-naming.mdc` 补充 `*View`、`*Dialog` 示例
3. 初始化 Git 仓库后，`auto-git-track-new-files` 规则方可生效
