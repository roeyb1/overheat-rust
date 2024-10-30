use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::{app::App, log::{Level, LogPlugin}, utils::default, DefaultPlugins};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use clap::Parser;
use lightyear::client::config::ClientConfig;
use lightyear::prelude::*;
use lightyear::prelude::{client, server};
use lightyear::server::config::ServerConfig;

use crate::settings::{build_server_netcode_config, get_client_net_config, get_server_net_configs, Settings};
use crate::{FIXED_TIMESTEP_HZ, REPLICATION_INTERVAL};

#[derive(Parser, PartialEq, Debug)]
pub enum Cli {
    /// Client and server run in the same application. Server is also a client
    HostServer {
        #[arg(short, long, default_value = None)]
        client_id: Option<u64>,
    },
    /// Dedicated server
    Server,
    /// Regular client
    Client {
        #[arg(short, long, default_value = None)]
        client_id: Option<u64>
    }
}

fn cli() -> Cli {
    Cli::parse()
}

impl Default for Cli { 
    fn default() -> Self {
        cli()
    }
}

pub enum Apps {
    Client { app: App, config: ClientConfig },
    Server { app: App, config: ServerConfig },
    HostServer {
        app: App,
        client_config: ClientConfig,
        server_config: ServerConfig,
    }
}

impl Apps {
    pub fn new(settings: &Settings, cli: Cli) -> Self {
        match cli {
            Cli::HostServer { client_id } => {
                let client_net_config = client::NetConfig::Local {
                    id: client_id.unwrap_or(settings.client.client_id),
                };
                let (app, client_config, server_config) = combined_app(settings, vec![], client_net_config);
                Apps::HostServer {
                    app,
                    client_config,
                    server_config
                }
            },
            Cli::Server => {
                let (app, config) = server_app(settings, vec![]);
                Apps::Server { app, config }
            },
            Cli::Client { client_id } => {
                let client_id = client_id.unwrap_or(settings.client.client_id);
                let net_config = get_client_net_config(&settings, client_id);
                let (app, config) = client_app(settings, net_config);
                Apps::Client { app, config }
            },
        }
    }

    pub fn with_server_replication_send_interval(mut self, replication_interval: Duration) -> Self {
        self.update_lightyear_client_config(|cc: &mut ClientConfig| {
            cc.shared.server_replication_send_interval = replication_interval
        });
        self.update_lightyear_server_config(|sc: &mut ServerConfig| {
            sc.shared.server_replication_send_interval = replication_interval
        });
        self
    }

    pub fn add_lightyear_plugins(&mut self) -> &mut Self {
        match self {
            Apps::Client { app, config } => {
                app.add_plugins(client::ClientPlugins {
                    config: config.clone(),
                });
            },
            Apps::Server { app, config } => {
                app.add_plugins(server::ServerPlugins {
                    config: config.clone(),
                });
            }
            Apps::HostServer {
                app,
                client_config,
                server_config
            } => {
                app.add_plugins(client::ClientPlugins {
                    config: client_config.clone(),
                });
                app.add_plugins(server::ServerPlugins {
                    config: server_config.clone(),
                });
            },
        }
        self
    }

    pub fn add_plugins(
        &mut self,
        client_plugin: impl Plugin,
        server_plugin: impl Plugin,
        shared_plugin: impl Plugin,
    ) -> &mut Self {
        match self {
            Apps::Client { app, .. } => {
                app.add_plugins((client_plugin, shared_plugin));
            },
            Apps::Server { app, .. } => {
                app.add_plugins((server_plugin, shared_plugin));
            }
            Apps::HostServer { app, .. } => {
                app.add_plugins((client_plugin, server_plugin, shared_plugin));
            },
        }
        self
    }

    pub fn update_lightyear_client_config(
        &mut self,
        f: impl FnOnce(&mut ClientConfig),
    ) -> &mut Self {
        match self {
            Apps::Client { config, .. } => f(config),
            Apps::Server { .. } => {},
            Apps::HostServer { client_config, .. } => f(client_config),
        }
        self
    }

    pub fn update_lightyear_server_config(
        &mut self,
        f: impl FnOnce(&mut ServerConfig),
    ) -> &mut Self {
        match self {
            Apps::Client { .. } => {},
            Apps::Server { config, ..} => f(config),
            Apps::HostServer { server_config, .. } => f(server_config),
        }
        self
    }

    pub fn run(self) {
        match self {
            Apps::Client { mut app, .. } => app.run(),
            Apps::Server { mut app, .. } => app.run(),
            Apps::HostServer { mut app, .. } => app.run(),
        };
    }
}

fn combined_app(
    settings: &Settings,
    extra_transport_configs: Vec<server::ServerTransport>,
    client_net_config: client::NetConfig,
) -> (App, ClientConfig, ServerConfig) {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.build().set(LogPlugin {
        level: Level::INFO,
        filter: "wgpu=error,bevy_render=info,bevy_ecs=warn".to_string(),
        ..default()
    }));
    if settings.client.inspector {
        app.add_plugins(WorldInspectorPlugin::new());
    }

    let mut net_configs = get_server_net_configs(&settings);
    let extra_net_configs = extra_transport_configs.into_iter().map(|c| {
        build_server_netcode_config(settings.server.conditioner.as_ref(), &settings.shared, c)
    });

    net_configs.extend(extra_net_configs);

    let server_config = ServerConfig {
        shared: shared_config(Mode::HostServer),
        net: net_configs,
        replication: ReplicationConfig {
            send_interval: REPLICATION_INTERVAL,
            ..default()
        },
        ..default()
    };

    let client_config = ClientConfig {
        shared: shared_config(Mode::HostServer),
        net: client_net_config,
        ..default()
    };

    (app, client_config, server_config)
}

fn server_app(
    settings: &Settings,
    extra_transport_configs: Vec<server::ServerTransport>
) -> (App, ServerConfig) {
    let mut app = App::new();
    if !settings.server.headless {
        app.add_plugins(DefaultPlugins.build().disable::<LogPlugin>());
    } else {
        app.add_plugins((MinimalPlugins, StatesPlugin));
    }

    app.add_plugins(LogPlugin {
        level: Level::INFO,
        filter: "wgpu=error,bevy_render=info,bevy_ecs=warn".to_string(),
        ..default()
    });

    if settings.server.inspector {
        app.add_plugins(WorldInspectorPlugin::new());
    }

    let mut net_configs = get_server_net_configs(&settings);
    let extra_net_configs = extra_transport_configs.into_iter().map(|c| {
        build_server_netcode_config(settings.server.conditioner.as_ref(), &settings.shared, c)
    });
    net_configs.extend(extra_net_configs);
    let server_config = ServerConfig {
        shared: shared_config(Mode::Separate),
        net: net_configs,
        replication: ReplicationConfig {
            send_interval: REPLICATION_INTERVAL,
            ..default()
        },
        ..default()
    };

    (app, server_config)
}

fn client_app(
    settings: &Settings,
    net_config: client::NetConfig,
) -> (App, ClientConfig) {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .build()
            .set(LogPlugin {
                level: Level::INFO,
                filter: "wgpu=error,bevy_render=info,bevy_ecs=warn".into(),
                ..default()
            }),
    );

    if settings.client.inspector {
        app.add_plugins(WorldInspectorPlugin::new());
    }
    let client_config = ClientConfig {
        shared: shared_config(Mode::Separate),
        net: net_config,
        replication: ReplicationConfig {
            send_interval: REPLICATION_INTERVAL,
            ..default()
        },
        ..default()
    };
    (app, client_config)
}

fn shared_config(mode: Mode) -> SharedConfig {
    SharedConfig {
        server_replication_send_interval: REPLICATION_INTERVAL,
        tick: TickConfig {
            tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
        },
        mode,
    }
}