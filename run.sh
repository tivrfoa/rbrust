
cargo build --release

docker build . --no-cache -f DockerfileUbuntu -t ubuntu-rust-rinha:batchinsert

docker-compose up
