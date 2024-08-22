use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    x: i64,
    y: i64,
    z: i64,
}

impl Position {
    pub fn new(x: i64, y: i64, z: i64) -> Self {
        Self { x, y, z }
    }

    pub fn encode(&self) -> i64 {
        ((self.x & 0x3FFFFFF) << 38) | ((self.z & 0x3FFFFFF) << 12) | (self.y & 0xFFF)
    }

    pub fn decode(val: i64) -> Self {
        let x = val >> 38;
        let y = val << 52 >> 52;
        let z = val << 26 >> 38;
        Self { x, y, z }
    }
    
    pub fn x(&self) -> i64 {
        self.x
    }
    
    pub fn y(&self) -> i64 {
        self.y
    }
    
    pub fn z(&self) -> i64 {
        self.z
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}, {}", self.x, self.y, self.z)
    }
}