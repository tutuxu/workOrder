CREATE TABLE IF NOT EXISTS work_order (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'NOT_STARTED',
    priority INTEGER NOT NULL DEFAULT 0,
    waiting_for VARCHAR(255),
    waiting_reason VARCHAR(255),
    extra_fields TEXT,
    due_date TIMESTAMP,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS progress_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    work_order_id INTEGER NOT NULL,
    title VARCHAR(255) NOT NULL,
    content TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'NOT_STARTED',
    extra_fields TEXT,
    created_at TIMESTAMP NOT NULL,
    FOREIGN KEY (work_order_id) REFERENCES work_order(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_work_order_status ON work_order(status);
CREATE INDEX IF NOT EXISTS idx_work_order_priority ON work_order(priority);
CREATE INDEX IF NOT EXISTS idx_progress_log_work_order_id ON progress_log(work_order_id);

CREATE TABLE IF NOT EXISTS attachment (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    owner_type VARCHAR(50) NOT NULL,
    owner_id INTEGER NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    original_name VARCHAR(255),
    mime_type VARCHAR(100) NOT NULL,
    file_size INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_attachment_owner ON attachment(owner_type, owner_id);
