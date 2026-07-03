use std::{str::FromStr, time::Duration};

use catplay_iap2_bt::BluetoothManager;
use catplay_iap2_client::CsmSessionCallbacks;
use catplay_tracing::logger::setup_test_logger;
use macaddr::MacAddr6;
use tokio::time::sleep;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    setup_test_logger(false);

    let mut srv = BluetoothManager::new(false, true, true, "hci0", || {
        CsmSessionCallbacks::new(async |_a| Ok(()), async |_a, _b| Ok(()))
    });
    srv.start().await.unwrap();

    BluetoothManager::invite_iphone("hci0", &MacAddr6::from_str("80:B9:83:71:0D:63").unwrap())
        .await
        .unwrap();

    sleep(Duration::from_secs(6000)).await;
}
