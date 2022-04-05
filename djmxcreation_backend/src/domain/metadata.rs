pub struct Metadata {
    title: Option<String>,
    sub_title: Option<String>,
    client: Option<String>,
}

impl Metadata {
    pub fn new(title: Option<String>, sub_title: Option<String>, client: Option<String>) -> Self {
        Self {
            title,
            sub_title,
            client,
        }
    }

    pub fn title(&self) -> Option<&String> {
        self.title.as_ref()
    }

    pub fn sub_title(&self) -> Option<&String> {
        self.sub_title.as_ref()
    }

    pub fn client(&self) -> Option<&String> {
        self.client.as_ref()
    }
}
