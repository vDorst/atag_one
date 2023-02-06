use reqwest::{
    self,
    header::{HeaderValue, CONNECTION, CONTENT_TYPE},
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::{io::Write, time::Duration};

#[allow(non_snake_case)]
#[derive(Deserialize, Debug, PartialEq, Serialize)]
struct ReportDetailReply {
    boiler_temp: f32,
    boiler_return_temp: f32,
    min_mod_level: u8,
    rel_mod_level: u8,
    boiler_capacity: u8,
    target_temp: f32,
    overshoot: f32,
    max_boiler_temp: f32,
    alpha_used: f32,
    regulation_state: u8,
    ch_m_dot_c: f32,
    c_house: u8,
    r_rad: f32,
    r_env: f32,
    alpha: f32,
    alpha_max: f32,
    delay: u8,
    mu: f32,
    threshold_offs: f32,
    wd_k_factor: f32,
    wd_exponent: f32,
    lmuc_burner_hours: u32,
    lmuc_dhw_hours: u32,
    KP: f32,
    KI: f32,
}

#[derive(Deserialize, Debug, PartialEq, Serialize)]
struct ReportReply {
    report_time: u64,
    burning_hours: f32,
    device_errors: String,
    boiler_errors: String,
    room_temp: f32,
    outside_temp: f32,
    dbg_outside_temp: f32,
    pcb_temp: f32,
    ch_setpoint: f32,
    dhw_water_temp: f32,
    ch_water_temp: f32,
    dhw_water_pres: f32,
    ch_water_pres: f32,
    ch_return_temp: f32,
    boiler_status: u16,
    boiler_config: u16,
    ch_time_to_temp: u16,
    shown_set_temp: f32,
    power_cons: u16,
    tout_avg: f32,
    rssi: u16,
    current: i16,
    voltage: u16,
    charge_status: u16,
    lmuc_burner_starts: u32,
    dhw_flow_rate: f32,
    resets: u16,
    memory_allocation: u16,
    details: Option<ReportDetailReply>,
}

#[derive(Deserialize, Debug, PartialEq, Serialize)]
struct StatusReply {
    device_id: String,
    device_status: u16,
    connection_status: u8,
    date_time: u64,
}

#[derive(Deserialize, Debug, PartialEq, Serialize)]
struct Retrievereplay {
    seqnr: u16,
    status: Option<StatusReply>,
    report: Option<ReportReply>,
    acc_status: u8,
}

#[derive(Deserialize, Debug, PartialEq, Serialize)]
struct JSONResponce {
    retrieve_reply: Retrievereplay,
}

async fn send_request(path: &str, payload: String) -> Result<String, reqwest::Error> {
    let u = format!("http://{}:10000{}", IP_ADDRESS, path);
    println!("URL: {}", u);

    println!("REQ: {}", payload);

    // let u = Url::parse(&u)?;

    let client = reqwest::Client::builder()
        .user_agent("Python-urllib/3.9")
        // Atag One is case sensitive
        .http1_title_case_headers()
        //.http1_only()
        .timeout(Duration::from_secs(3))
        .build()?;

    let resp = client
        .post(&u)
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .header(CONNECTION, "close")
        .body(payload)
        .send()
        .await?;

    println!("Status: {}", resp.status());
    println!("Headers:\n{:#?}", resp.headers());

    let res = resp.text().await?;

    Ok(res)
}

#[derive(Debug, Serialize)]
struct AccountAuth<'a> {
    user_account: &'a str,
    mac_address: &'a str,
}

#[derive(Debug, Serialize)]
struct RetrieveMessage<'a> {
    seqnr: u16,
    account_auth: AccountAuth<'a>,
    info: u8,
}

#[derive(Debug, Serialize)]
struct JSONMessage<'a> {
    retrieve_message: RetrieveMessage<'a>,
}

const MAC_ADDRESS: &str = "d8:61:62:00:00:00";
//const IP_ADDRESS: &str = "192.168.2.1";
const IP_ADDRESS: &str = "192.168.2.243";

const READ_PATH: &str = "/retrieve";
// const UPDATE_PATH: &str = "/update";

// https://github.com/kozmoz/atag-one-api/wiki/Thermostat-Protocol
// 01 = Control
// 02 = Schedules
// 04 = Configuration
// 08 = Report
// 16 = Status
// 32 = Wifi scan
// 64 = Report details

#[allow(dead_code)]
const INFO_CNTL: u8 = 0x01;
#[allow(dead_code)]
const INFO_SDL: u8 = 0x02;
#[allow(dead_code)]
const INFO_CFG: u8 = 0x04;
#[allow(dead_code)]
const INFO_RPT: u8 = 0x08;
#[allow(dead_code)]
const INFO_STS: u8 = 0x10;
#[allow(dead_code)]
const INFO_WFS: u8 = 0x20;
#[allow(dead_code)]
const INFO_RPTDL: u8 = 0x40;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let mut req = JSONMessage {
        retrieve_message: RetrieveMessage {
            seqnr: 1,
            account_auth: AccountAuth {
                user_account: "1",
                mac_address: MAC_ADDRESS,
            },
            info: INFO_RPTDL,
        },
    };

    let mut f = std::fs::File::options()
        .create(true)
        .append(true)
        .open("json.log")
        .unwrap();

    loop {
        req.retrieve_message.seqnr = req.retrieve_message.seqnr.wrapping_add(1);
        let ret = send_request(READ_PATH, serde_json::to_string(&req).unwrap()).await?;
        f.write_all(ret.as_bytes()).unwrap();
        f.write_all("\n".as_bytes()).unwrap();
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_reply() {
        let inp = r#"{ "retrieve_reply":{ "seqnr":1,"status":{"device_id":"0000-0000-0000_00-00-000-000","device_status":16385,"connection_status":7,"date_time":728864357},"acc_status":2} }"#;

        let ret = JSONResponce {
            retrieve_reply: Retrievereplay {
                seqnr: 1,
                status: Some(StatusReply {
                    device_id: "0000-0000-0000_00-00-000-000".to_owned(),
                    device_status: 16385,
                    connection_status: 7,
                    date_time: 728864357,
                }),
                report: None,
                acc_status: 2,
            },
        };

        assert_eq!(serde_json::from_str::<JSONResponce>(inp).unwrap(), ret);
    }

    #[test]
    fn test_compleet_reply() {
        let inp = r#"{ "retrieve_reply":{ "seqnr":1,"status":{"device_id":"0000-0000-0000_00-00-000-000","device_status":16385,"connection_status":7,"date_time":728865408},"report":{"report_time":728865408,"burning_hours":2560.10,"device_errors":"","boiler_errors":"","room_temp":18.3,"outside_temp":-100.0,"dbg_outside_temp":21.0,"pcb_temp":25.3,"ch_setpoint":0.0,"dhw_water_temp":43.3,"ch_water_temp":18.3,"dhw_water_pres":0.0,"ch_water_pres":1.4,"ch_return_temp":18.3,"boiler_status":516,"boiler_config":772,"ch_time_to_temp":0,"shown_set_temp":16.0,"power_cons":131,"tout_avg":0.0,"rssi":47,"current":7,"voltage":3850,"charge_status":1,"lmuc_burner_starts":35865,"dhw_flow_rate":0.0,"resets":2,"memory_allocation":5704,"details":{"boiler_temp":52.5,"boiler_return_temp":49.5,"min_mod_level":22,"rel_mod_level":0,"boiler_capacity":0,"target_temp":16.0,"overshoot":0.000,"max_boiler_temp":70.0,"alpha_used":0.00007,"regulation_state":1,"ch_m_dot_c":0.000,"c_house":0,"r_rad":0.0000,"r_env":0.0000,"alpha":0.00000,"alpha_max":0.00007,"delay":1,"mu":0.30,"threshold_offs":15.0,"wd_k_factor":1.6,"wd_exponent":1.2,"lmuc_burner_hours":2560,"lmuc_dhw_hours":29730,"KP":24.900,"KI":0.00433}},"acc_status":2} }"#;

        let ret = JSONResponce {
            retrieve_reply: Retrievereplay {
                seqnr: 1,
                status: Some(StatusReply {
                    device_id: "0000-0000-0000_00-00-000-000".to_string(),
                    device_status: 16385,
                    connection_status: 7,
                    date_time: 728865408,
                }),
                report: Some(ReportReply {
                    report_time: 728865408,
                    burning_hours: 2560.1,
                    device_errors: "".to_string(),
                    boiler_errors: "".to_string(),
                    room_temp: 18.3,
                    outside_temp: -100.0,
                    dbg_outside_temp: 21.0,
                    pcb_temp: 25.3,
                    ch_setpoint: 0.0,
                    dhw_water_temp: 43.3,
                    ch_water_temp: 18.3,
                    dhw_water_pres: 0.0,
                    ch_water_pres: 1.4,
                    ch_return_temp: 18.3,
                    boiler_status: 516,
                    boiler_config: 772,
                    ch_time_to_temp: 0,
                    shown_set_temp: 16.0,
                    power_cons: 131,
                    tout_avg: 0.0,
                    rssi: 47,
                    current: 7,
                    voltage: 3850,
                    charge_status: 1,
                    lmuc_burner_starts: 35865,
                    dhw_flow_rate: 0.0,
                    resets: 2,
                    memory_allocation: 5704,
                    details: Some(ReportDetailReply {
                        boiler_temp: 52.5,
                        boiler_return_temp: 49.5,
                        min_mod_level: 22,
                        rel_mod_level: 0,
                        boiler_capacity: 0,
                        target_temp: 16.0,
                        overshoot: 0.0,
                        max_boiler_temp: 70.0,
                        alpha_used: 7e-5,
                        regulation_state: 1,
                        ch_m_dot_c: 0.0,
                        c_house: 0,
                        r_rad: 0.0,
                        r_env: 0.0,
                        alpha: 0.0,
                        alpha_max: 7e-5,
                        delay: 1,
                        mu: 0.3,
                        threshold_offs: 15.0,
                        wd_k_factor: 1.6,
                        wd_exponent: 1.2,
                        lmuc_burner_hours: 2560,
                        lmuc_dhw_hours: 29730,
                        KP: 24.9,
                        KI: 0.00433,
                    }),
                }),
                acc_status: 2,
            },
        };

        assert_eq!(serde_json::from_str::<JSONResponce>(inp).unwrap(), ret);
    }
}
