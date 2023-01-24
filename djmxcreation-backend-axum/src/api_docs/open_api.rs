use aide::{openapi::Tag, transform::TransformOpenApi};
use axum::Json;

pub fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
    api.title("Aide axum Open API")
        .summary("An example Todo application")
        // .description(include_str!("README.md"))
        .tag(Tag {
            name: "todo".into(),
            description: Some("Todo Management".into()),
            ..Default::default()
        })
        .security_scheme(
            "ApiKey",
            aide::openapi::SecurityScheme::ApiKey {
                location: aide::openapi::ApiKeyLocation::Header,
                name: "X-Auth-Key".into(),
                description: Some("A key that is ignored.".into()),
                extensions: Default::default(),
            },
        )
    // .default_response_with::<Json<AppError>, _>(|res| {
    //     res.example(AppError {
    //         error: "some error happened".to_string(),
    //         error_details: None,
    //         error_id: Uuid::nil(),
    //         // This is not visible.
    //         status: StatusCode::IM_A_TEAPOT,
    //     })
    // })
}
