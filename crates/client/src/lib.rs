use tap::prelude::*;
use hobo::{create as e, prelude::*, signal::Mutable};

mod api {
	static CLIENT: std::sync::LazyLock<reqwest::Client> = std::sync::LazyLock::new(reqwest::Client::new);

	pub async fn dispatch(serialized: Vec<u8>) -> reqwest::Result<impl std::ops::Deref<Target = [u8]>> {
		CLIENT.post("http://localhost:3000/api")
			.body(serialized)
			.send().await?.error_for_status()?
			.bytes().await
	}
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
	wasm_log::init(wasm_log::Config::default());
	console_error_panic_hook::set_once();

	web_sys::window().unwrap()
		.document().unwrap()
		.body().unwrap()
		.tap(|body| body.set_inner_html(""))
		.pipe(|body| e::Body(e::html_element(&body)))
		.allow_no_parent()
		.pipe(root);

	log::info!("mounted successfully!");
}

fn root(element: e::Body) {
	let some_val = Mutable::new(shared::SharedType::Var1 { a: 1, b: true });
	element
		.class(css::style!{
			.& {
				css::display::flex,
				css::flex_direction::column,
				css::gap::px(16),
			}
			
			.& > div {
				css::background_color::rgba(css::colors::PALEVIOLETRED),
				css::border_radius::px(8),
				css::padding::px(16),
			}
		})
		.child(e::div().text("Hello Client."))
		.child(e::div().text_signal(some_val.signal_ref(|x| match x {
			shared::SharedType::Var1 { a, b } => format!("{a} {b}"),
			shared::SharedType::Var2(x) => x.clone(),
		})))
		.spawn(async move {
			#[pu_239::server]
			pub async fn fetch_some_val() -> shared::SharedType {
				println!("fetch_some_val called");
				shared::SharedType::Var2("magnificent".to_owned())
			}

			let Ok(res) = fetch_some_val().await else { return; };
			*some_val.lock_mut() = res;
		});
}
