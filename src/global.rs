use openaction::{
	OpenActionResult,
	global_events::{DidReceiveGlobalSettingsEvent, GlobalEventHandler, set_global_event_handler},
	visible_instances,
};

use std::sync::LazyLock;
use tokio::sync::RwLock;

#[derive(Debug, Default, Clone)]
pub struct GlobalSettings {
	pub api_key: String,
}

pub static GLOBAL: LazyLock<RwLock<GlobalSettings>> = LazyLock::new(Default::default);

struct Handler;

#[async_trait::async_trait]
impl GlobalEventHandler for Handler {
	async fn plugin_ready(&self) -> OpenActionResult<()> {
		openaction::get_global_settings().await
	}

	async fn did_receive_global_settings(
		&self,
		event: DidReceiveGlobalSettingsEvent,
	) -> OpenActionResult<()> {
		let settings = &event.payload.settings;
		let api_key = settings["apiKey"].as_str().unwrap_or("").to_string();

		{
			let mut g = GLOBAL.write().await;
			g.api_key = api_key;
		}

		// Trigger a refresh on all visible instances once the key is available.
		let instances = visible_instances("com.garrettfaucher.openweather.action").await;
		for inst in instances {
			let iid = inst.instance_id.clone();
			tokio::spawn(async move {
				crate::weather::force_refresh(iid).await;
			});
		}

		Ok(())
	}
}

pub fn register() {
	static HANDLER: Handler = Handler;
	set_global_event_handler(&HANDLER);
}
