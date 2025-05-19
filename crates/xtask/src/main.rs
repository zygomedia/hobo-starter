use clap::{Parser, Subcommand};
use cmd_lib::{run_cmd as cmd, run_fun as cmd_out};

#[derive(Parser)]
#[command(bin_name = "cargo xtask")]
struct Cli {
	#[command(subcommand)]
	command: Command,
}

#[derive(clap::ValueEnum, PartialEq, Eq, Clone, Copy)]
enum BuildComponent {
	Server,
	Client,
}

#[derive(Subcommand)]
enum Command {
	Check {
		#[arg(short, long)] debug: bool,
		#[arg(short, long)] release: bool,
	},
	Watch,
	WatchClient,
	WatchServer,
	Build {
		#[arg(long = "only", value_enum, value_delimiter = ',', num_args = 1..)]
		only: Vec<BuildComponent>,
		#[arg(long = "skip", value_enum, value_delimiter = ',', num_args = 1..)]
		skip: Vec<BuildComponent>,
	},
	BuildClient {
		#[arg(short, long)] release: bool,
	},
}

fn handle_command(command: Command) {
	match command {
		Command::Check { mut debug, mut release } => {
			if !(debug || release) { debug = true; release = true; }

			let packages = [
				("shared", ""),
				("server", ""),
			];

			if debug { handle_command(Command::BuildClient { release: false }); }
			if release { handle_command(Command::BuildClient { release: true }); }

			for (p, extra) in packages {
				if debug { cmd!(cargo check -p $p $extra).unwrap(); }
				if release { cmd!(cargo check --release -p $p $extra).unwrap(); }
			}
		},
		Command::Watch => {
			if cmd!(watchexec --version).is_err() {
				cmd!(cargo install watchexec-cli).unwrap();
			}

			let deps = [
				"crates/client",
				"crates/server",
				"crates/shared",
				"misc/response.html",
			].iter().flat_map(|x| ["-w", x]);

			cmd!(watchexec -r --debounce 1500 --no-project-ignore $[deps] "cargo xtask build-client; cargo run -p server --bin server").unwrap();
		},
		Command::WatchClient => {
			if cmd!(watchexec --version).is_err() {
				cmd!(cargo install watchexec-cli).unwrap();
			}

			let deps = [
				"crates/client",
				"crates/shared",
				"misc/response.html",
			].iter().flat_map(|x| ["-w", x]);

			cmd!(watchexec -r --debounce 1500 --no-project-ignore $[deps] "cargo xtask build-client").unwrap();
		},
		Command::WatchServer => {
			// v1.25
			if cmd!(watchexec --version).is_err() {
				cmd!(cargo install watchexec-cli).unwrap();
			}

			let deps = [
				"crates/client",
				"crates/server",
				"crates/shared",
				"misc/response.html",
			].iter().flat_map(|x| ["-w", x]);

			cmd!(watchexec -r --debounce 1500 --no-project-ignore $[deps] "cargo run -p server --bin server").unwrap();
		},
		Command::Build { only, skip } => {
			assert!(only.is_empty() || (only.len() == 1 && skip.is_empty()));
			let build_server = only.contains(&BuildComponent::Server) || (only.is_empty() && !skip.contains(&BuildComponent::Server));
			let build_client = only.contains(&BuildComponent::Client) || (only.is_empty() && !skip.contains(&BuildComponent::Client));

			if build_client {
				handle_command(Command::BuildClient { release: true });
			}

			if build_server {
				// uncomment this if building on Windows but targeting a Linux server
				// cmd!(wsl -d Ubuntu bash -c "source ~/.profile && cargo build --package server --release").unwrap();

				cmd!(cargo build --package server --release).unwrap();
			}
		},
		Command::BuildClient { release } => {
			if cmd!(wasm-bindgen -V).is_err() {
				cmd!(cargo install wasm-bindgen-cli -f).unwrap();
			}
			if cmd!(wasm-opt --version).is_err() {
				cmd!(cargo install wasm-opt -f).unwrap();
			}
			if !cmd_out!(rustup target list --installed).unwrap().lines().any(|x| x == "wasm32-unknown-unknown") {
				cmd!(rustup target add wasm32-unknown-unknown).unwrap();
			}

			let cargo_build_profile = if release { "--release" } else { "" };
			cmd!(cargo build -p client --target wasm32-unknown-unknown --target-dir target/wasm-target $cargo_build_profile).unwrap();

			let wasm_bindgen_params = if release {
				vec![
					"--out-dir", "target/wasm-target/wasm32-unknown-unknown/release",
					"--remove-name-section",
					"--remove-producers-section",
					"target/wasm-target/wasm32-unknown-unknown/release/client.wasm",
				]
			} else {
				vec![
					"--out-dir", "target/wasm-target/wasm32-unknown-unknown/debug",
					"--debug",
					"target/wasm-target/wasm32-unknown-unknown/debug/client.wasm",
				]
			};

			cmd!(wasm-bindgen --out-name client_bound --target web --no-typescript --omit-imports --omit-default-module-path --reference-types $[wasm_bindgen_params]).unwrap();

			if release {
				cmd!(wasm-opt target/wasm-target/wasm32-unknown-unknown/release/client_bound_bg.wasm --enable-reference-types --enable-bulk-memory --enable-nontrapping-float-to-int -o target/wasm-target/wasm32-unknown-unknown/release/client_bound_bg.wasm -Os).unwrap();
			}
		},
	}
}

fn main() {
	let cli = Cli::parse();
	handle_command(cli.command);
}
