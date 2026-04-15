use std::collections::HashMap;

use calcard::icalendar::{ICalendarComponent, ICalendarProperty, ICalendarValue};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct EventEntry {
    #[serde(rename = "filter", default)]
    filters: HashMap<EventProperty, EventFilter>,
    #[serde(rename = "action", default)]
    actions: HashMap<EventProperty, EventAction>,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EventProperty {
    Summary,
    Location,
    Description,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum EventFilter {
    Equals(String),
    StartsWith(String),
    Contains(String),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum EventAction {
    Set(String),
}

impl EventEntry {
    pub fn apply(&self, event: &mut ICalendarComponent) {
        let passes_filter = self
            .filters
            .iter()
            .any(|(property, filter)| filter.matches(property, event));

        if passes_filter {
            for (property, action) in &self.actions {
                action.apply(property, event);
            }
        }
    }
}

impl EventProperty {
    pub fn ical_property(&self) -> ICalendarProperty {
        match self {
            EventProperty::Summary => ICalendarProperty::Summary,
            EventProperty::Location => ICalendarProperty::Location,
            EventProperty::Description => ICalendarProperty::Description,
        }
    }
}

impl EventFilter {
    pub fn matches(&self, property: &EventProperty, event: &ICalendarComponent) -> bool {
        let Some(entry) = event.property(&property.ical_property()) else {
            println!("event {:?} does not have prop {:?}", event.uid(), property);
            return false;
        };

        let Some(ICalendarValue::Text(event_value)) = entry.values.first() else {
            println!("prop {:?} is not of type text", property);
            return false;
        };

        match &self {
            EventFilter::Equals(value) => event_value == value,
            EventFilter::StartsWith(value) => event_value.starts_with(value),
            EventFilter::Contains(value) => event_value.contains(value),
        }
    }
}

impl EventAction {
    pub fn apply(&self, property: &EventProperty, event: &mut ICalendarComponent) {
        match self {
            EventAction::Set(value) => {
                set_or_add_property(
                    event,
                    &property.ical_property(),
                    ICalendarValue::Text(value.clone()),
                );
            }
        }
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct CalendarActions {
    prodid: Option<String>,
    url: Option<String>,
}

impl CalendarActions {
    pub fn apply(&self, calendar: &mut ICalendarComponent) {
        if let Some(prodid) = &self.prodid {
            set_or_add_property(
                calendar,
                &ICalendarProperty::Prodid,
                ICalendarValue::Text(prodid.clone()),
            );
        }
        if let Some(url) = &self.url {
            set_or_add_property(
                calendar,
                &ICalendarProperty::Url,
                ICalendarValue::Text(url.clone()),
            );
        }
    }
}

fn set_or_add_property(
    component: &mut ICalendarComponent,
    property: &ICalendarProperty,
    value: ICalendarValue,
) {
    if let Some(param) = component.property_mut(property) {
        param.values = vec![value];
    } else {
        component.add_property(property.clone(), value);
    }
}
