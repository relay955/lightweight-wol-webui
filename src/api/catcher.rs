#[catch(422)]
fn foo_not_found() -> &'static str {
    "Foo 404"
}