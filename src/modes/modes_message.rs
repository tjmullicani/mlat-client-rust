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
 * References:
 *   https://www.codeconvert.ai/c-to-rust-converter
 *   https://thepythoncode.com/assistant/code-converter/rust/
 *   https://mode-s.org/decode/content/ads-b/8-error-control.html
 *   https://mode-s.org/decode/book-the_1090mhz_riddle-junzi_sun.pdf
 */

use std::cmp::Ordering;
use std::fmt;
use std::collections::BTreeMap;
use hex_slice::AsHex;

use crate::modes::modes_crc;
use crate::modes::modes::{self, *};
use crate::modes::modes_reader::{self, *};

/// Decodes altitude information from a compact binary format (AC13 format).
///
/// # Arguments
///
/// * `ac13` - An unsigned integer representing the encoded altitude.
///
/// # Returns
///
/// Returns an Option<i32> representing the decoded altitude in feet, or None if the input is invalid.
pub fn decode_ac13(ac13: u32) -> Option<i32> {
    let mut h = 0;
    let mut f = 0;
    //let mut a;

    // Check if ac13 is zero or if the M bit (bit 6) is set
    if ac13 == 0 || (ac13 & 0x0040) != 0 {
        return None;
    }

    // Check if the Q bit (bit 4) is set
    if (ac13 & 0x0010) != 0 {
        // Calculate altitude using a different encoding scheme
        let n = ((ac13 & 0x1F80) >> 2) | ((ac13 & 0x0020) >> 1) | (ac13 & 0x000F);
        let altitude = (n as i32) * 25 - 1000;
        return Some(altitude);
    }

    // Decode altitude using the Gillham code (Gray code)
    // Check for illegal Gillham code
    if (ac13 & 0x1500) == 0 {
        return None;
    }

    // Calculate the hundreds (h) and the fractional part (f) of the altitude
    if ac13 & 0x1000 != 0 { h ^= 7; } // C1
    if ac13 & 0x0400 != 0 { h ^= 3; } // C2
    if ac13 & 0x0100 != 0 { h ^= 1; } // C4

    if h & 5 != 0 {
        h ^= 5;
    }

    if h > 5 {
        // Illegal value for h
        return None;
    }

    if ac13 & 0x0010 != 0 { f ^= 0x1ff; } // D1
    if ac13 & 0x0004 != 0 { f ^= 0x0ff; } // D2
    if ac13 & 0x0001 != 0 { f ^= 0x07f; } // D4
    if ac13 & 0x0800 != 0 { f ^= 0x03f; } // A1
    if ac13 & 0x0200 != 0 { f ^= 0x01f; } // A2
    if ac13 & 0x0080 != 0 { f ^= 0x00f; } // A4
    if ac13 & 0x0020 != 0 { f ^= 0x007; } // B1
    if ac13 & 0x0008 != 0 { f ^= 0x003; } // B2
    if ac13 & 0x0002 != 0 { f ^= 0x001; } // B4
    /*if ac13 & 0x0800 != 0 { f ^= 0x03f; } // A1
    if ac13 & 0x0200 != 0 { f ^= 0x01f; } // A2
    if ac13 & 0x0080 != 0 { f ^= 0x00f; } // A4
    if ac13 & 0x0020 != 0 { f ^= 0x007; } // B1
    if ac13 & 0x0008 != 0 { f ^= 0x003; } // B2
    if ac13 & 0x0002 != 0 { f ^= 0x001; } // B4*/

    if f & 1 != 0 {
        h = 6 - h;
    }

    // Calculate the altitude
    //let a = 500 * (f as i32) + 100 * (h as i32) - 1300;
    let a = 500 * f + 100 * h - 1300;
    if a < -1200 {
        // Illegal altitude value
        return None;
    }

    // Return the decoded altitude
    return Some(a);
}

/// Helper function to decode altitude information from a compact binary format (AC12 format).
///
/// # Arguments
///
/// * `ac12` - An unsigned integer representing the encoded altitude in AC12 format.
///
/// # Returns
///
/// Returns an Option<i32> representing the decoded altitude in feet, or None if the input is invalid.
pub fn decode_ac12(ac12: u32) -> Option<i32> {
    // Reformat the bits to match the AC13 format
    let ac13 = ((ac12 & 0x0fc0) << 1) | (ac12 & 0x003f);
    // Call decode_ac13 to do the actual decoding
    decode_ac13(ac13)
}

// replaced by checksum() function in modes_crc.rs?
/*
pub fn crc_residual(message: &[u8], len: usize) -> u32 {
    let mut crc: u32;
    if len < 3 {
        return 0;
    }
    crc = modes_crc::modescrc_buffer_crc(&message[..len - 3]); //FIXME: hardcoded
    crc = crc ^ ((message[len - 3] as u32) << 16);
    crc = crc ^ ((message[len - 2] as u32) << 8);
    crc = crc ^ (message[len - 1] as u32);
    crc
}*/

// Returns the event name associated with a given DF event code.
pub fn df_event_name(df: u32) -> Option<String> {
    match df {
        DF_EVENT_TIMESTAMP_JUMP => Some("DF_EVENT_TIMESTAMP_JUMP".to_string()),
        DF_EVENT_MODE_CHANGE => Some("DF_EVENT_MODE_CHANGE".to_string()),
        DF_EVENT_EPOCH_ROLLOVER => Some("DF_EVENT_EPOCH_ROLLOVER".to_string()),
        DF_EVENT_RADARCAPE_STATUS => Some("DF_EVENT_RADARCAPE_STATUS".to_string()),
        _ => None,
    }
}

// internal entry point to build a new message from a buffer
pub fn modesmessage_from_buffer(timestamp: u64, signal: u8, data: Vec<u8>, datalen: usize) -> ModesMessage {
    let copydata = data;

    let mut message = ModesMessage::default();
    message.timestamp = timestamp;
    message.signal = signal;
    message.data = copydata;

    message
}

// internal entry point to build a new event message
// steals a reference from eventdata
pub fn modesmessage_new_eventmessage(msgtype: u32, timestamp: u64, eventdata: BTreeMap<String, EventData>) -> ModesMessage {
    let mut message: ModesMessage = ModesMessage::default();

    message.df = msgtype;
    message.timestamp = timestamp;
    message.eventdata = eventdata;
    
    message
}

// A structure representing a modes message.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ModesMessage {
    pub timestamp: u64,                        // 12MHz timestamp
    pub signal: u8,                            // signal level

    pub df: u32,                               // downlink format or a special DF_* value
    pub nuc: u32,                              // Navigation uncertainty category; NUCp value

    pub even_cpr: bool,                        // CPR even-format flag
    pub odd_cpr: bool,                         // CPR odd-format flag
    pub valid: bool,                           // Does the message look OK?
    pub crc: u32,                              // Cyclic redundancy check value
    pub address: i32,                          // ICAO address
    pub altitude: i32,                         // altitude information

    pub data: Vec<u8>,                         // The payload data
    pub datalen: usize,                        // Length of the payload data

    eventdata: BTreeMap<String, EventData>,     // event data dictionary for special event messages
}

impl ModesMessage {
    fn new(
        timestamp: u64,
        signal: u8,
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
        eventdata: BTreeMap<String, EventData>,
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

    fn default() -> Self {
        // minimal init
        ModesMessage {
            timestamp: 0,
            signal: 0,
            df: 0,
            nuc: 0,
            even_cpr: false,
            odd_cpr: false,
            valid: false,
            crc: 0,
            address: 0,
            altitude: 0,
            data: Vec::new(),
            datalen: 0,
            eventdata: BTreeMap::new(),
        }
    }

    // Function to build a new message from a buffer.
    pub fn from_buffer(timestamp: u64, signal: u32, data: Vec<u8>) -> Result<Self, &'static str> {
        let datalen = data.len();
        let copydata = data;

        // Assuming `decode` is a function that modifies the message in some way.
        // This function needs to be implemented.
        // if decode(&mut message) < 0 {
        //     return Err("Failed to decode message");
        // }

        Ok(ModesMessage::default())
    }

    // Function to build a new event message.
    pub fn new_event_message(
        event_type: u32,
        timestamp: u64,
        eventdata: BTreeMap<String, String>,
    ) -> Self {
        ModesMessage::default()
    }

    fn decode(&mut self) -> i32 {
        let mut crc: u32;

        // clear state
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
            // wrong length, no further processing
            return 0;
        }
        if self.df != 0 && self.df != 4 && self.df != 5 && self.df != 11 &&
            self.df != 16 && self.df != 17 && self.df != 18 && self.df != 20 && self.df != 21 {
            // we do not know how to handle this message type, no further processing
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
