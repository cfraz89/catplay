for i in $(echo is_using_asm rfc7539 compare); do cargo test --test fast_chacha20_$i; done
for i in $(echo poly1305); do cargo test --test fast_poly1305_$i; done
cargo bench
