// Copyright 2022 Jeffrey Bouman

use std::time::{Duration, Instant};
use nvml_wrapper as nvml;
use nvml::{NVML, Device};
use super::Sensor;

pub struct NvmlSensor<'a> {
    device: Device<'a>,
    name: String,
    // Timer values
    timer_start_position: Option<Instant>,
    timer_end_position: Option<Instant>,
    // Energy values
    energy_start_position: u128,
    energy_end_position: u128,
    energy_max_range: u128,
}

impl Sensor for NvmlSensor<'static> {
    fn start_measuring(&mut self){
        let mj_measurement = self.device.total_energy_consumption().unwrap();
        // Convert mj to uj
        self.energy_start_position = u128::from(mj_measurement) * 1000;
        
        self.timer_start_position = Some(Instant::now());
    }

    fn stop_measuring(&mut self){
        let mj_measurement = self.device.total_energy_consumption().unwrap();
        // Convert mj to uj
        self.energy_end_position = u128::from(mj_measurement) * 1000;
        
        self.timer_end_position = Some(Instant::now());
    }
    
    fn get_measured_uj(&self) -> u128 {
        if self.energy_end_position < self.energy_start_position {
            return (self.energy_max_range - self.energy_start_position) + self.energy_end_position;
        } else {
            return self.energy_end_position - self.energy_start_position;
        }
    }
    
    fn get_elapsed_time_us(&self) -> u128 {
        if self.timer_start_position.is_none() || self.timer_end_position.is_none() {
            return 0;
        }
        // Now unwrappable due to above check
        let start = self.timer_start_position.unwrap();
        let end = self.timer_end_position.unwrap();
        return end.duration_since(start).as_micros();
    }
    
    fn get_duration(&self) -> Duration {
        // Check whether both instants are set correctly
        if self.timer_start_position.is_none() || self.timer_end_position.is_none() {
            return Duration::new(0, 0);
        }
        // Now unwrappable due to above check
        let start = self.timer_start_position.unwrap();
        let end = self.timer_end_position.unwrap();
        return end.saturating_duration_since(start);
    }

}

impl NvmlSensor<'static> {
    // TODO: It is better to retrieve the device based on its uuid
    pub fn new<'a>(nvml: &'a NVML, index: u32) -> Result<NvmlSensor<'a>, &'static str> {
        let device_result = nvml.device_by_index(index);
        // Check if nvml van be initialized
        if device_result.is_err() {
            return Err("Unable to retrieve device with index");
        }
        let device = device_result.unwrap();
        
        let name = device.name().unwrap();
        Ok(NvmlSensor {
            device: device,
            name: name,
            timer_start_position: None,
            timer_end_position: None,
            energy_start_position: 0,
            energy_end_position: 0,
            energy_max_range: 0,
        })
    }
}