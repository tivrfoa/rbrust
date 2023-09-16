
cargo build --release

docker build . --no-cache -f DockerfileUbuntu -t ubuntu-rust-rinha:4

docker-compose up
