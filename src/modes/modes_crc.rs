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
 *
 * References:
 *   https://docs.rs/crc/latest/crc/struct.Algorithm.html
 *   https://stackoverflow.com/a/44560366
 *   https://llllllllll.github.io/c-extension-tutorial/appendix.html
 *   https://www.codeconvert.ai/c-to-rust-converter
 *   https://thepythoncode.com/assistant/code-converter/rust/
 *   https://godbolt.org/
 *   https://mode-s.org/decode/content/ads-b/8-error-control.html
 *   https://github.com/antirez/dump1090/
 *   https://github.com/flightrac/modes-crc/
 */

extern crate hex_slice;
extern crate crc;

use crc::{Crc, Algorithm};
use hex_slice::AsHex;
use log::{debug, error, trace};

pub const MODES_GENERATOR_POLY: u32 = 0x1FFF409;
pub const LONG_MSG_BITS: u8         = 112;
pub const SHORT_MSG_BITS: u8        = 56;

/* Parity table for MODE S Messages.
 * The table contains 112 elements, every element corresponds to a bit set
 * in the message, starting from the first bit of actual data after the
 * preamble.
 *
 * For messages of 112 bit, the whole table is used.
 * For messages of 56 bits only the last 56 elements are used.
 *
 * The algorithm is as simple as xoring all the elements in this table
 * for which the corresponding bit on the message is set to 1.
 *
 * The latest 24 elements in this table are set to 0 as the checksum at the
 * end of the message should not affect the computation.
 *
 * Note: this function can be used with DF11 and DF17, other modes have
 * the CRC xored with the sender address as they are reply to interrogations,
 * but a casual listener can't split the address from the checksum.
 */
static PARITY_TABLE: [u32; 112] = [
    0x3935ea, 0x1c9af5, 0xf1b77e, 0x78dbbf, 0xc397db, 0x9e31e9, 0xb0e2f0, 0x587178,
    0x2c38bc, 0x161c5e, 0x0b0e2f, 0xfa7d13, 0x82c48d, 0xbe9842, 0x5f4c21, 0xd05c14,
    0x682e0a, 0x341705, 0xe5f186, 0x72f8c3, 0xc68665, 0x9cb936, 0x4e5c9b, 0xd8d449,
    0x939020, 0x49c810, 0x24e408, 0x127204, 0x093902, 0x049c81, 0xfdb444, 0x7eda22,
    0x3f6d11, 0xe04c8c, 0x702646, 0x381323, 0xe3f395, 0x8e03ce, 0x4701e7, 0xdc7af7,
    0x91c77f, 0xb719bb, 0xa476d9, 0xadc168, 0x56e0b4, 0x2b705a, 0x15b82d, 0xf52612,
    0x7a9309, 0xc2b380, 0x6159c0, 0x30ace0, 0x185670, 0x0c2b38, 0x06159c, 0x030ace,
    0x018567, 0xff38b7, 0x80665f, 0xbfc92b, 0xa01e91, 0xaff54c, 0x57faa6, 0x2bfd53,
    0xea04ad, 0x8af852, 0x457c29, 0xdd4410, 0x6ea208, 0x375104, 0x1ba882, 0x0dd441,
    0xf91024, 0x7c8812, 0x3e4409, 0xe0d800, 0x706c00, 0x383600, 0x1c1b00, 0x0e0d80,
    0x0706c0, 0x038360, 0x01c1b0, 0x00e0d8, 0x00706c, 0x003836, 0x001c1b, 0xfff409,
    0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000,
    0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000,
    0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000
];

// Calculates the checksum of the data frame passed to it, based on the parity table provided.
// It takes a byte slice `data` and an optional number of bits.
// If the number of bits is not provided, it is determined based on the length of `data`.
pub fn checksum(data: &[u8], bits: Option<u8>) -> u32 {
    let bits = match bits {
        Some(b) => b as usize,
        None => {
            if data.len() * 8 == SHORT_MSG_BITS as usize {
                SHORT_MSG_BITS as usize
            } else if data.len() * 8 == LONG_MSG_BITS as usize {
                LONG_MSG_BITS as usize
            } else {
                return 0 as u32;
            }
        }
    };
    debug!("checksum: bits = {}", bits);

    let offset = if bits == LONG_MSG_BITS as usize {
        0
    } else {
        LONG_MSG_BITS - SHORT_MSG_BITS
    };

    let mut crc = 0;
    for j in 0..bits {
        let b = j / 8;
        let bit = j % 8;
        let bitmask = 1 << (7 - bit);

        if data.get(b).map_or(false, |&byte| byte & bitmask != 0) {
            crc ^= PARITY_TABLE[j + offset as usize];
        }
    }

    crc
}

// Calculates the checksum of the data frame passed to it, based on the parity table provided.
// It takes a byte slice `data` and an optional number of bits.
// If the number of bits is not provided, it is determined based on the length of `data`.
// Returns true if checksum in the message (last 3 bytes) matches the computed checksum, otherwise returns false.
pub fn checksum_compare(data: &[u8], bits: Option<u8>) -> bool {
    let bits = match bits {
        Some(b) => b as usize,
        None => {
            if data.len() * 8 == SHORT_MSG_BITS as usize {
                SHORT_MSG_BITS as usize
            } else if data.len() * 8 == LONG_MSG_BITS as usize {
                LONG_MSG_BITS as usize
            } else {
                return false;
            }
        }
    };
    trace!("checksum_compare: bits = {}", bits);

    let offset = if bits == LONG_MSG_BITS as usize {
        0
    } else {
        LONG_MSG_BITS - SHORT_MSG_BITS
    };

    trace!("checksum_compare: offset = {}", offset);
    let received_checksum = modescrc_buffer_crc(data, Some(bits));

    let mut expected_checksum = 0;
    for j in 0..bits {
        let b = j / 8;
        let bit = j % 8;
        let bitmask = 1 << (7 - bit);

        if data.get(b).map_or(false, |&byte| byte & bitmask != 0) {
            expected_checksum ^= PARITY_TABLE[j + offset as usize];
        }
    }
    trace!("checksum_compare: expected_checksum = {:#02X}", expected_checksum);
    trace!("checksum_compare: received_checksum = {:#02X}", received_checksum);

    // Compare the received checksum with the expected checksum
    received_checksum == expected_checksum 
}

// Extracts the CRC value from a data frame last 3 bytes.
// It takes a byte slice `data` and an optional number of bits.
// If the number of bits is not provided, it defaults to the length of `data` multiplied by 8 (to convert to bits).
pub fn modescrc_buffer_crc(data: &[u8], bits: Option<usize>) -> u32 {
    let bytes = bits.map_or(data.len() * 8, |b| b as usize) / 8;

    // Ensure that there are enough bytes in the data slice to prevent panic due to out-of-bounds access.
    trace!("crc: bytes = {}", bytes);
    if bytes < 3 {
        error!("Data slice is too short to calculate CRC");
        return 0;
    }

    trace!("crc: data = {:#02X}", data.as_hex());
    ((data[bytes - 3] as u32) << 16) | ((data[bytes - 2] as u32) << 8) | (data[bytes - 1] as u32)
}
