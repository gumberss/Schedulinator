
### Libraries Added
cargo add warp
cargo add serde --features derive
cargo add chrono --features serde
cargo add tokio --features full
cargo add pretty_env_logger
cargo add uuid --features v4
cargo install cargo-watch

### Watch Command
cargo watch -q -c -w src/ -x run


### Clippy
cargo clippy 
cargo clippy --fix
Reference: https://github.com/rust-lang/rust-clippy


### Docker
docker build -t schedulinator .
docker tag schedulinator gumberss/schedulinator:latest
docker push gumberss/schedulinator:latest