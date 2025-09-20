use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::peripheral,
    wifi::{AccessPointConfiguration, BlockingWifi, Configuration, EspWifi},
};

use crate::global;

pub fn wifi_ap(
    modem: impl peripheral::Peripheral<P = esp_idf_svc::hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
) -> anyhow::Result<Box<EspWifi<'static>>> {
    let mut esp_wifi = EspWifi::new(modem, sysloop.clone(), None)?;
    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sysloop)?;
    let mut wifi_ap_conf = AccessPointConfiguration::default();
    let mut tap_name = wifi_ap_conf.ssid;
    tap_name.clear();
    tap_name.push_str(global::WIFI_AP_NAME).unwrap();
    wifi_ap_conf.ssid = tap_name;
    wifi.set_configuration(&Configuration::AccessPoint(wifi_ap_conf))?;

    log::info!("Initializing wifi ap...");
    wifi.start()?;

    log::info!("Waiting for DHCP lease...");
    wifi.wait_netif_up()?;

    let ap_mac = wifi.wifi().sta_netif().get_mac();
    log::info!("Wifi AP MAC: {:?}", ap_mac);
    let ip_info = wifi.wifi().ap_netif().get_ip_info()?;
    log::info!("Wifi IP info: {:?}", ip_info);

    Ok(Box::new(esp_wifi))
}
