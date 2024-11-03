use std::time::Duration;

use app::{Apps, Cli};
use client::OverheatClientPlugin;
use server::OverheatServerPlugin;
use settings::{read_settings, Settings};
use shared::OverheatSharedPlugin;

mod settings;
mod app;
mod client;
mod server;
mod protocol;
mod shared;
mod player;
mod physics;
mod assets;
mod rendering;
mod animation;
mod ability_framework;

pub const FIXED_TIMESTEP_HZ: f64 = 64.;
pub const REPLICATION_INTERVAL: Duration = Duration::from_millis(100);

fn main() {
    let cli = Cli::default();
    let settings_str = include_str!("../assets/settings.ron");
    let settings = read_settings::<Settings>(settings_str);

    let mut apps = Apps::new(&settings, cli)
        .with_server_replication_send_interval(Duration::from_millis(settings.server_replication_send_interval));

    apps
    .update_lightyear_client_config(|config| {
        config.prediction.maximum_input_delay_before_prediction = settings.input_delay_ticks;
        config.prediction.correction_ticks_factor = settings.correction_ticks_factor;
    })
    .add_lightyear_plugins()
    .add_plugins(
        OverheatClientPlugin,
        OverheatServerPlugin {
            predict_all: settings.predict_all,
        },
        OverheatSharedPlugin
    );

    apps.run();
}

