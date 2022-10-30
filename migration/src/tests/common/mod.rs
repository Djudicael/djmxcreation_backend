struct MyResource {}

impl Drop for MyResource {
    fn drop(&mut self) {
        println!("Dropped my rescource");
    }
}

impl MyResource {
    pub fn new() -> MyResource {
        println!("Created MyResource");
        MyResource {}
    }
}

pub fn setup() {
    // some setup code, like creating required files/directories, starting
    // servers, etc.

    let test = MyResource::new();
}