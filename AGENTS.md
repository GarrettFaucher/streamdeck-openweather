# Agent Guide

## What this repo is

A Rust OpenAction plugin that fetches weather from WeatherAPI or OpenWeatherMap and displays the temperature + animated SVG icon on a Stream Deck key.

Uses the `openaction` crate (OpenAction protocol, compatible with both OpenDeck and Elgato Stream Deck software).

## Build

```sh
cargo build --release        # binary: target/release/oaopenweather
cargo check                  # fast type-check without linking
```

Edition 2024 — requires Rust 1.85+.

## Project layout

| File | Purpose |
|------|---------|
| `src/main.rs` | Entry point — registers global handler and action, calls `run()` |
| `src/global.rs` | `GlobalEventHandler` impl — receives `apiKey`+`provider` from the PI; triggers refresh on all visible instances |
| `src/weather.rs` | `WeatherAction` — spawns per-instance poll loop; `key_down` wakes it early via `Notify` |
| `src/providers.rs` | `fetch_weather()` for WeatherAPI and OWM; `fetch_icon_data_url()` for CDN icons |
| `src/icon_map.rs` | `owm_icon(code)` and `wapi_icon(code, is_day)` mapping tables; `meteocon_url(name)` |
| `assets/manifest.json` | Plugin metadata, dual-host CodePaths |
| `assets/pi/` | Property Inspector (HTML/JS/CSS) — the PI speaks the standard OpenAction protocol |

## Global settings flow

1. On `plugin_ready`, the handler calls `openaction::get_global_settings()`.
2. When the PI calls `setGlobalSettings` with `{ apiKey, provider }`, the host delivers `didReceiveGlobalSettings` to the plugin.
3. `global.rs::Handler::did_receive_global_settings` writes to `GLOBAL` (a `LazyLock<RwLock<GlobalSettings>>`), then sends `force_refresh(id)` to every visible instance.
4. Each instance's poll task reads from `GLOBAL` before each fetch — no restart needed when the key changes.

## Per-instance poll loop

- Spawned in `will_appear`; cancelled when `get_instance()` returns `None` (i.e., `will_disappear` removed the instance).
- A `Notify` per instance is stored in `NOTIFIERS: LazyLock<DashMap<...>>`.
- `key_down` and `did_receive_settings` both call `force_refresh()` which sends a `notify_one()`.
- Frequency is read from the instance's `WeatherSettings.frequency` on each loop iteration.

## Icons

Icons are fetched at runtime from `cdn.jsdelivr.net/gh/basmilius/weather-icons@v2.0.0/production/fill/all/`.
They are sent as `data:image/svg+xml;base64,...`. OpenDeck rasterizes images to a single JPEG frame via `<img>` + `canvas.drawImage()`, so animation never runs — the static frame is what appears on the key. The `all/` path is correct for v2.0.0 of the icon set; `animated/` does not exist at that tag.

## Reference

- openaction API: `~/.cargo/registry/src/**/openaction-2.6.*/`
- OpenAction docs: https://github.com/OpenActionAPI/docs
- Meteocons: https://github.com/basmilius/weather-icons
