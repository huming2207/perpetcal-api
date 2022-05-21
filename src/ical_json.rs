use chrono::{Local, TimeZone};
use chrono_tz::Tz;
use icalendar::{parser::{unfold, read_calendar_simple}, Component, CalendarComponent, Todo, DatePerhapsTime, Event};
use serde::{Serialize, Deserialize};

use crate::error_type::PerpetcalError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CalendarItem {
    start: Option<String>,
    due: Option<String>,
    summary: String,
    description: Option<String>,
    location: Option<String>,
}

pub async fn fetch_ical(url: &str) -> Result<String, PerpetcalError> {
    let resp = reqwest::get(url).await.map_err(|err| PerpetcalError::FetchError(err))?;
    let raw_text = resp.text().await.map_err(|err| PerpetcalError::FetchError(err))?;
    let unfolded_text = unfold(&raw_text);

    Ok(unfolded_text)
}

pub fn ical_dpt_to_dt_string(dpt: DatePerhapsTime, fmt: &str, tzid: String) -> Result<String, PerpetcalError> {
    match dpt {
        DatePerhapsTime::DateTime(cdt) =>  {
            match cdt {
                icalendar::CalendarDateTime::Floating(ts) => return Ok(ts.format(fmt).to_string()),
                icalendar::CalendarDateTime::Utc(ts) => {
                    let tz: Tz = tzid.parse().map_err(|err| PerpetcalError::IcalParseError("Invalid timezone".to_string()))?;
                    return Ok(ts.with_timezone(&tz).format(fmt).to_string())
                }
                icalendar::CalendarDateTime::WithTimezone { date_time, tzid } => {
                    let tz: Tz = tzid.as_str().parse().map_err(|err| PerpetcalError::IcalParseError("Timezone error".to_string()))?;
                    let dt = tz.from_utc_datetime(&date_time);
                    let dt_utc = dt.with_timezone(&Local);
                    return Ok(dt_utc.format(fmt).to_string());
                }
            }
        }
        DatePerhapsTime::Date(date) => {
            return Ok(date.format(fmt).to_string());
        }
    }
}

impl CalendarItem {
    pub fn from_todo(todo: Todo, tzid: String, fmt: &str) -> Result<CalendarItem, PerpetcalError> {
        let start = match todo.get_start() {
            Some(ts) => {
                Some(ical_dpt_to_dt_string(ts, "%H:%M, %m %h", tzid.clone())?)
            }
            None => {
                match todo.get_timestamp() {
                    Some(ts) => {
                        let tz: Tz = tzid.parse().map_err(|err| PerpetcalError::IcalParseError(format!("Invalid timezone: {}", err)))?;
                        Some(ts.with_timezone(&tz).format(fmt).to_string())
                    }
                    None => None,
                }
            }
        };

        let due = match todo.get_end() {
            Some(ts) => Some(ical_dpt_to_dt_string(ts, "%H:%M, %m %h", tzid.clone())?),
            None => None,
        };

        let summary = match todo.get_summary() {
            Some(str) => str.to_string(),
            None => "Untitled TODO".to_string(),
        };

        let description = match todo.get_description() {
            Some(str) => Some(str.to_string()),
            None => None,
        };

        let location = match todo.get_location() {
            Some(str) => Some(str.to_string()),
            None => None,
        };

        Ok(CalendarItem { start, due, summary, description, location })
    }

    pub fn from_event(event: Event, tzid: String, fmt: &str) -> Result<CalendarItem, PerpetcalError> {
        let start = match event.get_start() {
            Some(ts) => {
                Some(ical_dpt_to_dt_string(ts, fmt, tzid.clone())?)
            }
            None => {
                match event.get_timestamp() {
                    Some(ts) => {
                        let tz: Tz = tzid.parse().map_err(|err| PerpetcalError::IcalParseError(format!("Invalid timezone: {}", err)))?;
                        Some(ts.with_timezone(&tz).format(fmt).to_string())
                    }
                    None => None,
                }
            }
        };

        let due = match event.get_end() {
            Some(ts) => Some(ical_dpt_to_dt_string(ts, fmt, tzid.clone())?),
            None => None,
        };

        let summary = match event.get_summary() {
            Some(str) => str.to_string(),
            None => "Untitled TODO".to_string(),
        };

        let description = match event.get_description() {
            Some(str) => Some(str.to_string()),
            None => None,
        };

        let location = match event.get_location() {
            Some(str) => Some(str.to_string()),
            None => None,
        };

        Ok(CalendarItem { start, due, summary, description, location })
    }

    pub async fn from_ical_url(url: &str, tzid: String, time_fmt: &str) -> Result<Vec<CalendarItem>, PerpetcalError> {
        let cal_text = fetch_ical(url).await?;
        let generic_component = read_calendar_simple(&cal_text).map_err(|err| PerpetcalError::IcalParseError(err.to_string()))?;
        
        let mut cal_items: Vec<CalendarItem> = Vec::new();
        for item in generic_component {
            match CalendarComponent::from(item) {
                CalendarComponent::Todo(cal_item) =>  {
                    cal_items.push(CalendarItem::from_todo(cal_item, tzid.clone(), time_fmt)?);
                }
                CalendarComponent::Event(cal_item) => {
                    cal_items.push(CalendarItem::from_event(cal_item, tzid.clone(), time_fmt)?);
                }
                _ => {}
            }
            
        }

        Ok(cal_items)
    }
}