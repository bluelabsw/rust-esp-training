use std::sync::Arc;

use embedded_svc::wifi::Wifi;
use esp_idf_svc::{
    netif::EspNetifStack, nvs::EspDefaultNvs, sysloop::EspSysLoopStack, wifi::EspWifi,
};

use esp_idf_sys as _;

fn main() -> anyhow::Result<()> {
    println!("Scanning for WiFi access points...");

    let netif_stack = Arc::new(EspNetifStack::new()?);
    let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
    let nvs = Arc::new(EspDefaultNvs::new()?);
    let mut wifi = EspWifi::new(netif_stack, sys_loop_stack, nvs)?;

    loop {
        let ap_infos = wifi.scan()?;

        println!("=====");
        for ap_info in ap_infos {
            let ssid = ap_info.ssid;
            let signal_strength = (ap_info.signal_strength as f32 / 2.55) as u8;
            println!("- Discovered '{ssid}' ({signal_strength}%)");
        }
        println!("=====");
    }
}
