mod global;
mod icon_map;
mod providers;
mod weather;

use openaction::*;
use weather::WeatherAction;

#[tokio::main]
async fn main() -> OpenActionResult<()> {
	{
		use simplelog::*;
		if let Err(e) = TermLogger::init(
			LevelFilter::Debug,
			Config::default(),
			TerminalMode::Stdout,
			ColorChoice::Never,
		) {
			eprintln!("Logger initialization failed: {e}");
		}
	}

	global::register();
	register_action(WeatherAction).await;
	run(std::env::args().collect()).await
}
