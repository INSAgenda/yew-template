use yew_template::*;
use yew::prelude::*;

fn main() {
    let boobool = false.to_string();
    let _html = template_html!("tests/test.html", value="tes", value2={5.to_string()}, boobool, opt_value={Some("tes")}, opt_value2={Some("optvalue2")});
}
