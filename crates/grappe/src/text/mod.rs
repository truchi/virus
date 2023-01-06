pub mod meta;

use self::meta::TextMeta;
use crate::page::Page;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Text {
    pages: Arc<Vec<Page>>,
    meta: TextMeta,
}
