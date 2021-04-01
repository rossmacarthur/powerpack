#[test]
fn main_template() {
    #[allow(dead_code)]
    mod main {
        include!("../src/main.rs.template");
    }
}
