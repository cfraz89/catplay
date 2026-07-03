use std::error::Error;
use std::time::Duration;

use catplay_bt::BluezManager;

use catplay_tracing::logger::setup_test_logger;
use log::info;
use tokio::time::sleep;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_test_logger(false);
    let mut mgr = BluezManager::new();

    mgr.register_pin_agent().await?;

    info!("Agent registered. Waiting for events...");
    mgr.set_powered("hci0", true).await?;
    mgr.set_pairable("hci0", true).await?;
    mgr.set_discoverable("hci0", true).await?;
    mgr.set_alias("hci0", "AutoKit_A77").await?;
    info!("Agent ready");

    loop {
        let req = mgr.get_pairing_request();
        if let Some(req) = req {
            println!("{req:?}");
            req.accept();
        }

        sleep(Duration::from_millis(100)).await;
    }
}
