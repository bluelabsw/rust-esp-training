use bsc::{temp_sensor::BoardTempSensor, wifi::wifi};
use embedded_svc::mqtt::client::{
    Client, Details::Complete, Event::Received, Message, Publish, QoS,
};
use esp32_c3_dkc02_bsc as bsc;
use esp_idf_svc::{
    log::EspLogger,
    mqtt::client::{EspMqttClient, EspMqttMessage, MqttClientConfiguration},
};
use std::{thread::sleep, time::Duration};
// If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_idf_sys as _;
use log::{error, info};

#[toml_cfg::toml_config]
pub struct Config {
    #[default("localhost")]
    mqtt_host: &'static str,
    #[default("")]
    mqtt_user: &'static str,
    #[default("")]
    mqtt_pass: &'static str,
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

fn main() -> anyhow::Result<()> {
    EspLogger::initialize_default();

    let app_config = CONFIG;

    let mut temp_sensor = BoardTempSensor::new_taking_peripherals();

    let _wifi = wifi(app_config.wifi_ssid, app_config.wifi_psk)?;

    let broker_url = if app_config.mqtt_user != "" {
        format!(
            "mqtt://{}:{}@{}",
            app_config.mqtt_user, app_config.mqtt_pass, app_config.mqtt_host
        )
    } else {
        format!("mqtt://{}", app_config.mqtt_host)
    };

    let config = MqttClientConfiguration::default();
    let mut client = EspMqttClient::new_with_callback(broker_url, &config, move |message_event| {
        if let Some(Ok(Received(message))) = message_event {
            process_message(message);
        }
    })?;

    client.subscribe("v1/devices/me/attributes", QoS::AtLeastOnce)?;

    loop {
        sleep(Duration::from_secs(3));
        let temp = temp_sensor.read_owning_peripherals();
        let topic = "v1/devices/me/telemetry";
        let payload = format!("{{\"temperature\":{temp:.2}}}");

        client.publish(topic, QoS::ExactlyOnce, false, payload.as_bytes())?;
        info!("Published {topic}: {payload}");
    }
}

fn process_message(message: EspMqttMessage) {
    match message.details() {
        Complete(token) => {
            let topic = message.topic(token);
            let message_data: &[u8] = &message.data();
            if let Ok(payload) = String::from_utf8(message_data.to_vec()) {
                info!("Received {topic}: {payload}");
            }
        }
        _ => error!("Could not process message"),
    }
}
