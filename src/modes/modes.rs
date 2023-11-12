/*
 * Part of mlat-client-rust (https://github.com/tjmullicani/mlat-client-rust) - an ADS-B multilateration client.
 * Based on mlat-client (https://github.com/mutability/mlat-client)
 * Copyright 2023, Timothy Mullican <timothy.j.mullican@gmail.com>
 * Copyright 2015, Oliver Jowett <oliver@mutability.co.uk>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use std::collections::HashMap;
use std::cmp::Ordering;
use std::fmt;

use crate::modes::modes_message::*;

// Special DF types for non-Mode-S messages
pub const DF_MODEAC: u32 = 32;
pub const DF_EVENT_TIMESTAMP_JUMP: u32 = 33;
pub const DF_EVENT_MODE_CHANGE: u32 = 34;
pub const DF_EVENT_EPOCH_ROLLOVER: u32 = 35;
pub const DF_EVENT_RADARCAPE_STATUS: u32 = 36;
pub const DF_EVENT_RADARCAPE_POSITION: u32 = 37;

//mod modes_message;

// A structure representing a modes message.
#[derive(Debug)]
pub struct ModesMessage {
    pub timestamp: u64,                     /* 12MHz timestamp                                  */
    pub signal: u32,                        /* signal level                                     */

    pub df: u32,                            /* downlink format or a special DF_* value          */
    pub nuc: u32,                           /* NUCp value                                       */

    pub even_cpr: bool,                     /* CPR even-format flag                             */
    pub odd_cpr: bool,                      /* CPR odd-format flag                              */
    pub valid: bool,                        /* Does the message look OK?                        */
    pub crc: u32,                           /* CRC                                              */
    pub address: i32,                       /* ICAO address                                     */
    pub altitude: i32,                      /* altitude                                         */

    pub data: Vec<u8>,                      /*  */
    pub datalen: usize,                     /*  */

    eventdata: HashMap<String, String>, // Assuming the keys and values are Strings for simplicity. /* event data dictionary for special event messages */
}

// Constructs a new `ModesMessage`.
//
// # Arguments
//
// * `timestamp` - A timestamp for the message.
// * `signal` - The signal strength.
// * `df` - Downlink format.
// * `nuc` - Navigation uncertainty category.
// * `even_cpr` - Even CPR value.
// * `odd_cpr` - Odd CPR value.
// * `valid` - Validity flag.
// * `crc` - Cyclic redundancy check value.
// * `address` - Address of the sender.
// * `altitude` - Altitude information.
// * `data` - The payload data.
// * `datalen` - Length of the payload data.
// * `eventdata` - Additional event data.
impl ModesMessage {
    fn new(
        timestamp: u64,
        signal: u32,
        df: u32,
        nuc: u32,
        even_cpr: bool,
        odd_cpr: bool,
        valid: bool,
        crc: u32,
        address: i32,
        altitude: i32,
        data: Vec<u8>,
        datalen: usize,
        eventdata: HashMap<String, String>,
    ) -> Self {
        ModesMessage { 
            timestamp,
            signal,   
            df, 
            nuc,
            even_cpr,
            odd_cpr,
            valid,
            crc,
            address,
            altitude,
            data,
            datalen,
            eventdata,
        }
    }

    // Function to build a new message from a buffer.
    pub fn from_buffer(timestamp: u64, signal: u32, data: &[u8]) -> Result<Self, &'static str> {
        let datalen = data.len();
        let copydata = data.to_vec();

        // Assuming `decode` is a function that modifies the message in some way.
        // This function needs to be implemented.
        // if decode(&mut message) < 0 {
        //     return Err("Failed to decode message");
        // }

        Ok(ModesMessage {
            timestamp,
            signal,
            df: 0, // Placeholder value, as the original code does not specify.
            nuc: 0, // Placeholder value, as the original code does not specify.
            even_cpr: false, // Placeholder value, as the original code does not specify.
            odd_cpr: false, // Placeholder value, as the original code does not specify.
            valid: false, // Placeholder value, as the original code does not specify.
            crc: 0, // Placeholder value, as the original code does not specify.
            address: 0, // Placeholder value, as the original code does not specify.
            altitude: 0, // Placeholder value, as the original code does not specify.
            data: copydata,
            datalen,
            eventdata: HashMap::new(), // Placeholder value, as the original code does not specify.
        })
    }

    // Function to build a new event message.
    pub fn new_event_message(
        event_type: u32,
        timestamp: u64,
        eventdata: HashMap<String, String>,
    ) -> Self {
        ModesMessage {
            timestamp,
            signal: 0, // Placeholder value, as the original code does not specify.
            df: event_type,
            nuc: 0, // Placeholder value, as the original code does not specify.
            even_cpr: false, // Placeholder value, as the original code does not specify.
            odd_cpr: false, // Placeholder value, as the original code does not specify.
            valid: false, // Placeholder value, as the original code does not specify.
            crc: 0, // Placeholder value, as the original code does not specify.
            address: 0, // Placeholder value, as the original code does not specify.
            altitude: 0, // Placeholder value, as the original code does not specify.
            data: Vec::new(), // Placeholder value, as the original code does not specify.
            datalen: 0, // Placeholder value, as the original code does not specify.
            eventdata,
        }
    }
    
    // Retrieve an item from the HashMap.
    fn get_data_item(&self, key: &str) -> Option<&String> {
        self.eventdata.get(key)
    }

    fn decode(&mut self) -> i32 {
        let mut crc: u32;
        /* clear state */
        self.valid = false;
        self.nuc = 0;
        self.odd_cpr = false;
        self.even_cpr = false;
        self.crc = 0;
        self.address = 0;
        self.altitude = 0;
        if self.datalen == 2 {
            self.df = DF_MODEAC;
            self.address = ((self.data[0] as i32) << 8) | self.data[1] as i32;
            self.valid = true;
            return 0;
        }
        self.df = ((self.data[0] >> 3) & 31) as u32;
        if (self.df < 16 && self.datalen != 7) || (self.df >= 16 && self.datalen != 14) {
            /* wrong length, no further processing */
            return 0;
        }
        if self.df != 0 && self.df != 4 && self.df != 5 && self.df != 11 &&
            self.df != 16 && self.df != 17 && self.df != 18 && self.df != 20 && self.df != 21 {
            /* we do not know how to handle this message type, no further processing */
            return 0;
        }
        //crc = crc_residual(&self.data, self.datalen); //TODO: fixme
        //self.crc = crc;
        match self.df {
            0 | 4 | 16 | 20 => {
                self.address = self.crc as i32;
                self.altitude = decode_ac13(((self.data[2] & 0x1f) as u32) << 8 | (self.data[3] as u32)).unwrap() as i32;
                self.valid = true;
            },
            5 | 21 | 24 => {
                self.address = self.crc as i32;
                self.valid = true;
            },
            11 => {
                self.valid = (self.crc & !0x7f) == 0;
                if self.valid {
                    self.address = ((self.data[1] as u32) << 16 | (self.data[2] as u32) << 8 | self.data[3] as u32) as i32;
                }
            },
            17 | 18 => {
                self.valid = false;
                self.crc = 0;
                if self.valid {
                    let mut metype: u8;
                    self.address = ((self.data[1] as u32) << 16 | (self.data[2] as u32) << 8 | self.data[3] as u32) as i32;
                    metype = self.data[4] >> 3;
                    if (metype >= 9 && metype <= 18) || (metype >= 20 && metype < 22) {
                        if metype == 22 {
                            self.nuc = 0;
                        } else if metype <= 18 {
                            self.nuc = 18 - metype as u32;
                        } else {
                            self.nuc = 29 - metype as u32;
                        }
                        if self.data[6] & 0x04 != 0 {
                            self.odd_cpr = true;
                        } else {
                            self.even_cpr = true;
                        }
                        self.altitude = decode_ac12(((self.data[5] << 4) | ((self.data[6] & 0xF0) >> 4)) as u32).unwrap() as i32;
                    }
                }
            },
            _ => {},
        }
        return 0;
    }

    /// Returns the length of the data in the message.
    fn len(&self) -> usize {
        self.datalen
    }

    /// Calculates a hash for the message using a simple hashing algorithm.
    fn hash(&self) -> u32 {
        let mut hash: u32 = 0;

        // Jenkins one-at-a-time hash
        for i in 0..4.min(self.datalen as usize) {
            hash += self.data[i] as u32;
            hash = hash.wrapping_add(hash << 10);
            hash ^= hash >> 6;
        }

        hash = hash.wrapping_add(hash << 3);
        hash ^= hash >> 11;
        hash = hash.wrapping_add(hash << 15);

        hash as u32
    }

    /// Compares two `ModesMessage` instances.
    fn compare(&self, other: &Self) -> Ordering {
        if self.datalen != other.datalen {
            return self.datalen.cmp(&other.datalen);
        }
        self.data.as_slice().cmp(&other.data.as_slice())
    }
}

impl fmt::Display for ModesMessage {
    /// Formats the `ModesMessage` for display.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.data.is_empty() {
            let hex_data: String = self.data.iter()
                .map(|byte| format!("{:02x}", byte))
                .collect();
            write!(f, "{}", hex_data)
        } else {
            if let Some(event_name) = df_event_name(self.df) {
                write!(f, "{}@{}:{:?}", event_name, self.timestamp, self.eventdata)
            } else {
                write!(f, "DF{}@{}:{:?}", self.df, self.timestamp, self.eventdata)
            }
        }
    }
}
