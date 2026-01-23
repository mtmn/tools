use common::status;

use crate::airpods_consts::BATTERY_STATUS;

#[derive(Default, Debug)]
pub struct BatteryPacket {
    pub left: Option<ComponentStatus>,
    pub right: Option<ComponentStatus>,
    pub case: Option<ComponentStatus>,
    pub primary: Pod,
}

#[derive(Debug, Clone, Copy)]
pub struct ComponentStatus {
    pub level: u8,
    pub status: BatteryStatus,
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Pod {
    Left,
    Right,
    #[default]
    None,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum BatteryStatus {
    Charging = 0x01,
    Discharging = 0x02,
    Disconnected = 0x04,
}

#[derive(Debug)]
#[repr(u8)]
enum Component {
    Right = 0x02,
    Left = 0x04,
    Case = 0x08,
}

impl BatteryPacket {
    pub fn parse(raw: &[u8]) -> Option<Self> {
        if !raw.starts_with(BATTERY_STATUS) || raw.len() <= 6 {
            return None;
        }

        let components = raw[6] as usize;
        if components > 3 || raw.len() != 7 + 5 * components {
            return None;
        }

        let mut out = Self::default();
        for i in 0..components {
            let i = 7 + (5 * i);
            if raw[i + 1] != 0x01 || raw[i + 4] != 0x01 {
                continue;
            }

            let component_type = Component::from(raw[i])?;
            let level = raw[i + 2];
            let status = BatteryStatus::from(raw[i + 3])?;

            if matches!(out.primary, Pod::None) {
                out.primary = component_type.as_pod();
            }

            let component_status = Some(ComponentStatus { level, status });
            match component_type {
                Component::Left => out.left = component_status,
                Component::Right => out.right = component_status,
                Component::Case => out.case = component_status,
            }
        }

        Some(out)
    }

    pub fn as_arr(&self) -> [&Option<ComponentStatus>; 3] {
        [&self.left, &self.right, &self.case]
    }
}

impl Component {
    pub fn from(byte: u8) -> Option<Self> {
        match byte {
            0x02 => Some(Self::Right),
            0x04 => Some(Self::Left),
            0x08 => Some(Self::Case),
            _ => None,
        }
    }

    pub fn as_pod(&self) -> Pod {
        match self {
            Component::Left => Pod::Left,
            Component::Right => Pod::Right,
            Component::Case => Pod::None,
        }
    }
}

impl BatteryStatus {
    pub fn from(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(Self::Charging),
            0x02 => Some(Self::Discharging),
            0x04 => Some(Self::Disconnected),
            _ => None,
        }
    }
}

impl From<ComponentStatus> for status::ComponentStatus {
    fn from(val: ComponentStatus) -> status::ComponentStatus {
        status::ComponentStatus {
            level: val.level,
            status: val.status.into(),
        }
    }
}

impl From<BatteryStatus> for status::BatteryStatus {
    fn from(val: BatteryStatus) -> status::BatteryStatus {
        match val {
            BatteryStatus::Charging => status::BatteryStatus::Charging,
            BatteryStatus::Discharging => status::BatteryStatus::Discharging,
            BatteryStatus::Disconnected => status::BatteryStatus::Disconnected,
        }
    }
}
