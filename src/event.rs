use crate::calendar3;
use crate::utils;
use crate::CALENDAR_HUB;

use calendar3::api::{ConferenceData, ConferenceSolutionKey, EntryPoint, Event, EventDateTime};
use calendar3::chrono::{DateTime, Utc};
use calendar3::hyper::{Body, Response};
use tracing::error;
use tracing::trace;

fn make_meet_event(
    summary: impl Into<String>,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    timezone: Option<impl Into<String> + Clone>,
) -> Event {
    let mut req = Event::default();
    let mut conf_data = ConferenceData::default();
    conf_data.create_request = Some(calendar3::api::CreateConferenceRequest {
        conference_solution_key: Some(ConferenceSolutionKey {
            type_: Some("hangoutsMeet".into()),
        }),
        request_id: Some(utils::unique_id(32)),
        ..Default::default()
    });
    req.conference_data = Some(conf_data);
    req.summary = Some(summary.into());
    req.start = Some(EventDateTime {
        date_time: Some(start_time),
        time_zone: timezone.clone().map(|s| s.into()),
        ..Default::default()
    });
    req.end = Some(EventDateTime {
        date_time: Some(end_time),
        time_zone: timezone.map(|s| s.into()),
        ..Default::default()
    });
    req
}

pub async fn insert_meet_event(
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    summary: &str,
) -> Result<(Response<Body>, Event), anyhow::Error> {
    let req = make_meet_event(summary, start_time, end_time, Some("Asia/Shanghai"));

    let result = CALENDAR_HUB
        .get()
        .unwrap()
        .events()
        .insert(req, "primary")
        .supports_attachments(true)
        .send_notifications(true)
        .conference_data_version(1)
        .doit()
        .await;

    let Ok(res) = result else {
        let e = result.unwrap_err();
        error!("{e}");
        return Err(e)?;
    };

    if !res.0.status().is_success() {
        error!("{:#?}", res);
        return Err(ReqError::FailedCode(res.0.status().as_u16()).into());
    }

    trace!("ok: {res:#?}");

    Ok(res)
}

#[derive(Debug, thiserror::Error)]
enum ReqError {
    #[error("Error code: {0}")]
    FailedCode(u16),
}

pub fn get_meet_link(event: &Event) -> Option<&str> {
    event
        .conference_data
        .as_ref()
        .and_then(|cdata| cdata.entry_points.as_ref())
        .and_then(|entry| {
            entry
                .get(0)
                .and_then(|EntryPoint { uri, .. }| uri.as_deref())
        })
}
