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

pub mod login {
    pub mod clientbound {
        pub mod disconnect;
        pub mod encryption_request;
        pub mod login_success;
    }
    pub mod serverbound {
        pub mod encryption_response;
        pub mod login_acknowledged;
        pub mod login_start;
    }
}