//! A simple demo to showcase how player could send inputs to move the square and server replicates position back.
//! Also demonstrates the single-player and how sever also could be a player.

use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::SystemTime;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::{
    renet::{
        transport::{
            ClientAuthentication, NetcodeClientTransport,
        },
        ConnectionConfig, RenetClient,
    },
    RenetChannelsExt,
};
use mmo_game_shared::components::*;

pub(crate) struct MmoGameClientPlugin;

impl Plugin for MmoGameClientPlugin {
    fn build(&self, app: &mut App) {
        app.replicate::<PlayerPosition>()
            .replicate::<PlayerColor>()
            .add_client_event::<MoveDirection>(ChannelKind::Ordered)
            .add_systems(Startup, (Self::connect, Self::spawn_camera))
            .add_systems(
                Update,
                (
                    Self::apply_movement.run_if(has_authority), // Runs only on the server or a single player.
                    Self::handle_connections.run_if(server_running), // Runs only on the server.
                    (Self::draw_boxes, Self::read_input),
                ),
            );
    }
}

impl MmoGameClientPlugin {
    fn connect(mut commands: Commands, channels: Res<RepliconChannels>,) {
        let server_channels_config = channels.get_server_configs();
        let client_channels_config = channels.get_client_configs();

        let client = RenetClient::new(ConnectionConfig {
            server_channels_config,
            client_channels_config,
            ..Default::default()
        });

        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("s");
        let client_id = current_time.as_millis() as u64;
        let server_addr = SocketAddr::new(IpAddr::from(Ipv4Addr::new(127, 0, 0, 1)), 5000);
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).expect(" ");
        let authentication = ClientAuthentication::Unsecure {
            client_id,
            protocol_id: PROTOCOL_ID,
            server_addr,
            user_data: None,
        };
        let transport = NetcodeClientTransport::new(current_time, authentication, socket).expect("");

        commands.insert_resource(client);
        commands.insert_resource(transport);

        commands.spawn(TextBundle::from_section(
            format!("Client: {client_id:?}"),
            TextStyle {
                font_size: 30.0,
                color: Color::WHITE,
                ..default()
            },
        ));
    }

    fn spawn_camera(mut commands: Commands) {
        commands.spawn(Camera2dBundle::default());
    }

    /// Logs server events and spawns a new player whenever a client connects.
    fn handle_connections(mut commands: Commands, mut server_events: EventReader<ServerEvent>) {
        for event in server_events.read() {
            match event {
                ServerEvent::ClientConnected { client_id } => {
                    info!("{client_id:?} connected");
                    // Generate pseudo random color from client id.
                    let r = ((client_id.get() % 23) as f32) / 23.0;
                    let g = ((client_id.get() % 27) as f32) / 27.0;
                    let b = ((client_id.get() % 39) as f32) / 39.0;
                    commands.spawn(PlayerBundle::new(
                        *client_id,
                        Vec2::ZERO,
                        Color::srgb(r, g, b),
                    ));

                }
                ServerEvent::ClientDisconnected { client_id, reason } => {
                    info!("{client_id:?} disconnected: {reason}");
                }
            }
        }
    }

    fn draw_boxes(mut gizmos: Gizmos, players: Query<(&PlayerPosition, &PlayerColor)>) {
        for (position, color) in &players {
            gizmos.rect(
                Vec3::new(position.x, position.y, 0.0),
                Quat::IDENTITY,
                Vec2::ONE * 50.0,
                color.color,
            );
        }
    }

    /// Reads player inputs and sends [`MoveDirection`] events.
    fn read_input(mut move_events: EventWriter<MoveDirection>, input: Res<ButtonInput<KeyCode>>) {
        let mut direction = Vec2::ZERO;
        if input.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }
        if input.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }
        if input.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
        }
        if input.pressed(KeyCode::ArrowDown) {
            direction.y -= 1.0;
        }
        if direction != Vec2::ZERO {
            move_events.send(MoveDirection{direction:direction.normalize_or_zero()});
        }
    }

    /// Mutates [`PlayerPosition`] based on [`MoveDirection`] events.
    ///
    /// Fast-paced games usually you don't want to wait until server send a position back because of the latency.
    /// But this example just demonstrates simple replication concept.
    fn apply_movement(
        time: Res<Time>,
        mut move_events: EventReader<FromClient<MoveDirection>>,
        mut players: Query<(&Player, &mut PlayerPosition)>,
    ) {
        const MOVE_SPEED: f32 = 200.0;
        for FromClient { client_id, event } in move_events.read() {
            info!("received event {event:?} from {client_id:?}");
            for (player, mut position) in &mut players {
                if *client_id == player.client_id {
                    **position += event.direction * time.delta_seconds() * MOVE_SPEED;
                }
            }
        }
    }
}

//const PORT: u16 = 5000;
const PROTOCOL_ID: u64 = 0;

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    position: PlayerPosition,
    color: PlayerColor,
    replicated: Replicated,
}

impl PlayerBundle {
    fn new(client_id: ClientId, position: Vec2, color: Color) -> Self {
        Self {
            player: Player{client_id},
            position: PlayerPosition{position},
            color: PlayerColor{color},
            replicated: Replicated,
        }
    }
}
