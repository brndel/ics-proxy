use calcard::{
    common::IanaParse,
    icalendar::{
        ICalendar, ICalendarComponent, ICalendarComponentType, ICalendarProperty, ICalendarValue,
    },
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize, Debug)]
pub struct EventFilter {
    #[serde(serialize_with = "ser_calendar_prop_as_str", deserialize_with = "de_calendar_prop_as_str")]
    pub name: ICalendarProperty,
    #[serde(flatten)]
    pub kind: FilterKind,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind")]
#[serde(rename_all = "snake_case")]
pub enum FilterKind {
    Equals { value: ICalendarValue },
    StartsWith { value: String },
    Contains { value: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SetAction {
    #[serde(serialize_with = "ser_calendar_prop_as_str", deserialize_with = "de_calendar_prop_as_str")]
    pub name: ICalendarProperty,
    pub value: ICalendarValue,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Action {
    pub filter: Vec<EventFilter>,
    pub set: Vec<SetAction>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CalendarActions {
    pub actions: Vec<Action>,
}

impl CalendarActions {
    pub fn apply(&self, event: &mut ICalendarComponent) -> bool {
        let mut changed_anything = false;

        for action in &self.actions {
            let apply_replacements = action.filter.iter().any(|filter| filter.matches(event));

            if apply_replacements {
                for replacement in &action.set {
                    replacement.apply(event);
                }

                changed_anything = true;
            }
        }

        return changed_anything;
    }

    pub fn apply_to_events(&self, cal: &mut ICalendar) {
        for event in &mut cal.components {
            if event.component_type == ICalendarComponentType::VEvent {
                self.apply(event);
            }
        }
    }
}

impl EventFilter {
    pub fn matches(&self, event: &ICalendarComponent) -> bool {
        let Some(entry) = event.property(&self.name) else {
            return false;
        };

        let Some(event_value) = entry.values.first() else {
            return false;
        };

        match &self.kind {
            FilterKind::Equals { value } => event_value == value,
            FilterKind::StartsWith { value } => event_value
                .as_text()
                .is_some_and(|event_value| event_value.starts_with(value)),
            FilterKind::Contains { value } => event_value
                .as_text()
                .is_some_and(|event_value| event_value.contains(value)),
        }
    }
}

impl SetAction {
    pub fn apply(&self, event: &mut ICalendarComponent) {
        set_or_add_property(event, &self.name, self.value.clone());
    }
}

fn set_or_add_property(
    event: &mut ICalendarComponent,
    name: &ICalendarProperty,
    value: ICalendarValue,
) {
    if let Some(param) = event.property_mut(name) {
        param.values = vec![value];
    } else {
        event.add_property(name.clone(), value);
    }
}

fn ser_calendar_prop_as_str<S: Serializer>(value: &ICalendarProperty, ser: S) -> Result<S::Ok, S::Error> {
    ser.serialize_str(value.as_str())
}

fn de_calendar_prop_as_str<'de, D: Deserializer<'de>>(de: D) -> Result<ICalendarProperty, D::Error> {
    let value = String::deserialize(de)?;

    ICalendarProperty::parse(value.as_bytes())
        .ok_or_else(|| <D::Error as serde::de::Error>::custom("foo"))
}
