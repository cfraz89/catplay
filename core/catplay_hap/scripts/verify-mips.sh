docker build -f ../../../Dockerfile.mips-cargo-test -t catplay-mips-test .
docker run --rm -it -v "$PWD/../../..":/work catplay-mips-test sh -c "cd core/catplay_hap/scripts; sh do-verify-mips.sh"
