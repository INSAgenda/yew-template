use yew_template::*;
use yew::prelude::*;

fn main() {
    let html = template_html!("tests/test.html", value="tes", value2={5.to_string()}, boobool=true);

}