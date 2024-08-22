use crate::{custom_types::bitmasks::DisplayedSkinParts, network::packet::{PacketReader, ServerboundPacket}, utils::errors::PacketReadError};

pub struct ConfigurationServerboundClientInformation {
    pub locale: String,
    pub view_distance: i16,
    pub chat_mode: ClientChatMode,
    pub chat_colors: bool,
    pub displayed_skin_parts: DisplayedSkinParts,
    pub main_hand: MainHand,
    pub enable_text_filtering: bool,
    pub allow_server_listings: bool,
}

pub enum ClientChatMode {
    Enabled,
    CommandsOnly,
    Hidden,
}

pub enum MainHand {
    Left,
    Right,
}

impl ServerboundPacket for ConfigurationServerboundClientInformation {
    fn packet_id() -> i32 
    where 
        Self: Sized {
        0x00
    }

    fn read(reader: &mut PacketReader) -> Result<Self, PacketReadError>
    where 
        Self: Sized {
            let locale = reader.read_string()?;
            let view_distance = reader.read_byte()? as i16;
            let chat_mode = match reader.read_varint()? {
                0 => ClientChatMode::Enabled,
                1 => ClientChatMode::CommandsOnly,
                2 => ClientChatMode::Hidden,
                _ => return Err(PacketReadError::UnexpectedValue),
            };
            let chat_colors = reader.read_boolean()?;
            let displayed_skin_parts_bitmask = reader.read_ubyte()?;
            let displayed_skin_parts = DisplayedSkinParts::from_bitmask(displayed_skin_parts_bitmask);
            let main_hand = match reader.read_varint()? {
                0 => MainHand::Left,
                1 => MainHand::Right,
                _ => return Err(PacketReadError::UnexpectedValue),
            };
            let enable_text_filtering = reader.read_boolean()?;
            let allow_server_listings = reader.read_boolean()?;

            Ok(Self {
                locale,
                view_distance,
                chat_mode,
                chat_colors,
                displayed_skin_parts,
                main_hand,
                enable_text_filtering,
                allow_server_listings,
            })
    }
}