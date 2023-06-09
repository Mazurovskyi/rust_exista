use std::process;
use std::{thread, time::Duration};

use paho_mqtt::{AsyncClient, Message};

use crate::application::{loger::Log, constants::*};

mod msg;
use msg::Handler;


pub struct Callbacks{
    message_callback: Option<fn(&AsyncClient, Option<Message>)>,
    connected: Option<fn(&AsyncClient)>,
    connection_lost: Option<fn(&AsyncClient)>,
    on_connect_success: Option<fn(&AsyncClient, u16)>,
    on_connect_failure: Option<fn(&AsyncClient, u16, i32)>
}
impl Callbacks{
    /// returns mqtt callbacks
    pub fn new()->Self{
        Self {
            message_callback: Some(message_callback), 
            connected: Some(connected), 
            connection_lost: Some(connection_lost), 
            on_connect_success: Some(on_connect_success), 
            on_connect_failure: Some(on_connect_failure)
        }
    }

    /// callback on incoming messages.
    pub fn message_callback(&mut self)->fn(&AsyncClient, Option<Message>){
        self.message_callback.take().unwrap()
    }

    /// closure to be called when connection is established.
    pub fn connected(&mut self)->fn(&AsyncClient){
        self.connected.take().unwrap()
    }

    /// closure to be called if the client loses the connection. Try to reconect
    pub fn connection_lost(&mut self)->fn(&AsyncClient){
        self.connection_lost.take().unwrap()
    }

    /// Callback for a successful connection to the broker. Subscribe the topics
    pub fn on_connect_success(&mut self)->fn(&AsyncClient, u16){
        self.on_connect_success.take().unwrap()
    }

    /// Callback for a fail connection
    pub fn on_connect_failure(&mut self)->fn(&AsyncClient, u16, i32){
        self.on_connect_failure.take().unwrap()
    }
}


fn message_callback(_client: &AsyncClient, msg: Option<Message>){
    match msg.handle(){
        Ok(report) => Log::write(format!("mqtt message handle result: {report}").as_str()),
        Err(report) => {
            Log::write(format!("Error was heappen handling the message: {report}").as_str());
            process::exit(1);
        }
    }
}


fn connected(_client: &AsyncClient){
    Log::write("connected to mqtt broker")
}


fn connection_lost(client: &AsyncClient){
    Log::write("mqtt broker connection lost. Trying to reconnect...");
    thread::sleep(Duration::from_millis(1000));
    client.reconnect_with_callbacks(on_connect_success, on_connect_failure);
}


fn on_connect_success(client: &AsyncClient, _msgid: u16){

    client.subscribe_many(&[TOPIC_BATTERY_INFO_REQ, TOPIC_DEVICE_INFO], &[QOS, QOS]);

    Log::write(
        format!("successful connection to the broker.
        subscribed to topics: {:?}", [TOPIC_BATTERY_INFO_REQ, TOPIC_DEVICE_INFO]).as_str()
    );
}


fn on_connect_failure(client: &AsyncClient, _msgid: u16, rc: i32){
    Log::write(
        format!("Connection attempt failed with error code {}.
        trying to reconnect...", rc).as_str()
    );
        
    thread::sleep(Duration::from_millis(1000));
    client.reconnect_with_callbacks(on_connect_success, on_connect_failure);
}

