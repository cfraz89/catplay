A minimal HomeKit implementation for use within CarPlay pairing.

Based on hap-rs but with:

- modernized and slimmed down crypto dependencies (SRP upgrade to 0.6, Ring as base crypto library)
- added support for /auth-setup (MFI-SAP)
- a new abstraction layer that can be hooked into CarPlay RTSP protocol while minimizing the complexity.
- fixed all `panic!` issues when parsing invalid/malicious data
- role reversal support (supports iPhone "Controller" role in addition to the Accessory role)
- removed `std` dependency
- replaced SRP with OpenSSL for fastest PairSetup implementation
- replaced Ring with fast_chacha(Cryptogams/OpenSSL ASM integration) for fastest HomeKit ChaChaPoly1305 cipher
(except it was missing the Poly1305, which here was added from scratch)