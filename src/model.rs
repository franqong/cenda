pub enum Status {
    Pass,
    Warn,
    Info,
    Error,
}

pub enum Category {
    Ssh,
    Users,
    Permissions,
}

pub struct CheckResult {
    pub id: String,
    pub category: Category,
    pub status: Status,
    pub message: String,
    pub recommendation: Option<String>,
}

pub struct PasswdEntry {
    pub username: String,
    pub uid: u32,
    pub gid: u32,
    pub home: String,
    pub shell: String
}

pub enum ParseError {
    InvalidFieldCount,
    EmptyUsername,
    InvalidUid,
    InvalidGid,
}
