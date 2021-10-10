use chrono::{DateTime, NaiveDateTime, Utc};

/// Hello holds a row from the `hello` table.
///
/// Models are usually defined in the dal crate, but this is here so we don't pollute that crate
/// with examples.
#[derive(Debug)]
pub struct Hello {
    pub id: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,

    pub name: String,
    pub status: Option<HelloStatus>,
}

// Super janky. Needed because sqlx derive is very limited and implementing FromRow ourselves is
// herrendously complicated.
#[derive(sqlx::FromRow)]
pub struct HelloRow {
    pub id: u64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,

    pub name: String,
}

impl From<HelloRow> for Hello {
    fn from(row: HelloRow) -> Self {
        Self {
            id: row.id,
            created_at: DateTime::from_utc(row.created_at, Utc),
            updated_at: DateTime::from_utc(row.updated_at, Utc),
            deleted_at: row.deleted_at.map(|time| DateTime::from_utc(time, Utc)),
            name: row.name,
            status: None,
        }
    }
}

/// HelloStatus holds a row from the `hello_status` table.
///
/// Models are usually defined in the dal crate, but this is here so we don't pollute that crate
/// with examples.
#[derive(Debug)]
pub struct HelloStatus {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,

    pub hello_id: u64,
    pub message: String,
}

// Super janky. Needed because sqlx derive is very limited and implementing FromRow ourselves is
// herrendously complicated.
#[derive(sqlx::FromRow)]
pub struct HelloStatusRow {
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,

    pub hello_id: u64,
    pub message: String,
}

impl From<HelloStatusRow> for HelloStatus {
    fn from(row: HelloStatusRow) -> Self {
        Self {
            created_at: DateTime::from_utc(row.created_at, Utc),
            updated_at: DateTime::from_utc(row.updated_at, Utc),
            deleted_at: row.deleted_at.map(|time| DateTime::from_utc(time, Utc)),
            hello_id: row.hello_id,
            message: row.message,
        }
    }
}
