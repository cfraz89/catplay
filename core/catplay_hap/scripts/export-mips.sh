# NOTE: don't use, broken MUSL toolchain
docker build -f ../../../Dockerfile.mips-musl-cargo-test -t catplay-mips-test2 .
docker run --rm -it -v "$PWD/../../..":/work catplay-mips-test2 sh -c "cd core/catplay_hap/scripts/; sh scripts/do-export-mips.sh"
