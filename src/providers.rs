use crate::icon_map::{meteocon_url, owm_icon};
use anyhow::{Result, anyhow};
use serde::Deserialize;

#[derive(Debug)]
pub struct WeatherResult {
	pub temperature: String,
	pub icon_url: String,
}

pub async fn fetch_weather(
	client: &reqwest::Client,
	api_key: &str,
	city: &str,
	celsius: bool,
	round: bool,
) -> Result<WeatherResult> {
	fetch_owm(client, api_key, city, celsius, round).await
}

// ── OpenWeatherMap ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct OwmWeather {
	icon: String,
}

#[derive(Deserialize)]
struct OwmMain {
	temp: f64,
}

#[derive(Deserialize)]
struct OwmResponse {
	main: OwmMain,
	weather: Vec<OwmWeather>,
}

async fn fetch_owm(
	client: &reqwest::Client,
	api_key: &str,
	city: &str,
	celsius: bool,
	round: bool,
) -> Result<WeatherResult> {
	let units = if celsius { "metric" } else { "imperial" };
	let url = format!(
		"https://api.openweathermap.org/data/2.5/weather?q={city}&appid={api_key}&units={units}"
	);
	let resp: OwmResponse = client
		.get(&url)
		.send()
		.await?
		.error_for_status()?
		.json()
		.await?;

	let icon_code = resp
		.weather
		.first()
		.map(|w| w.icon.as_str())
		.unwrap_or("01d");
	let icon_name = owm_icon(icon_code);

	Ok(WeatherResult {
		temperature: format_temp(resp.main.temp, celsius, round),
		icon_url: meteocon_url(icon_name),
	})
}

// ── Shared helpers ────────────────────────────────────────────────────────────

fn format_temp(val: f64, celsius: bool, round: bool) -> String {
	let suffix = if celsius { "°C" } else { "°F" };
	if round {
		format!("{:.0}{suffix}", val)
	} else {
		format!("{:.1}{suffix}", val)
	}
}

pub async fn fetch_icon_data_url(client: &reqwest::Client, icon_url: &str) -> Result<String> {
	let bytes = client
		.get(icon_url)
		.send()
		.await?
		.error_for_status()?
		.bytes()
		.await?;
	use base64::Engine as _;
	let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
	// Icons are SVG — OpenDeck rasterizes to a single JPEG frame via canvas,
	// so animation never plays; the static frame is what renders on the key.
	Ok(format!("data:image/svg+xml;base64,{b64}"))
}

pub fn make_client() -> Result<reqwest::Client> {
	reqwest::Client::builder()
		.timeout(std::time::Duration::from_secs(10))
		.build()
		.map_err(|e| anyhow!("Failed to build HTTP client: {e}"))
}
