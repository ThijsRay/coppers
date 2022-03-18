// Copyright 2022 Jeffrey Bouman
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use regex::Regex;
use std::fs::read_to_string;
use std::option::Option;
use std::result::Result;
use std::time::{Duration, Instant};

// Trait for all kind of sensors to implement
pub trait Sensor {
    // Retrieve name of the sensor
    fn get_name(&self) -> String;
    // Start position to measure the power consumption and timer
    fn start_measuring(&mut self);
    // Stop position to measure the power consumption and timer
    fn stop_measuring(&mut self);
    // Retrieve the final value from the sensor, AFTER start and stop
    fn get_measured_uj(&self) -> u128;
    // Retrieve the elapsed time between start and stop call
    fn get_elapsed_time_us(&self) -> u128;
    // Retrieve a duration value instead of seconds directly
    fn get_duration(&self) -> Duration;
}

#[derive(Debug)]
pub struct RAPLSensor {
    location: String,
    name: String,
    // Timer values
    timer_start_position: Option<Instant>,
    timer_end_position: Option<Instant>,
    // Energy values
    energy_start_position: u128,
    energy_end_position: u128,
    energy_max_range: u128,
}

// Sensor trait implementation for RAPLSensor
impl Sensor for RAPLSensor {
    fn get_name(&self) -> String {
        return self.name.to_string();
    }

    fn start_measuring(&mut self){
        let measuring_location = self.location.to_string() + "/energy_uj";
        // Access rights and enabling is checked at initialization of sensor so unwrap is allowed
        let current_measured_uj = read_to_string(measuring_location).unwrap();
        self.energy_start_position = RAPLSensor::convert_read_string_to_u128(current_measured_uj);

        self.timer_start_position = Some(Instant::now());
    }

    fn stop_measuring(&mut self) {
        let measuring_location = self.location.to_string() + "/energy_uj";
        let current_measured_uj = read_to_string(measuring_location).unwrap();
        self.energy_end_position = RAPLSensor::convert_read_string_to_u128(current_measured_uj);

        self.timer_end_position = Some(Instant::now());
    }

    fn get_measured_uj(&self) -> u128 {
        if self.energy_end_position < self.energy_start_position {
            (self.energy_max_range - self.energy_start_position) + self.energy_end_position
        } else {
            self.energy_end_position - self.energy_start_position
        }
    }

    fn get_elapsed_time_us(&self) -> u128 {
        // Check whether both instants are set correctly
        if self.timer_start_position.is_none() || self.timer_end_position.is_none() {
            return 0;
        }
        // Now unwrappable due to above check
        let start = self.timer_start_position.unwrap();
        let end = self.timer_end_position.unwrap();
        end.duration_since(start).as_micros()
    }

    fn get_duration(&self) -> Duration {
        // Check whether both instants are set correctly
        if self.timer_start_position.is_none() || self.timer_end_position.is_none() {
            return Duration::new(0, 0);
        }
        // Now unwrappable due to above check
        let start = self.timer_start_position.unwrap();
        let end = self.timer_end_position.unwrap();
        end.saturating_duration_since(start)
    }
}

// Implementation of RAPLSensor sepcific functions
impl RAPLSensor {
    pub fn new(location: String) -> Result<RAPLSensor, String> {
        let enabled_location = format!("{location}/enabled");
        let enabled = read_to_string(&enabled_location);
        // Check whether the location is actually reachable
        if enabled.is_err() {
            return Err(format!("The location `{location}` is unreachable"));
        }
        // Check if RAPL is enabled at this location
        // No error received to unwrap is possible
        /*if !enabled.unwrap().starts_with("1") {       ENABLED ONLY IN PARENT DIRECTORY NOT USABLE IN CURRENT STATE
            return Err("RAPL sensor is not enabled");
        }*/
        // Check whether permission is set correctly of the measuring location
        let measuring_location = format!("{location}/energy_uj");
        let measured_energy = read_to_string(&measuring_location);
        if measured_energy.is_err() {
            return Err(format!("Insufficient permissions to read from {measuring_location}. You might want to retry as root."));
        }
        // Retrieve the name of the sensor
        let name_location = location.to_string() + "/name";
        let mut name = read_to_string(name_location).unwrap();
        name.pop(); // remove trailing breakline
        // Retrieve the max range value of the sensor
        let max_range_location = location.to_string() + "/max_energy_range_uj";
        let max_range_string = read_to_string(max_range_location).unwrap();
        let max_range = RAPLSensor::convert_read_string_to_u128(max_range_string);

        Ok(RAPLSensor {
            location: location,
            name: name,
            timer_start_position: None,
            timer_end_position: None,
            energy_start_position: 0,
            energy_end_position: 0,
            energy_max_range: max_range,
        })
    }

    // Input should look like "xxxxxxxxx\n"
    fn convert_read_string_to_u128(input_string: String) -> u128 {
        // One or more digit(s) followed by a breakline
        // My re is awesome so unwrap directly :P
        let re = Regex::new(r"^([0-9]+)\n").unwrap();
        if !re.is_match(input_string.as_str()) {
            return 0;
        }
        let captures = re.captures(input_string.as_str()).unwrap();
        let value: u128 = captures.get(1).map_or(0, |m| m.as_str().parse().unwrap());
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raplsensor_convert_read_string_to_u128_zero() {
        let result = RAPLSensor::convert_read_string_to_u128(String::from("0\n"));
        assert_eq!(result, 0);
    }

    #[test]
    fn raplsensor_convert_read_string_to_u128_one() {
        let result = RAPLSensor::convert_read_string_to_u128(String::from("1\n"));
        assert_eq!(result, 1);
    }

    #[test]
    fn raplsensor_convert_read_string_to_u128_float() {
        let result = RAPLSensor::convert_read_string_to_u128(String::from("20.22\n"));
        assert_eq!(result, 0);
    }

    #[test]
    fn raplsensor_convert_read_string_to_u128_garbage_end() {
        let result = RAPLSensor::convert_read_string_to_u128(String::from("2022\nasnsdb11786"));
        assert_eq!(result, 2022);
    }

    #[test]
    fn raplsensor_convert_read_string_to_u128_large() {
        let result = RAPLSensor::convert_read_string_to_u128(String::from(
            "12345678901234567890123456789\n",
        ));
        assert_eq!(result, 12345678901234567890123456789);
    }
}
