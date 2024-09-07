# RustCraft - Rust Minecraft Server Software

> [!NOTE]
> This is an **experimental** project. I don't know any Rust and this is how I'm learning it. This project is **not** under heavy development and is just me wanting to build something useful in my free time.

I don't yet have a clear direction with this project, however I would love to make it something that would have it's benefits like speed, low resource usage, flexibility, blablabla.

Meanwhile you should check out [Dockyard](https://github.com/DockyardMC/Dockyard) by [LukynkaCZE](https://github.com/LukynkaCZE)!

## Progress
The project is currently no use for **any** environment. 
- [v] Handshaking
- [v] Status *(Server list)*
- [v] Login *(Encryption & online mode)*
- [ ] Configuration *(Working on this rn...)*
- [ ] Actually join a world
- [ ] Proper packet handling
- [ ] Plugins *(Will probably be .dll & .so plugins made with Rust)*

## Build
This command should build the server for your architecture.
```sh
cargo build --release
```

## Developing
### Run
```sh
cargo run
```

### Linting
I use clippy for linting.
```sh
cargo clippy
```

### Tests
Got random unit tests in the project, but I don't yet aim to cover the entire project with unit tests.
```sh
cargo test
```