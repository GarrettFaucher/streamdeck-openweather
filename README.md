# streamdeck-openweather

An [OpenDeck](https://github.com/nekename/OpenDeck) / Elgato Stream Deck plugin written in Rust that displays current weather conditions and temperature for any city.

Supports WeatherAPI and OpenWeatherMap. Animated weather icons from [Meteocons](https://github.com/basmilius/weather-icons) by Bas Milius (MIT) — fetched as animated SVGs and rendered natively by the host.

## Features

- Live temperature and weather condition for any city
- Choose between WeatherAPI and OpenWeatherMap (both free-tier compatible)
- Animated Meteocon SVG icons, fetched on demand (no bundled assets)
- Per-key city, units, and display options via Property Inspector
- Manual refresh on key press; configurable poll interval (10 min / 30 min / 1 hr)
- Shared API key across all instances via global settings

## Compatibility

| Host | Supported |
|------|-----------|
| OpenDeck (Linux x86_64) | Yes — primary target |
| OpenDeck (Linux aarch64) | Builds in CI |
| OpenDeck (macOS x86_64 / aarch64) | Builds in CI, untested |
| Elgato Stream Deck (Windows) | Builds in CI, untested |

This plugin uses the [`openaction`](https://github.com/OpenActionAPI/rust) crate and stays within the Elgato-compatible feature subset (`set_image`, `set_title`, settings, PI, Keypad controller only).

## Requirements

- Rust 1.85+ (edition 2024)
- A [WeatherAPI](https://weatherapi.com/my/) or [OpenWeatherMap](https://home.openweathermap.org/api_keys) API key (free tier works)

## Build

```sh
cargo build --release
```

Binary lands at `target/release/oaopenweather`.

## Install (OpenDeck on Linux)

```sh
./deploy.sh
```

Or manually:

```sh
PLUGIN_DIR=~/.config/opendeck/plugins/com.garrettfaucher.openweather.sdPlugin
TARGET=x86_64-unknown-linux-gnu

mkdir -p "$PLUGIN_DIR"
cp -r assets/. "$PLUGIN_DIR/"
cp target/release/oaopenweather "$PLUGIN_DIR/oaopenweather-$TARGET"
```

Restart OpenDeck, drag **Weather** from the Open Weather category onto a key, then open the property inspector to enter your API key and city.

## Settings (Property Inspector)

| Setting | Description |
|---------|-------------|
| Provider | WeatherAPI or OpenWeatherMap |
| API Key | Your provider key (shared across all instances) |
| City Name | City to show weather for |
| Temperature | Celsius or Fahrenheit |
| Fetch frequency | Manual, 10 min, 30 min, or 1 hour |
| Display city | Show city name in title (positions 0–4) |
| Round degree | Round temperature to nearest integer |

Press the key to force an immediate refresh at any time.

## Animated icons

The plugin fetches animated SVG weather icons from the [Bas Milius Meteocon CDN](https://cdn.jsdelivr.net/gh/basmilius/weather-icons@v2.0.0/production/fill/animated/) and sends them as `data:image/svg+xml;base64,...` data URLs. Icons are not bundled — they require network access on each fetch.

## Project structure

```
streamdeck-openweather/
├── .github/workflows/build.yml   # 5-target CI build matrix
├── Cargo.toml                    # package: oaopenweather
├── rustfmt.toml                  # hard tabs, Unix line endings
├── assets/
│   ├── pi/                       # Property Inspector (HTML/JS/CSS)
│   │   ├── main_pi.html
│   │   ├── main_pi.js
│   │   ├── sdpi.css
│   │   └── caret.svg
│   ├── imgs/weather.png          # action icon
│   ├── icon.png                  # plugin icon
│   └── manifest.json
└── src/
    ├── main.rs         (entry point)
    ├── global.rs       (GlobalEventHandler — receives apiKey/provider)
    ├── weather.rs      (WeatherAction — per-instance poll loop)
    ├── providers.rs    (WeatherAPI + OWM HTTP fetchers)
    └── icon_map.rs     (condition code → meteocon name mapping tables)
```

## Credits

Weather icons by [Bas Milius](https://github.com/basmilius/weather-icons) (MIT).

## License

MIT — see [LICENSE](LICENSE).
