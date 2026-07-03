use crate::{decoder::*, packet_type};

packet_type! {
    pub struct RequestAuthenticationCertificate: 0xAA00 {
        #[csm_id(0)]
        pub request_authentication_certificate_serial_number: CsmFlag,
    }
}

packet_type! {
    pub struct AuthenticationCertificate: 0xAA01 {
        #[csm_id(0)]
        pub authentication_certificate: CsmByteArray,
    }
}

packet_type! {
    pub struct RequestAuthenticationChallengeResponse: 0xAA02 {
        #[csm_id(0)]
        pub authentication_challenge: CsmByteArray,
    }
}

packet_type! {
    pub struct AuthenticationResponse: 0xAA03 {
        #[csm_id(0)]
        pub authentication_response: CsmByteArray,
    }
}

packet_type! {
    pub struct AuthenticationFailed: 0xAA04 {}
}

packet_type! {
    pub struct AuthenticationSucceeded: 0xAA05 {}
}

packet_type! {
    pub struct AccessoryAuthenticationSerialNumber: 0xAA06 {
        #[csm_id(0)]
        pub authentication_serial_number: CsmByteArray,
    }
}
