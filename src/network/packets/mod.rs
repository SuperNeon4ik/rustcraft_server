pub mod handshaking {
    pub mod serverbound {
        pub mod handshake;
    }
}

pub mod status {
    pub mod clientbound {
        pub mod ping_response;
        pub mod status_response;
    }
    pub mod serverbound {
        pub mod ping_request;
        pub mod status_request;
    }
}