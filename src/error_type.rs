use thiserror::Error;

#[derive(Error, Debug)]
pub enum PerpetcalError {
    #[error("Failed to fetch iCalendar feed, reason: {0}")]
    FetchError(reqwest::Error),

    #[error("Failed to parse iCalendar: {0}")]
    IcalParseError(String)
}