fn reply() -> Vec<u8> {
	fn minify_js() -> String {
		let opts = swc::config::Options {
			config: swc::config::Config {
				minify: swc::BoolConfig::new(Some(true)),
				jsc: swc::config::JscConfig {
					target: Some(swc_ecma_ast::EsVersion::EsNext),
					minify: Some(swc::config::JsMinifyOptions {
						compress: swc::BoolOrDataConfig::from_bool(true),
						mangle: swc::BoolOrDataConfig::from_bool(true),
						..Default::default()
					}),
					..Default::default()
				},
				..Default::default()
			},
			..Default::default()
		};

		let cm = std::sync::Arc::<swc_common::SourceMap>::default();
		let output = swc_common::GLOBALS.set(&Default::default(), ||
			swc::try_with_handler(std::sync::Arc::clone(&cm), Default::default(), |handler| {
				#[cfg(debug_assertions)] let mut file = std::fs::read_to_string("target/wasm-target/wasm32-unknown-unknown/debug/client_bound.js").expect("couldn't find the js payload");
				#[cfg(not(debug_assertions))] let mut file = include_str!("../../../target/wasm-target/wasm32-unknown-unknown/release/client_bound.js").to_owned();
				file.push_str(r#";__wbg_init({ module_or_path: "/payload.wasm" });"#);
				let file = cm.new_source_file(swc_common::sync::Lrc::new(std::path::PathBuf::from("client_bound.js").into()), file);
				Ok(swc::Compiler::new(cm)
					.process_js_file(file, handler, &opts)
					.expect("failed to process js payload"))
			})
		).unwrap();

		output.code
	}

	fn make_reply(js: &str) -> Vec<u8> {
		minify_html::minify(
			format!(include_str!("../../../misc/response.html"),
				js = js,
			).as_bytes(),
			&minify_html::Cfg {
				minify_doctype: false,
				allow_noncompliant_unquoted_attribute_values: false,
				keep_closing_tags: true,
				keep_html_and_head_opening_tags: true,
				allow_removing_spaces_between_attributes: false,
				minify_js: false,
				..Default::default()
			},
		)
	}

	if cfg!(debug_assertions) {
		make_reply(&minify_js())
	} else {
		static JS: std::sync::LazyLock<String> = std::sync::LazyLock::new(minify_js);
		make_reply(&JS)
	}
}

#[cfg(debug_assertions)] fn wasm_payload() -> Vec<u8> { std::fs::read("target/wasm-target/wasm32-unknown-unknown/debug/client_bound_bg.wasm").unwrap() }
#[cfg(not(debug_assertions))] fn wasm_payload() -> &'static [u8] { include_bytes!("../../../target/wasm-target/wasm32-unknown-unknown/release/client_bound_bg.wasm") }

#[tokio::main]
async fn main() {
	use axum::{Router, routing::{get, post}, response::IntoResponse, http::header::CONTENT_TYPE};

	let app = Router::new()
		.route("/favicon.ico", get(async || ([(CONTENT_TYPE, "image/x-icon")], include_bytes!("../../../public/img/favicon.ico"))))
		.route("/payload.wasm", get(async || ([(CONTENT_TYPE, "application/wasm")], wasm_payload())))
		/*.route("/ws", any(async |ws: WebSocketUpgrade| ws.on_upgrade(async |mut socket: WebSocket| {
			while let Some(Ok(msg)) = socket.recv().await {
			}
		})))*/
		.route("/api", post(async |bytes: axum::body::Bytes| {
			pu_239::build_api!(["crates/client/src/lib.rs"]);

			match deserialize_api_match(&*bytes).await {
				Ok(x) => x.into_response(),
				Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
			}
		}))
		.route("/", get(async || ([(CONTENT_TYPE, "text/html; charset=utf-8")], reply())));

	let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
	axum::serve(listener, app).await.unwrap();
}
