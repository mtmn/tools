use std::fmt::Write as _;
use std::io::{Write, stdout};

use serde::Serialize;

use crate::status::{BatteryStatus, Components, Status};

#[derive(Serialize, Debug)]
pub struct Output {
    text: String,
    tooltip: Option<String>,
    class: Option<String>,
    percentage: Option<f32>,
}

impl Output {
    #[must_use]
    pub fn not_connected() -> Self {
        Output {
            text: "󰟦".into(),
            tooltip: Some("Daemon not active".into()),
            class: Some("disconnected".into()),
            percentage: None,
        }
    }

    #[must_use]
    pub fn from_status(status: &Status) -> Self {
        if !status.is_valid() {
            return Output::default();
        }

        let mut tooltip = String::new();
        if let Some(metadata) = &status.metadata {
            let _ = writeln!(tooltip, "{} ({})", metadata.name, metadata.model);
        }

        let Components { left, right, case } = &status.components;
        for (idx, (name, component)) in [("Left", left), ("Right", right), ("Case", case)]
            .iter()
            .enumerate()
        {
            let Some(component) = component else {
                continue;
            };

            let icon = match component.status {
                BatteryStatus::Charging => "󰢝",
                BatteryStatus::Discharging => match idx {
                    0 => status.ear.left,
                    1 => status.ear.right,
                    _ => crate::status::EarStatus::Disconnected,
                }
                .icon(),
                BatteryStatus::Disconnected => continue,
            };

            let _ = writeln!(tooltip, "{icon} {name}: {}%", component.level);
        }

        for device in &status.devices {
            let icon = if let Some(text) = &device.text {
                text.as_str()
            } else {
                match device.status {
                    BatteryStatus::Charging => "󰢝",
                    BatteryStatus::Discharging => "󰂯",
                    BatteryStatus::Disconnected => continue,
                }
            };
            let _ = writeln!(tooltip, "{icon} {}: {}%", device.name, device.battery);
        }

        let mut min_level = status.min_pods();
        for device in &status.devices {
            if device.status == BatteryStatus::Discharging {
                min_level = min_level.min(device.battery);
            }
        }

        let is_low = min_level <= 15;
        let class = ["connected", "connected-low"][usize::from(is_low)];

        // Base text for pods
        let mut text_parts = Vec::new();

        let battery = ["", "󱃍"][usize::from(is_low)];
        if status.min_pods() != u8::MAX {
            let min_pods = status.min_pods();
            text_parts.push(format!("󱡏{battery} {min_pods}%"));
        }

        for device in &status.devices {
            let icon = if let Some(text) = &device.text {
                text.as_str()
            } else {
                match device.status {
                    BatteryStatus::Charging => "󰢝",
                    BatteryStatus::Discharging => "󰂯",
                    BatteryStatus::Disconnected => continue,
                }
            };
            text_parts.push(format!("{icon} {}%", device.battery));
        }

        let text = if text_parts.is_empty() {
            // Default empty/disconnected state
            format!("󱡏{battery}")
        } else {
            text_parts.join(" ")
        };

        Output {
            text,
            tooltip: Some(tooltip[..tooltip.len() - 1].to_owned()),
            class: Some(class.into()),
            percentage: if min_level == u8::MAX {
                None
            } else {
                Some(f32::from(min_level) / 100.0)
            },
        }
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn print(&self) {
        let str = serde_json::to_string(&self).unwrap();

        let mut stdout = stdout();
        let _ = stdout.write_fmt(format_args!("{str}\n"));
        let _ = stdout.flush();
    }
}

impl Default for Output {
    fn default() -> Self {
        Output {
            text: "󱡐".into(),
            tooltip: None,
            class: Some("disconnected".into()),
            percentage: None,
        }
    }
}
