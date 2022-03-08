use  error::Error;

mod error;
mod router;
pub async fn start()->Result<(), Error> {
// Apis
let apis = todo_rest_filters("api", db);

}