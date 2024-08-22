pub struct DisplayedSkinParts {
    pub cape_enabled: bool,
    pub jacket_enabled: bool,
    pub left_sleeve_enabled: bool,
    pub right_sleeve_enabled: bool,
    pub left_pants_enabled: bool,
    pub right_pants_enabled: bool,
    pub hat_enabled: bool,
}

impl DisplayedSkinParts {
    pub fn from_bitmask(byte: u8) -> Self {
        Self {
            cape_enabled: (0x01 & byte) == byte,
            jacket_enabled: (0x02 & byte) == byte,
            left_sleeve_enabled: (0x04 & byte) == byte,
            right_sleeve_enabled: (0x08 & byte) == byte,
            left_pants_enabled: (0x10 & byte) == byte,
            right_pants_enabled: (0x20 & byte) == byte,
            hat_enabled: (0x40 & byte) == byte,
        }
    }
}