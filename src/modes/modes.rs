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