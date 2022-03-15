use crate::domain::about_me::AboutMe;
use crate::view::about_me_view::AboutMeView;
pub fn to_model(view: &AboutMeView) -> AboutMe {
    AboutMe::new(
        None,
        view.first_name().to_string(),
        view.last_name.to_string(),
        view.description.to_owned(),
        None,
    )
}

pub fn to_view() -> AboutMeView {
    unimplemented!()
}
