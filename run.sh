
cargo build --release

docker build . --no-cache -f DockerfileUbuntu -t ubuntu-rust-rinha:3

docker-compose up
