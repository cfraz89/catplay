use std::time::Duration;

use catplay_iap2_usb::host::GadgetHostHelper;

fn main() {
    let phones = GadgetHostHelper::find_iphones().unwrap();
    let g = phones.first().expect("no phones connected");
    let serial = g.query_serial().unwrap();
    println!("serial: {:?}", serial);

    let data = g.get_capabilities(Duration::from_secs(5)).unwrap();
    println!("cap: {data:?}");

    g.role_switch(true, Duration::from_secs(5)).unwrap();
    println!("role switch OK")
}
