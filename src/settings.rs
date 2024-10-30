use std::{net::{Ipv4Addr, SocketAddr}, time::Duration};

use bevy::{asset::ron, utils::default};
use lightyear::prelude::{client::{self, Authentication, SocketConfig, SteamConfig}, server, CompressionConfig, LinkConditionerConfig};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub fn read_settings<T: DeserializeOwned>(settings_str: &str) -> T {
    ron::de::from_str::<T>(settings_str).expect("Error deserializing the settings file")
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub client: ClientSettings,
    pub shared: SharedSettings,

    pub predict_all: bool,
    pub input_delay_ticks: u16,
    pub correction_ticks_factor: f32,
    pub server_replication_send_interval: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerSettings {
    pub headless: bool,
    pub inspector: bool,
    pub conditioner: Option<Conditioner>,
    pub transports: Vec<ServerTransports>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Conditioner {
    latency_ms: u16,
    jitter_ms: u16,
    packet_loss: f32,
}

impl Conditioner {
    fn build(&self) -> LinkConditionerConfig {
        LinkConditionerConfig {
            incoming_latency: Duration::from_millis(self.latency_ms as u64),
            incoming_jitter: Duration::from_millis(self.jitter_ms as u64),
            incoming_loss: self.packet_loss,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ServerTransports {
    Udp {
        local_port: u16,
    },
    Steam {
        app_id: u32,
        server_ip: Ipv4Addr,
        game_port: u16,
        query_port: u16,
    }
}


#[derive(Debug, Deserialize, Serialize)]
pub struct ClientSettings {
    pub inspector: bool,
    pub client_id: u64,
    pub client_port: u16,
    pub server_addr: Ipv4Addr,
    pub server_port: u16,
    pub transport: ClientTransports,
    pub conditioner: Option<Conditioner>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ClientTransports {
    Udp,
    Steam {
        app_id: u32,
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct SharedSettings {
    protocol_id: u64,
    private_key: [u8; 32],
    compression: CompressionConfig,
}

pub(crate) fn build_server_netcode_config(
    conditioner: Option<&Conditioner>,
    shared: &SharedSettings,
    transform_config: server::ServerTransport,
) -> server::NetConfig {
    let conditioner = conditioner.map_or(None, |c| {
        Some(c.build())
    });

    let netcode_config = server::NetcodeConfig::default()
        .with_protocol_id(shared.protocol_id)
        .with_key(shared.private_key);

    let io_config = server::IoConfig {
        transport: transform_config,
        conditioner,
        compression: shared.compression
    };

    server::NetConfig::Netcode {
        config: netcode_config,
        io: io_config
    }
}

pub(crate) fn get_server_net_configs(settings: &Settings) -> Vec<server::NetConfig> {
    settings
        .server
        .transports
        .iter()
        .map(|t| match t {
            ServerTransports::Udp {
                local_port
            } => build_server_netcode_config(
                    settings.server.conditioner.as_ref(),
                    &settings.shared,
                    server::ServerTransport::UdpSocket(SocketAddr::new(
                        Ipv4Addr::UNSPECIFIED.into(),
                        *local_port,
                    ))
                ),
            ServerTransports::Steam {
                app_id,
                server_ip,
                game_port,
                query_port
            } => server::NetConfig::Steam {
                steamworks_client: None,
                config: server::SteamConfig {
                    app_id: *app_id,
                    socket_config: server::SocketConfig::Ip {
                        server_ip: *server_ip,
                        game_port: *game_port,
                        query_port: *query_port,
                    },
                    max_clients: 16,
                    ..default()
                },
                conditioner: settings
                    .server
                    .conditioner
                    .as_ref()
                    .map_or(None, |c| Some(c.build())),
            }
        }).collect()
}

pub(crate) fn build_client_netcode_config(
    client_id: u64,
    server_addr: SocketAddr,
    conditioner: Option<&Conditioner>,
    shared: &SharedSettings,
    transform_config: client::ClientTransport,
) -> client::NetConfig {
    let conditioner = conditioner.map_or(None, |c| Some(c.build()));
    let auth = Authentication::Manual {
        server_addr: server_addr,
        client_id: client_id,
        private_key: shared.private_key,
        protocol_id: shared.protocol_id
    };
    let netcode_config = client::NetcodeConfig::default();
    let io_config = client::IoConfig {
        transport: transform_config,
        conditioner,
        compression: shared.compression,
    };
    client::NetConfig::Netcode { auth: auth,
        config: netcode_config,
        io: io_config
    }
}

pub(crate) fn get_client_net_config(settings: &Settings, client_id: u64) -> client::NetConfig {
    let server_addr = SocketAddr::new(
        settings.client.server_addr.into(),
        settings.client.server_port,
    );
    let client_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), settings.client.client_port);
    match &settings.client.transport {
        ClientTransports::Udp => build_client_netcode_config(
            client_id,
            server_addr,
            settings.client.conditioner.as_ref(),
            &settings.shared,
            client::ClientTransport::UdpSocket(client_addr)
        ),
        ClientTransports::Steam { app_id } => client::NetConfig::Steam {
            steamworks_client: None,
            config: SteamConfig {
                socket_config: SocketConfig::Ip { server_addr },
                app_id: *app_id,
            },
            conditioner: settings
                .server
                .conditioner
                .as_ref()
                .map_or(None, |c| Some(c.build()))
        },
    }
}