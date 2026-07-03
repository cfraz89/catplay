export RUSTFLAGS='-C target-feature=+crt-static
  -C link-arg=-Wl,--no-as-needed
  -C link-arg=-Wl,-Bstatic
  -C link-arg=-latomic
  -C link-arg=-Wl,-Bdynamic'
cargo bench --bench cipher --no-run
cargo bench --bench pair_setup --no-run
