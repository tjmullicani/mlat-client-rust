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
use std::collections::HashMap;
use hex_slice::AsHex;

use crate::modes::modes_crc;

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

pub fn crc_residual(message: &[u8], len: usize) -> u32 {
    let mut crc: u32;
    if len < 3 {
        return 0;
    }
    //crc = modes_crc::modescrc_buffer_crc(&message[..len - 3]); //FIXME: hardcoded
    crc = 0;
    crc = crc ^ ((message[len - 3] as u32) << 16);
    crc = crc ^ ((message[len - 2] as u32) << 8);
    crc = crc ^ (message[len - 1] as u32);
    crc
}

/// Returns the event name associated with a given DF event code.
pub fn df_event_name(df: u32) -> Option<&'static str> {
    match df {
        DF_EVENT_TIMESTAMP_JUMP => Some("DF_EVENT_TIMESTAMP_JUMP"),
        DF_EVENT_MODE_CHANGE => Some("DF_EVENT_MODE_CHANGE"),
        DF_EVENT_EPOCH_ROLLOVER => Some("DF_EVENT_EPOCH_ROLLOVER"),
        DF_EVENT_RADARCAPE_STATUS => Some("DF_EVENT_RADARCAPE_STATUS"),
        _ => None,
    }
}
