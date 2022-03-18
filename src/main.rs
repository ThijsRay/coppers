// Copyright 2022 Jeffrey Bouman

mod sensors;
use sensors::{RAPLSensor, Sensor};
use std::time::Duration;
use std::thread::sleep;

fn main() {
    let mut sensor = RAPLSensor::new(String::from("/sys/devices/virtual/powercap/intel-rapl/intel-rapl:0")).unwrap();
    sensor.start_measuring();
    sleep(Duration::new(2,0));
    sensor.stop_measuring();
    println!("{} measured {}uJ", sensor.get_name(), sensor.get_measured_uj());
    println!("Hello, world!");
}
