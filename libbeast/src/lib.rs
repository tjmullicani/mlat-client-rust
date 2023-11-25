use std::io::{Error, ErrorKind};
use adsb_deku::deku::DekuContainerRead;

// pub struct Frames {
//     pub frames: Vec<Frame>,
// }

// pub struct Frame {
//     pub message_type: u8,
//     pub timestamp: u64,
//     pub signal: u8,
//     //pub data: String,
//     pub data: Option<adsb_deku::Frame>,
//     pub hex: String,
// }

// impl Frame {
//     fn to_string(&self) -> String {
//         format!("Type: {},\n Timestamp: {},\n Signal: {}", self.message_type, self.timestamp, self.signal)
//     }
// }

// impl Default for Frame {
//     fn default() -> Self {
//         Frame {
//             message_type: 0,
//             timestamp: 0,
//             signal: 0,
//             data: None,
//             hex: String::new(),
//         }
//     }
// }

impl Frames {
    pub fn to_string(&self) -> String {
        // Create a string representation of the frames
        let frames_str: String = self.frames.iter()
            .map(|frame| {
                format!(
                    " Message Type: {:02X},\n Timestamp: {},\n Signal: {:02X},\n Data: \n ---\n{}",
                    frame.message_type,
                    frame.timestamp,
                    frame.signal,
                    frame.data.as_ref().unwrap().to_string(),
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        // Combine the buffer and frames into a single string
        format!("Frames:\n{}", frames_str)
    }
}

impl Default for Frames {
    fn default() -> Self {
        Frames {
            frames: Vec::new(),
        }
    }
}

// based on https://github.com/junzis/pyModeS/blob/master/pyModeS/extra/tcpclient.py#L65
/// Handle mode-s beast data type.
///
/// <esc> "1" : 6 byte MLAT timestamp, 1 byte signal level,
///     2 byte Mode-AC
/// <esc> "2" : 6 byte MLAT timestamp, 1 byte signal level,
///     7 byte Mode-S short frame
/// <esc> "3" : 6 byte MLAT timestamp, 1 byte signal level,
///     14 byte Mode-S long frame
/// <esc> "4" : 6 byte MLAT timestamp, status data, DIP switch
///     configuration settings (not on Mode-S Beast classic)
/// <esc><esc>: true 0x1a
/// <esc> is 0x1a, and "1", "2" and "3" are 0x31, 0x32 and 0x33
///
/// timestamp:
/// wiki.modesbeast.com/Radarcape:Firmware_Versions#The_GPS_timestamp
pub fn read_single_frame(mut buffer: Vec<u8>) -> Result<Frame, Error> {
    let mut msg: Vec<u8> = Vec::new();
    let mut iter =  buffer.iter().peekable();
    while let Some(&byte) = iter.next() {
        match byte {
            0x1A if iter.peek() == Some(&&0x1A) => {
                // If the current and next bytes are 0x1A, append one 0x1A to `msg`
                msg.push(0x1A);
                iter.next(); // Skip the next byte as it's part of the escape sequence
            }
            0x1A if iter.peek().is_none() => {
                // Special case where the last byte is 0x1A
                msg.push(0x1A);
            }
            0x1A if iter.peek() == Some(&&0x31) => {
                // Start new frame
                if !msg.is_empty() { msg.clear(); }
            }
            0x1A if iter.peek() == Some(&&0x32) => {
                // Start new frame
                if !msg.is_empty() { msg.clear(); }
            }
            0x1A if iter.peek() == Some(&&0x33) => {
                // Start new frame
                if !msg.is_empty() { msg.clear(); }
            }
            _ => {
                // Otherwise, append the current byte to `msg`
                msg.push(byte);
            }
        }
    }

    // Extract messages
    let msgtype = msg[0];

    let ms: Vec<u8> = match msgtype {
        0x31 => {
            let _ = msg.len() != 10 && return Err(Error::new(ErrorKind::UnexpectedEof, format!("invalid message: expected 11 bytes, received {}", msg.len() + 1)));
            msg[8..10].to_vec()
        },
        0x32 => {
            let _ = msg.len() != 15 && return Err(Error::new(ErrorKind::UnexpectedEof, format!("invalid message: expected 16 bytes, received {}", msg.len() + 1)));
            msg[8..15].to_vec()
        },
        0x33 => {
            let _ = msg.len() != 22 && return Err(Error::new(ErrorKind::UnexpectedEof, format!("invalid message: expected 23 bytes, received {}", msg.len() + 1)));
            msg[8..22].to_vec()
        },
        _ => return Err(Error::new(ErrorKind::UnexpectedEof, format!("invalid message: message type {:#02X} is not one of: [0x,31, 0x32, 0x33]", msgtype))),
    };

    let mut frame = Frame::default();
    frame.message_type = msgtype;

    frame.timestamp = u64::from_be_bytes([
        0, 0, // Pad to 8 bytes
        msg[1],
        msg[2],
        msg[3],
        msg[4],
        msg[5],
        msg[6],
    ]);

    frame.signal = msg[7];
    frame.data = Some(adsb_deku::Frame::from_bytes((&ms, 0)).unwrap().1);
    frame.hex = std::iter::once(0x1A).chain(msg.iter().copied()).map(|b| format!("{:02X}", b)).collect::<String>();

    Ok(frame)
}
pub fn read_beast_buffer(mut buffer: Vec<u8>) -> Result<Frames, Error> {
    let mut error: Option<Error> = None;
    let mut messages_mlat: Vec<Vec<u8>> = Vec::new();
    let mut msg: Vec<u8> = Vec::new();
    let mut iter =  buffer.iter().peekable();

    // process the buffer until the last divider <esc> 0x1a
    // then, reset the self.buffer with the remainder
    while let Some(&byte) = iter.next() {
        match byte {
            0x1A if iter.peek() == Some(&&0x1A) => {
                // If the current and next bytes are 0x1A, append one 0x1A to `msg`
                msg.push(0x1A);
                iter.next(); // Skip the next byte as it's part of the escape sequence
            }
            0x1A if iter.peek().is_none() => {
                // Special case where the last byte is 0x1A
                msg.push(0x1A);
            }
            0x1A if iter.peek() == Some(&&0x31) => {
                // Start new frame
                if !msg.is_empty() { messages_mlat.push(msg.clone()); msg.clear(); }
            }
            0x1A if iter.peek() == Some(&&0x32) => {
                // Start new frame
                if !msg.is_empty() { messages_mlat.push(msg.clone()); msg.clear(); }
            }
            0x1A if iter.peek() == Some(&&0x33) => {
                // Start new frame
                if !msg.is_empty() { messages_mlat.push(msg.clone()); msg.clear(); }
            }
            _ => {
                // Otherwise, append the current byte to `msg`
                msg.push(byte);
            }
        }
    }

    // Save the remander for the next reading cycle, if not empty
    if !msg.is_empty() {
        let mut reminder = Vec::new();
        for (i, &m) in msg.iter().enumerate() {
            if m == 0x1A && i < msg.len() - 1 {
                reminder.extend_from_slice(&[m, m]);
            } else {
                reminder.push(m);
            }
        }
        buffer = std::iter::once(0x1A).chain(reminder).collect();
    } else {
        buffer.clear();
    }

    // Extract messages
    let mut frames: Frames = Frames { frames: Vec::new() };
    for mm in messages_mlat {
        let msgtype = mm[0];

        let ms: Vec<u8> = match msgtype {
            0x31 => {
                let _ = mm.len() != 10 && return Err(Error::new(ErrorKind::UnexpectedEof, format!("invalid message: expected 11 bytes, received {}", mm.len() + 1)));
                mm[8..10].to_vec()
            },
            0x32 => {
                let _ = mm.len() != 15 && return Err(Error::new(ErrorKind::UnexpectedEof, format!("invalid message: expected 16 bytes, received {}", mm.len() + 1)));
                mm[8..15].to_vec()
            },
            0x33 => {
                let _ = mm.len() != 22 && return Err(Error::new(ErrorKind::UnexpectedEof, format!("invalid message: expected 23 bytes, received {}", mm.len() + 1)));
                mm[8..22].to_vec()
            },
            _ => {
                error = Some(Error::new(ErrorKind::UnexpectedEof, format!("invalid message: message type {:#02X} is not one of: [0x,31, 0x32, 0x33]", msgtype)));
                continue;
            },
        };

        let mut frame = Frame::default();
        frame.message_type = msgtype;

        frame.timestamp = u64::from_be_bytes([
            0, 0, // Pad to 8 bytes
            mm[1],
            mm[2],
            mm[3],
            mm[4],
            mm[5],
            mm[6],
        ]);

        frame.signal = mm[7];
        frame.data = Some(adsb_deku::Frame::from_bytes((&ms, 0)).unwrap().1);
        frame.hex = std::iter::once(0x1A).chain(msg.iter().copied()).map(|b| format!("{:02X}", b)).collect::<String>();
        frames.frames.push(frame);
    }

    if let Some(err) = error {
        Err(err)
    } else {
        Ok(frames)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
