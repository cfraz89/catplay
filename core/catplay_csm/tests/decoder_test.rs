#[cfg(test)]
mod tests {
    use catplay_csm::{
        decoder::*,
        msg::{
            AccessoryWiFiConfigurationInformation, AuthenticationCertificate, BluetoothTransportComponent, EngineTypes,
            IdentificationInformation, IdentificationRejected, PlaybackQueueListContentTransferInfoRequest,
            RequestAuthenticationChallengeResponse, SetNowPlayingInformation, StartNowPlayingMediaItemAttributes,
            StartNowPlayingPlaybackAttributes, StartNowPlayingUpdates, StopNowPlayingUpdates, VehicleInformationComponent,
        },
    };
    use insta::assert_debug_snapshot;

    #[derive(Debug, PartialEq, Eq)]
    pub struct TestSnapshot {
        packet: String,
        bytes: Vec<u8>,
    }

    #[test]
    fn test_auth_cert() {
        let reg = CsmPacketRegistry::static_registry();
        let auth = AuthenticationCertificate {
            authentication_certificate: CsmByteArray::new(vec![1, 2, 3, 4, 5]),
        };
        let bytes = reg.encode(&auth).unwrap();
        assert_eq!(
            bytes,
            [
                0x40, 0x40, 0x00, 0x0F, 0xAA, 0x01, 0x00, 0x09, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05
            ]
        );
    }

    #[test]
    fn test_risky_packets() {
        let mut snap = Vec::new();

        pub fn assert_packet<T: CsmPacket + PartialEq + AsCsmPacket>(snap: &mut Vec<TestSnapshot>, packet: &T) {
            let registry = CsmPacketRegistry::static_registry();
            let encoded = registry.encode(packet).expect("Failed to encode packet");
            let decoded_box = registry.decode(&encoded).expect("Failed to decode packet");
            let decoded = T::cast(&decoded_box).expect("Failed to cast");

            assert_eq!(*decoded, *packet, "Re-serialization yielded different bytes at {packet:?}");
            snap.push(TestSnapshot {
                packet: format!("{decoded:?}"),
                bytes: encoded,
            });
        }

        assert_packet(
            &mut snap,
            &IdentificationInformation {
                name: "Name".into(),
                model_identifier: "Model".into(),
                manufacturer: "Manufacturer".into(),
                serial_number: "Serial".into(),
                firmware_version: "FirmwareVersion".into(),
                messages_sent_by_accessory: IdentificationInformation::pack_ids(&[
                    AccessoryWiFiConfigurationInformation::PACKET_ID,
                    StartNowPlayingUpdates::PACKET_ID,
                    StopNowPlayingUpdates::PACKET_ID,
                    SetNowPlayingInformation::PACKET_ID,
                ]),
                vehicle_information_component: Some(VehicleInformationComponent {
                    identifier: 1,
                    name: "Auto Box".into(),
                    display_name: "DisplayName".into(),
                    engine_type: EngineTypes::Gasoline,
                }),
                supported_language: vec!["en".into()],
                bluetooth_transport_component: vec![BluetoothTransportComponent {
                    transport_component_identifier: 1,
                    transport_component_name: "Blue".into(),
                    transport_supports_iap2_connection: CsmFlag::Yes,
                    bluetooth_transport_mac_address: vec![1, 2, 3, 4, 5, 6].into(),
                }],
                ..IdentificationInformation::default()
            },
        );

        assert_packet(
            &mut snap,
            &StartNowPlayingUpdates {
                attributes: Some(StartNowPlayingMediaItemAttributes::all()),
                playback_attributes: Some(StartNowPlayingPlaybackAttributes::all()),
                playback_queue_list_content_transfer_info_request: Some(PlaybackQueueListContentTransferInfoRequest::all()),
            },
        );

        assert_packet(
            &mut snap,
            &IdentificationRejected {
                name: CsmFlag::Yes,
                ..IdentificationRejected::default()
            },
        );

        assert_packet(
            &mut snap,
            &RequestAuthenticationChallengeResponse {
                authentication_challenge: vec![0x42u8; 20].into(),
            },
        );

        assert_debug_snapshot!(snap);
    }
}
