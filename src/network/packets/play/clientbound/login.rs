use crate::{custom_types::{identifier::Identifier, position::Position}, network::packet::ClientboundPacket};

pub struct PlayClientboundLogin {
    pub entity_id: i32,
    pub is_hardcode: bool,
    pub dimention_names: Vec<Identifier>,
    pub max_players: i32, // ignored by client
    pub view_distance: i32,
    pub simulation_distance: i32,
    pub reduced_debug_info: bool,
    pub enable_respawn_screen: bool,
    pub do_limited_crafting: bool,
    pub dimention_type: i32,
    pub dimention_name: Identifier,
    pub hashed_seed: i64,
    pub gamemode: u8, // TODO: should be an enum
    pub previous_gamemode: i8, // TODO: should be an optional enum
    pub is_debug: bool,
    pub is_flat: bool,
    pub death_location: Option<(Identifier, Position)>,
    pub portal_cooldown: i32,
    pub enforces_secure_chat: bool,
}

impl ClientboundPacket for PlayClientboundLogin {
    fn packet_id() -> i32 {
        0x2B
    }

    fn build(&self) -> Vec<u8> {
        todo!()
    }
}