use serde_json::Value;

use crate::view::about_me_view::AboutMeView;

pub fn to_view(
    id: Option<i32>,
    first_name: &str,
    last_name: &str,
    description: Option<&Value>,
    picture: Option<String>,
) -> AboutMeView {
    AboutMeView::new(
        id,
        first_name.to_owned(),
        last_name.to_owned(),
        description.cloned(),
        picture,
    )
}
