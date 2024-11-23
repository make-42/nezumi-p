use serde::Deserialize;

use crate::config::Config;

// Departure Data Types
#[derive(Deserialize, Debug, Default)]
pub struct Value {
    #[serde(rename = "value")]
    pub value: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct MonitoredCall {
    #[serde(rename = "ExpectedDepartureTime")]
    pub expected_departure_time: String,
    //#[serde(rename = "AimedDepartureTime")] pub aimed_departure_time: String,
    #[serde(rename = "DepartureStatus")]
    pub departure_status: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct Departure {
    #[serde(rename = "DirectionName")]
    pub direction_name: Vec<Value>,
    // #[serde(rename = "VehicleJourneyName")]
    // pub vehicle_journey_name: Vec<Value>,
    // #[serde(rename = "JourneyNote")]
    // pub journey_note: Vec<Value>,
    // #[serde(rename = "LineRef")]
    // pub line_ref: Value,
    #[serde(rename = "MonitoredCall")]
    pub monitored_call: MonitoredCall,
    //#[serde(rename = "VehicleFeatureRef")] pub vehicle_feature_ref: Vec<String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct MonitoredStopVisit {
    #[serde(rename = "MonitoredVehicleJourney")]
    pub monitored_vehicle_journey: Departure,
}

#[derive(Deserialize, Debug, Default)]
pub struct StopMonitoringDelivery {
    #[serde(rename = "MonitoredStopVisit")]
    pub monitored_stop_visit: Vec<MonitoredStopVisit>,
}

#[derive(Deserialize, Debug, Default)]
pub struct ServiceDelivery {
    #[serde(rename = "StopMonitoringDelivery")]
    pub stop_monitoring_delivery: Vec<StopMonitoringDelivery>,
}

#[derive(Deserialize, Debug, Default)]
pub struct Siri {
    #[serde(rename = "ServiceDelivery")]
    pub service_delivery: ServiceDelivery,
}

#[derive(Deserialize, Debug, Default)]
pub struct DepartureData {
    #[serde(rename = "Siri")]
    pub siri: Siri,
}

// General Message Data Types

#[derive(Deserialize, Debug, Default)]
pub struct GeneralMessageData {
    #[serde(rename = "Siri")]
    pub siri: SiriB,
}

#[derive(Deserialize, Debug, Default)]
pub struct SiriB {
    #[serde(rename = "ServiceDelivery")]
    pub service_delivery: ServiceDeliveryB,
}

#[derive(Deserialize, Debug, Default)]
pub struct ServiceDeliveryB {
    #[serde(rename = "GeneralMessageDelivery")]
    pub general_message_delivery: Vec<GeneralMessageDelivery>,
}

#[derive(Deserialize, Debug, Default)]
pub struct GeneralMessageDelivery {
    #[serde(rename = "InfoMessage")]
    pub info_message: Vec<InfoMessage>,
}

#[derive(Deserialize, Debug, Default)]
pub struct InfoMessage {
    #[serde(rename = "InfoChannelRef")]
    pub info_channel_ref: Value,
    #[serde(rename = "Content")]
    pub info_channel_content: Content,
}

#[derive(Deserialize, Debug, Default)]
pub struct Content {
    #[serde(rename = "Message")]
    pub message: Vec<Message>,
}
#[derive(Deserialize, Debug, Default)]
pub struct Message {
    #[serde(rename = "MessageText")]
    pub message_text: Value,
}

#[derive(Debug, Default)]
pub struct CollectedData {
    pub departure_data_list: Vec<DepartureData>,
    pub general_message_data_list: Vec<GeneralMessageData>,
}

pub async fn get_departures(config: Config) -> Result<CollectedData, Box<dyn std::error::Error>> {
    let mut departure_data = vec![];
    let mut general_message_data = vec![];
    let client = reqwest::Client::new();
    for station in config.stations {
        let body = client.get(format!("https://prim.iledefrance-mobilites.fr/marketplace/stop-monitoring?MonitoringRef=STIF%3AStopPoint%3AQ%3{}%3A&LineRef=STIF%3ALine%3A%3A{}%3A",station.stop_point_ref,station.line_ref))
        .header("apiKey", &config.api_key)
        .send()
        .await?
        .text()
        .await?;

        let departures: DepartureData = serde_json::from_str(&body)?;
        departure_data.push(departures);

        //
        let body = client.get(format!("https://prim.iledefrance-mobilites.fr/marketplace/general-message?LineRef=STIF%3ALine%3A%3A{}%3A",station.line_ref))
        .header("apiKey", &config.api_key)
        .send()
        .await?
        .text()
        .await?;

        let general_messages: GeneralMessageData = serde_json::from_str(&body)?;
        general_message_data.push(general_messages);
    }
    Ok(CollectedData {
        departure_data_list: departure_data,
        general_message_data_list: general_message_data,
    })
}
