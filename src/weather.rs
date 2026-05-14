use crate::global::GLOBAL;
use crate::providers::{fetch_icon_data_url, fetch_weather, make_client};
use async_trait::async_trait;
use dashmap::DashMap;
use openaction::*;
use serde::{Deserialize, Deserializer, Serialize};
use std::sync::{Arc, LazyLock};
use tokio::sync::Notify;

fn deser_bool_or_str<'de, D: Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
	use serde::de::Error;
	match serde_json::Value::deserialize(d)? {
		serde_json::Value::Bool(b) => Ok(b),
		serde_json::Value::String(s) => Ok(s != "false"),
		_ => Err(D::Error::custom("expected bool or string for roundDegree")),
	}
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct WeatherSettings {
	#[serde(rename = "cityName")]
	pub city_name: String,
	pub unit: String,      // "celsius" | "fahrenheit"
	pub frequency: String, // "0" | "600000" | "1800000" | "3600000" ms
	#[serde(rename = "roundDegree", deserialize_with = "deser_bool_or_str")]
	pub round_degree: bool,
}

impl WeatherSettings {
	pub fn frequency_secs(&self) -> u64 {
		self.frequency
			.parse::<u64>()
			.ok()
			.filter(|&ms| ms > 0)
			.map(|ms| ms / 1000)
			.unwrap_or(600)
	}

	pub fn celsius(&self) -> bool {
		self.unit != "fahrenheit"
	}
}

// Per-instance Notify: signal the poll task to wake early.
static NOTIFIERS: LazyLock<DashMap<InstanceId, Arc<Notify>>> = LazyLock::new(DashMap::new);
// Latest settings for each instance — task reads from here each iteration.
static SETTINGS: LazyLock<DashMap<InstanceId, WeatherSettings>> = LazyLock::new(DashMap::new);

pub async fn force_refresh(instance_id: InstanceId) {
	if let Some(n) = NOTIFIERS.get(&instance_id) {
		n.notify_one();
	}
}

pub struct WeatherAction;

async fn do_refresh(instance_id: &str) {
	let settings = match SETTINGS.get(instance_id) {
		Some(s) => s.clone(),
		None => return,
	};

	let api_key = {
		let g = GLOBAL.read().await;
		g.api_key.clone()
	};

	if api_key.is_empty() || settings.city_name.is_empty() {
		if let Some(inst) = get_instance(instance_id.to_string()).await {
			let _ = inst.show_alert().await;
		}
		return;
	}

	let client = match make_client() {
		Ok(c) => c,
		Err(e) => {
			log::error!("HTTP client error: {e}");
			return;
		}
	};

	match fetch_weather(
		&client,
		&api_key,
		&settings.city_name,
		settings.celsius(),
		settings.round_degree,
	)
	.await
	{
		Ok(result) => {
			let inst = match get_instance(instance_id.to_string()).await {
				Some(i) => i,
				None => return,
			};
			let _ = inst.set_title(Some(result.temperature.clone()), None).await;

			match fetch_icon_data_url(&client, &result.icon_url).await {
				Ok(data_url) => {
					let _ = inst.set_image(Some(data_url), None).await;
				}
				Err(e) => log::warn!("Icon fetch failed for {}: {e}", result.icon_url),
			}
		}
		Err(e) => {
			log::error!("Weather fetch failed: {e}");
			if let Some(inst) = get_instance(instance_id.to_string()).await {
				let _ = inst.show_alert().await;
			}
		}
	}
}

#[async_trait]
impl Action for WeatherAction {
	const UUID: ActionUuid = "com.garrettfaucher.openweather.action";
	type Settings = WeatherSettings;

	async fn will_appear(
		&self,
		instance: &Instance,
		settings: &Self::Settings,
	) -> OpenActionResult<()> {
		let instance_id = instance.instance_id.clone();
		SETTINGS.insert(instance_id.clone(), settings.clone());
		let notify = Arc::new(Notify::new());
		NOTIFIERS.insert(instance_id.clone(), notify.clone());

		tokio::spawn(async move {
			loop {
				do_refresh(&instance_id).await;

				let Some(_inst) = get_instance(instance_id.clone()).await else {
					NOTIFIERS.remove(&instance_id);
					SETTINGS.remove(&instance_id);
					break;
				};

				let secs = SETTINGS
					.get(&instance_id)
					.map(|s| s.frequency_secs())
					.unwrap_or(600);

				tokio::select! {
					_ = tokio::time::sleep(std::time::Duration::from_secs(secs)) => {}
					_ = notify.notified() => {}
				}
			}
		});

		Ok(())
	}

	async fn will_disappear(
		&self,
		instance: &Instance,
		_settings: &Self::Settings,
	) -> OpenActionResult<()> {
		NOTIFIERS.remove(&instance.instance_id);
		SETTINGS.remove(&instance.instance_id);
		Ok(())
	}

	async fn key_down(
		&self,
		instance: &Instance,
		_settings: &Self::Settings,
	) -> OpenActionResult<()> {
		force_refresh(instance.instance_id.clone()).await;
		Ok(())
	}

	async fn did_receive_settings(
		&self,
		instance: &Instance,
		settings: &Self::Settings,
	) -> OpenActionResult<()> {
		// Update the shared settings map so the running task sees new values.
		SETTINGS.insert(instance.instance_id.clone(), settings.clone());
		force_refresh(instance.instance_id.clone()).await;
		Ok(())
	}
}
