/// Map OWM icon codes to Bas Milius meteocon names.
/// Code is the `weather[0].icon` field from the OWM API response (e.g. "01d").
pub fn owm_icon(code: &str) -> &'static str {
	match code {
		"01d" => "clear-day",
		"01n" => "clear-night",
		"02d" => "partly-cloudy-day",
		"02n" => "partly-cloudy-night",
		"03d" | "03n" => "cloudy",
		"04d" => "overcast-day",
		"04n" => "overcast-night",
		"09d" | "09n" => "rain",
		"10d" => "partly-cloudy-day-rain",
		"10n" => "partly-cloudy-night-rain",
		"11d" => "thunderstorms-day",
		"11n" => "thunderstorms-night",
		"13d" | "13n" => "snow",
		"50d" | "50n" => "mist",
		_ => "not-available",
	}
}

pub fn meteocon_url(name: &str) -> String {
	format!(
		"https://cdn.jsdelivr.net/gh/basmilius/weather-icons@v2.0.0/production/fill/all/{name}.svg"
	)
}
