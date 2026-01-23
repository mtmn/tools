use lofty::tag::{ItemKey, Tag};
use std::borrow::Cow;

pub fn get_label(tag: &Tag) -> Option<Cow<'_, str>> {
    tag.get_string(&ItemKey::Label)
        .map(Cow::Borrowed)
}
