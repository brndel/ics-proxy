use calcard::icalendar::{ICalendar, ICalendarComponentType};
use serde::Deserialize;

use crate::actions::{CalendarActions, EventEntry};

#[derive(Deserialize, Debug)]
pub struct ConfigFile {
    pub url: String,
    #[serde(rename = "event", default)]
    pub events: Vec<EventEntry>,
    #[serde(default)]
    pub calendar: CalendarActions,
}

impl ConfigFile {
    pub fn apply(&self, calendar: &mut ICalendar) {
        for component in &mut calendar.components {
            match component.component_type {
                ICalendarComponentType::VCalendar => {
                    self.calendar.apply(component);
                }
                ICalendarComponentType::VEvent => {
                    for event in &self.events {
                        event.apply(component);
                    }
                },
                _ => (),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_de() {
        let input = r##"
url = "https://campus.kit.edu/sp/webcal/..."

[[event]]
filter.summary.starts_with = "42679"
action.summary.set = "SWT 2"

[[event]]
filter.summary.starts_with = "2424638"
action.summary.set = "Routen Algo"

[[event]]
filter.summary.starts_with = "24679"
action.summary.set = "Inter CGI"
        "##;

        let file: ConfigFile = toml::from_str(input).unwrap();

        println!("{:?}", file);
    }
}
