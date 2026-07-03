use std::{env, sync::Arc};

use catplay_mfi::{
    MfiDevice, MfiDeviceI2C, MfiResult,
    server::{MfiDeviceRemoteClient, MfiDeviceServer},
};

fn main() -> MfiResult<()> {
    let args: Vec<String> = env::args().collect();
    let bus_offset = 0;
    let dev_addr = 0x10;

    if args[1] == "test" {
        let mfi_device = MfiDeviceI2C::new(bus_offset, dev_addr).unwrap();

        let cert: Vec<u8> = mfi_device.read_certificate()?;
        println!("CERT: {:x?}", cert);

        let challenge = b"12211213131231231231";
        let response = mfi_device.generate_challenge_response(challenge)?;
        println!("RESPONSE: {:x?}", response);

        return Ok(());
    } else if args[1] == "server" {
        let mfi_device = MfiDeviceI2C::new(bus_offset, dev_addr).unwrap();
        let mut server: MfiDeviceServer = MfiDeviceServer::new(args[2].clone(), Arc::new(mfi_device));
        server.listen().map_err(|e| e.to_string())?;

        return Ok(());
    } else if args[1] == "client" {
        let client = MfiDeviceRemoteClient::new(args[2].clone()).map_err(|e| e.to_string())?;

        // Example 1: Read certificate
        let response = client.read_certificate();
        println!("Certificate: {:x?}", response);

        // Example 2: Generate challenge response
        let challenge = vec![0xAA, 0xBB, 0xCC];
        let response = client.generate_challenge_response(&challenge)?;
        println!("Challenge Response: {:x?}", response);

        return Ok(());
    }

    println!("./catplay_mfi test|server|client <0.0.0.0:9000>");
    Ok(())
}
