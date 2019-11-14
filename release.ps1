rm docs/*
cargo web deploy --release
cp target/deploy/* docs/
