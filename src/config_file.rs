use serde::{Deserialize, Serialize};

use crate::actions::CalendarActions;

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigFile {
    pub url: String,
    #[serde(flatten)]
    pub actions: CalendarActions,
}

#[cfg(test)]
mod tests {

    use super::*;

    use calcard::icalendar::{ICalendarComponentType, ICalendarProperty, ICalendarValue};

    use crate::actions::{Action, CalendarActions, EntryFilter, FilterKind, SetAction};

    #[test]
    fn test_serde_de() {
        let input = r##"
{
    "url": "https://campus.kit.edu/sp/webcal/...",
    "actions": [
        {
            "kind": "VEvent",
            "filter": [
                {
                    "name": "SUMMARY",
                    "kind": "starts_with",
                    "value": "2424079"
                }
            ],
            "set": [
                {
                    "name": "SUMMARY",
                    "value": {
                        "type": "Text",
                        "data": "Algo 2"
                    }
                }
            ]
        }
    ]
}
        "##;

        let file: ConfigFile = serde_json::from_str(input).unwrap();

        println!("{:?}", file);
    }

    #[test]
    fn test_serde_ser() {
        let file = ConfigFile {
            url: "https://campus.kit.edu/sp/webcal/...".to_string(),
            actions: CalendarActions {
                actions: vec![Action {
                    kind: ICalendarComponentType::VEvent,
                    filter: vec![EntryFilter {
                        name: ICalendarProperty::Summary,
                        kind: FilterKind::StartsWith {
                            value: "2424079".to_string(),
                        },
                    }],
                    set: vec![SetAction {
                        name: ICalendarProperty::Summary,
                        value: ICalendarValue::Text("Algo 2".to_string()),
                    }],
                }],
            },
        };

        let string = serde_json::to_string_pretty(&file).unwrap();

        println!("{}", string);
    }
}
