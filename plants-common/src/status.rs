use serde::{Deserialize, Serialize};
use std::hash::{DefaultHasher, Hash, Hasher};
use zbus::zvariant::Type;

#[derive(Default, Hash, Clone, Debug, Serialize, Deserialize, Type)]
pub struct Status {
    pub metadata: Option<Metadata>,
    pub components: Components,
    pub ear: InEar,
    pub devices: Vec<GenericDeviceStatus>,
}

#[derive(Hash, Debug, Clone, Serialize, Deserialize, Type)]
pub struct GenericDeviceStatus {
    pub name: String,
    pub battery: u8,
    pub text: Option<String>,
    pub status: BatteryStatus,
}

#[derive(Hash, Clone, Debug, Serialize, Deserialize, Type)]
pub struct Metadata {
    pub name: String,
    pub model: String,
}

#[derive(Default, Hash, Clone, Debug, Serialize, Deserialize, Type)]
pub struct Components {
    pub left: Option<ComponentStatus>,
    pub right: Option<ComponentStatus>,
    pub case: Option<ComponentStatus>,
}

#[derive(Default, Hash, Clone, Debug, Serialize, Deserialize, Type)]
pub struct InEar {
    pub left: EarStatus,
    pub right: EarStatus,
}

#[derive(Hash, Clone, Debug, Serialize, Deserialize, Type)]
pub struct ComponentStatus {
    pub level: u8,
    pub status: BatteryStatus,
}

#[derive(Hash, Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Type)]
pub enum BatteryStatus {
    Charging,
    Discharging,
    Disconnected,
}

#[derive(Default, Hash, Clone, Copy, Debug, Serialize, Deserialize, Type)]
pub enum EarStatus {
    InEar,
    NotInEar,
    InCase,
    #[default]
    Disconnected,
}

impl Status {
    #[must_use]
    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        Hash::hash(self, &mut hasher);
        hasher.finish()
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        let Components { left, right, case } = &self.components;
        left.is_some() || right.is_some() || case.is_some() || !self.devices.is_empty()
    }

    #[must_use]
    pub fn min_pods(&self) -> u8 {
        let mut out = u8::MAX;

        let Components { left, right, .. } = &self.components;
        for component in [&left, &right] {
            if let Some(component) = &component
                && matches!(component.status, BatteryStatus::Discharging)
            {
                out = out.min(component.level);
            }
        }

        out
    }
}

impl Components {
    pub fn as_arr_mut(&mut self) -> [&mut Option<ComponentStatus>; 3] {
        [&mut self.left, &mut self.right, &mut self.case]
    }
}

impl EarStatus {
    #[must_use]
    pub fn icon(&self) -> &'static str {
        match self {
            EarStatus::InEar => "󱡏",
            EarStatus::NotInEar => "󱡒",
            EarStatus::InCase => "󱡑",
            EarStatus::Disconnected => "",
        }
    }
}
