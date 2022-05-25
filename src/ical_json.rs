use std::cmp::Ordering;

use chrono::{TimeZone, DateTime, Utc};
use chrono_tz::Tz;
use icalendar::{parser::{unfold, read_calendar_simple}, Component, CalendarComponent, Todo, DatePerhapsTime, Event};
use serde::{Serialize, Deserialize};

use crate::error_type::PerpetcalError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CalendarItem {
    pub start: Option<DateTime<Utc>>,
    pub due: Option<DateTime<Utc>>,
    pub summary: String,
    pub description: Option<String>,
    pub location: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CalendarItemStr {
    pub start: Option<String>,
    pub due: Option<String>,
    pub summary: String,
    pub description: Option<String>,
    pub location: Option<String>,
}

pub async fn fetch_ical(url: &str) -> Result<String, PerpetcalError> {
    let resp = reqwest::get(url).await.map_err(|err| PerpetcalError::FetchError(err))?;
    let raw_text = resp.text().await.map_err(|err| PerpetcalError::FetchError(err))?;
    let unfolded_text = unfold(&raw_text);

    Ok(unfolded_text)
}

pub fn serialize_to_string(item: &Vec<CalendarItem>, fmt: &str, sort: bool, limit: usize) -> Result<Vec<CalendarItemStr>, PerpetcalError> {
    let mut str_item: Vec<CalendarItemStr> = Vec::new();
        let mut ctr: usize = 0;

    if sort {
        let mut sorted_item = item.clone();

        // TODO: This shit should be rewritten soon
        sorted_item.sort_by(|a, b| {
            if !a.due.is_none() && !b.due.is_none() {
                return b.due.unwrap().cmp(&a.due.unwrap());
            } else if !a.start.is_none() && !b.start.is_none() {
                return b.start.unwrap().cmp(&a.start.unwrap());
            } else if !a.due.is_none() && b.due.is_none() {
                return Ordering::Greater;
            } else if a.due.is_none() && !b.due.is_none() {
                return Ordering::Less;
            } else if !a.start.is_none() && b.start.is_none() {
                return Ordering::Greater;
            } else if a.start.is_none() && !b.start.is_none() {
                return Ordering::Less;
            } else {
                return b.summary.cmp(&a.summary)
            }
        });

        for it in &sorted_item {
            if limit != 0 && ctr > limit {
                break;
            }

            let start = match it.start {
                Some(ts) => {
                    Some(ts.format(fmt).to_string())
                }
                None => None,
            };
    
            let due = match it.due {
                Some(ts) => {
                    Some(ts.format(fmt).to_string())
                }
                None => None,
            };

            let fmtted_item = CalendarItemStr {
                start, 
                due,
                summary: it.summary.clone(),
                description: it.description.clone(),
                location: it.location.clone()
            };

            str_item.push(fmtted_item);

            if limit != 0 {
                ctr += 1;
            }
        }
    } else {
        for it in item {
            if limit != 0 && ctr > limit {
                break;
            }
    
            let start = match it.start {
                Some(ts) => {
                    Some(ts.format(fmt).to_string())
                }
                None => None,
            };
    
            let due = match it.due {
                Some(ts) => {
                    Some(ts.format(fmt).to_string())
                }
                None => None,
            };
    
            let fmtted_item = CalendarItemStr {
                start, 
                due,
                summary: it.summary.clone(),
                description: it.description.clone(),
                location: it.location.clone()
            };
    
            str_item.push(fmtted_item);
    
            if limit != 0 {
                ctr += 1;
            }
        }
    }

    Ok(str_item)
}

pub fn ical_dpt_to_dt(dpt: DatePerhapsTime, tzid: String) -> Result<DateTime<Utc>, PerpetcalError> {
    let tz: Tz = tzid.as_str().parse().map_err(|err| PerpetcalError::IcalParseError(format!("Invalid timezone: {}", err)))?;

    match dpt {
        DatePerhapsTime::DateTime(cdt) =>  {
            match cdt {
                icalendar::CalendarDateTime::Floating(ts) => {
                    match tz.from_local_datetime(&ts) {
                        chrono::LocalResult::None => {
                            return Err(PerpetcalError::IcalParseError("Invalid timezone".to_string()));
                        }
                        chrono::LocalResult::Single(dt) => {
                            let dt_utc = dt.with_timezone(&Utc);
                            return Ok(dt_utc);
                        }
                        chrono::LocalResult::Ambiguous(dt, _) => {
                            let dt_utc = dt.with_timezone(&Utc);
                            return Ok(dt_utc);
                        }
                    }

                }
                icalendar::CalendarDateTime::Utc(ts) => {
                    return Ok(ts);
                }
                icalendar::CalendarDateTime::WithTimezone { date_time, tzid } => {
                    let dt = tz.from_utc_datetime(&date_time);
                    let dt_utc = dt.with_timezone(&Utc);
                    return Ok(dt_utc);
                }
            }
        }
        DatePerhapsTime::Date(date) => {
            let dt = tz.from_local_date(&date);
            match dt {
                chrono::LocalResult::None => {
                    return Err(PerpetcalError::IcalParseError("Invalid timezone".to_string()));
                }
                chrono::LocalResult::Single(dt) => {
                    let dt_utc = dt.with_timezone(&Utc);
                    return Ok(dt_utc.and_hms(0, 0, 0));
                }
                chrono::LocalResult::Ambiguous(dt, _) => {
                    let dt_utc = dt.with_timezone(&Utc);
                    return Ok(dt_utc.and_hms(0, 0, 0));
                }
            }
        }
    }
}

impl CalendarItem {
    pub fn from_todo(todo: Todo, tzid: String) -> Result<CalendarItem, PerpetcalError> {
        let start = match todo.get_start() {
            Some(ts) => {
                Some(ical_dpt_to_dt(ts, tzid.clone())?)
            }
            None => {
                match todo.get_timestamp() {
                    Some(ts) => {
                        Some(ts)
                    }
                    None => None,
                }
            }
        };

        let due = match todo.get_end() {
            Some(ts) => Some(ical_dpt_to_dt(ts, tzid.clone())?),
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

    pub fn from_event(event: Event, tzid: String) -> Result<CalendarItem, PerpetcalError> {
        let start = match event.get_start() {
            Some(ts) => {
                Some(ical_dpt_to_dt(ts, tzid.clone())?)
            }
            None => {
                match event.get_timestamp() {
                    Some(ts) => {
                        Some(ts)
                    }
                    None => None,
                }
            }
        };

        let due = match event.get_end() {
            Some(ts) => Some(ical_dpt_to_dt(ts, tzid.clone())?),
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

    pub async fn from_ical_url(url: &str, tzid: String, time_fmt: &str, sort: bool, limit: usize) -> Result<Vec<CalendarItemStr>, PerpetcalError> {
        let cal_text = fetch_ical(url).await?;
        let generic_component = read_calendar_simple(&cal_text).map_err(|err| PerpetcalError::IcalParseError(err.to_string()))?;
        
        let mut cal_items: Vec<CalendarItem> = Vec::new();
        for item in generic_component {
            match CalendarComponent::from(item) {
                CalendarComponent::Todo(cal_item) =>  {
                    cal_items.push(CalendarItem::from_todo(cal_item, tzid.clone())?);
                }
                CalendarComponent::Event(cal_item) => {
                    cal_items.push(CalendarItem::from_event(cal_item, tzid.clone())?);
                }
                _ => {}
            }
        }

        let serialised_items = serialize_to_string(&cal_items, time_fmt, sort, limit)?;
        Ok(serialised_items)
    }
}
