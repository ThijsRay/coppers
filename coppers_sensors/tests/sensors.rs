// Copyright 2022 Thijs Raymakers, Jeffrey Bouman
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

use coppers_sensors::*;
use std::thread::sleep;
use std::time::Duration;

#[test]
fn test_rapl_sensor() {
    let mut sensor = RAPLSensor::new(String::from(
        "/sys/devices/virtual/powercap/intel-rapl/intel-rapl:0",
    ))
    .unwrap();
    sensor.start_measuring();
    sleep(Duration::new(2, 0));
    sensor.stop_measuring();
    println!("measured {}uJ", sensor.get_measured_uj());
}
