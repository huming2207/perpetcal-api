use chrono::{DateTime, Utc, Offset, FixedOffset, Local, TimeZone};
use chrono_tz::Tz;
use icalendar::{Calendar, parser::{unfold, read_calendar_simple}, Component, CalendarComponent, Todo, DatePerhapsTime};
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

pub fn ical_dpt_to_dt_string(dpt: DatePerhapsTime, fmt: &str) -> Result<String, PerpetcalError> {
    match dpt {
        DatePerhapsTime::DateTime(cdt) =>  {
            match cdt {
                icalendar::CalendarDateTime::Floating(ts) => return Ok(ts.format(fmt).to_string()),
                icalendar::CalendarDateTime::Utc(ts) => return Ok(ts.format(fmt).to_string()),
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
    pub fn from_todo(todo: Todo) -> CalendarItem {
        let start_ts = match todo.get_start() {
            Some(ts) => {

            }
            None => todo!(),
        }
    }

    pub async fn from_ical_url(url: &str) -> Result<Vec<CalendarItem>, PerpetcalError> {
        let cal_text = fetch_ical(url).await?;
        let generic_component = read_calendar_simple(&cal_text).map_err(|err| PerpetcalError::IcalParseError(err.to_string()))?;
        
        let mut cal_items: Vec<CalendarItem> = Vec::new();
        for item in generic_component {
            match CalendarComponent::from(item) {
                CalendarComponent::Todo(cal_item) =>  {
                    cal_item.
                    cal_items.pus
                }
                CalendarComponent::Event(cal_item) => {

                }
                _ => {}
            }
            
        }

        Ok()

    }
}