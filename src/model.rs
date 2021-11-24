use chrono::{DateTime, FixedOffset};

pub struct GitLogElement {
    pub(crate) commit_id: String,
    pub(crate) date: DateTime<FixedOffset>,
    pub(crate) commit_message: String,
}
