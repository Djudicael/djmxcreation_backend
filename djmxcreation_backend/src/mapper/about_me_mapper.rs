use crate::{domain::about_me::AboutMe, view::about_me_view::AboutMeView};

pub fn to_model(view: &AboutMeView) -> AboutMe {
    AboutMe::new(
        None,
        view.first_name().to_string(),
        view.last_name.to_string(),
        view.description.to_owned(),
        None,
    )
}

pub fn to_view(
    id: Option<i32>,
    first_name: &str,
    last_name: &str,
    description:Option<String>,
    picture: Option<String>,
) -> AboutMeView {
    AboutMeView::new(
        id,
        first_name.to_owned(),
        last_name.to_owned(),
        description,
        picture,
    )
}
