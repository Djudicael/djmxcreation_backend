use serde_json::Value;
use sqlx::types::Json;

use crate::{domain::about_me::AboutMe, view::about_me_view::AboutMeView};

pub fn to_model(view: &AboutMeView) -> AboutMe {
    AboutMe::new(
        None,
        view.first_name().to_string(),
        view.last_name.to_string(),
        view.description()
            .map(|description| Json(description.clone())),
        None,
    )
}

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
        description.map(|desc| desc.clone()),
        picture,
    )
}
