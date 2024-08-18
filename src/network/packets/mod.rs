pub mod configuration {
    pub mod clientbound {
        pub mod disconnect;
        pub mod feature_flags;
        pub mod finish_configuration;
        pub mod keep_alive;
        pub mod ping;
    }
    pub mod serverbound {
        pub mod acknowledge_finish_configuration;
        pub mod client_information;
        pub mod keep_alive;
        pub mod pong;
    }
}

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