use common::status;

use crate::{airpods_consts::EAR_DETECTION, packets::battery::Pod};

#[derive(Default, Debug)]
pub struct InEarPacket {
    pub primary: EarStatus,
    pub secondary: EarStatus,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum EarStatus {
    InEar = 0x00,
    NotInEar = 0x01,
    InCase = 0x02,
    #[default]
    Disconnected,
}

impl InEarPacket {
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 8 || !bytes.starts_with(EAR_DETECTION) {
            return None;
        }

        Some(InEarPacket {
            primary: EarStatus::from(bytes[6]),
            secondary: EarStatus::from(bytes[7]),
        })
    }

    pub fn get(&self, primary: Pod) -> Option<[EarStatus; 2]> {
        Some(match primary {
            Pod::Left => [self.primary, self.secondary],
            Pod::Right => [self.secondary, self.primary],
            Pod::None => return None,
        })
    }
}

impl EarStatus {
    pub fn from(byte: u8) -> Self {
        match byte {
            0x00 => EarStatus::InEar,
            0x01 => EarStatus::NotInEar,
            0x02 => EarStatus::InCase,
            _ => EarStatus::Disconnected,
        }
    }
}

impl From<EarStatus> for status::EarStatus {
    fn from(val: EarStatus) -> status::EarStatus {
        match val {
            EarStatus::InEar => status::EarStatus::InEar,
            EarStatus::NotInEar => status::EarStatus::NotInEar,
            EarStatus::InCase => status::EarStatus::InCase,
            EarStatus::Disconnected => status::EarStatus::Disconnected,
        }
    }
}
