
use std::borrow::{Borrow, BorrowMut};
use std::fmt::Display;

use json::JsonValue;

use crate::application::constants::cube_serial_num::CubeSerialNumber;
use crate::requests::ModbusMsg;
use crate::application::constants::*;

use super::*;

pub struct BatteryInfo{
    json: JsonValue,
    modbus_requests: Vec<ModbusMsg>,
    publish_topic: &'static str,
    qos: i32
}
impl BatteryInfo{
    pub fn new()->Self{
        Self {
            json: Self::build_json(), 
            modbus_requests: Self::build_request_list(),
            publish_topic: TOPIC_BATTERY_INFO_REP,
            qos: 0
        }
    }


    fn json(&self)->&JsonValue{
        self.json.borrow()
    }
    fn json_mut(&mut self)->& mut JsonValue{
        self.json.borrow_mut()
    }
    fn requests_list(&self)-> &Vec<ModbusMsg>{
        self.modbus_requests.borrow()
    }
    fn build_request_list()->Vec<ModbusMsg>{
        [READ_DC_STATUS, READ_BATTERY_STATUS, READ_VOLTAGE,
        READ_CURRENT_VALUE, READ_SOC, READ_SOH, READ_BACKUP_TIME]
            .iter()
            .map(|msg|ModbusMsg::from(&msg[..], msg.len()))
            .collect()
    }


    // decoding operations 
    fn decode(msg: ModbusMsg, i: usize)->Option<JsonValue>{
        if i == 4 || i == 5 {
            ModbusMsg::registers_value_percent(msg.data())
        }
        else{
            ModbusMsg::registers_value(msg.data())
        } 
    }
}

impl JsonCreation for BatteryInfo{
    fn build_json()->JsonValue {
        object! {
            serialNumber: null,
            batteryInfo: {
                comStatus: null,
                dcStatus: null,
                batteryStatus: null,
                batteryVoltage: null,
                batteryCurrent: null,
                soc: null,
                soh: null,
                timeLeft: null
            }
        }
    }
}

impl MqttSending for BatteryInfo{
    fn serialize(&self)->String{
        self.json.dump()
    }
    fn topic(&self)->&str{
        self.publish_topic
    }
    fn qos(&self)->i32{
        self.qos
    }
    fn bat_ic_low(&self)->bool{
        false
    }
}

impl RequestObject for BatteryInfo{
    fn insert_data<'a>(&mut self, bus: &'a Modbus)->Result<(), Box<dyn Error + 'a>>{

        let serial_number: JsonValue = CubeSerialNumber::get().into();
        let com_status: JsonValue  = bus.get_status().into();
        let mut battery_info = Vec::from([serial_number, com_status]);


        let raw_data = self.get_modbus_data(bus)?;
        let parsed_data = self.parse_modbus_data(raw_data);

        battery_info.extend(parsed_data);

        self.json_mut().assign(battery_info);
        Ok(())
    }
}

impl ModbusData for BatteryInfo{
    fn get_modbus_data<'a>(&self, bus: &'a Modbus)->Result<Vec<ModbusMsg>, Box<dyn Error + 'a>>{
        let mut modbus_replies = Vec::new();

        for msg in self.requests_list(){
            modbus_replies.push(bus.send(msg)?)
        };

        Ok(modbus_replies)
    }
    fn parse_modbus_data(&self, raw_data: Vec<ModbusMsg>)->Vec<JsonValue>{
        raw_data.into_iter().enumerate().map(|(i, msg)|Self::decode(msg, i).into()).collect()
    }
}

impl Display for BatteryInfo{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(BatteryInfo:,\n{})", self.json().pretty(4))
    }
}

