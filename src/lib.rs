use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppointmentState {
    Confirmed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationDecision {
    SendConfirmation,
    SendCancellation,
    Skip,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceError {
    MissingApiKey,
    InvalidEnvelope(String),
    Infrai(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Appointment {
    pub id: String,
    pub patient_email: String,
    pub state: AppointmentState,
}

pub fn notification_for(appointment: &Appointment) -> NotificationDecision {
    match appointment.state {
        AppointmentState::Confirmed => NotificationDecision::SendConfirmation,
        AppointmentState::Cancelled => NotificationDecision::SendCancellation,
    }
}

pub fn email_payload(appointment: &Appointment) -> Option<HashMap<&'static str, String>> {
    let subject = match notification_for(appointment) {
        NotificationDecision::SendConfirmation => "Appointment confirmed",
        NotificationDecision::SendCancellation => "Appointment cancelled",
        NotificationDecision::Skip => return None,
    };
    let mut payload = HashMap::new();
    payload.insert("to", appointment.patient_email.clone());
    payload.insert("subject", subject.to_owned());
    payload.insert("body", format!("Appointment {} is now {:?}.", appointment.id, appointment.state));
    Some(payload)
}

pub fn parse_envelope(body: &str) -> Result<(), ServiceError> {
    if body.contains("\"ok\":true") {
        return Ok(());
    }
    if let Some(code) = body.split("\"code\":\"").nth(1).and_then(|v| v.split('\"').next()) {
        return Err(ServiceError::Infrai(code.to_owned()));
    }
    Err(ServiceError::InvalidEnvelope(body.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancelled_appointment_gets_a_patient_safe_notice() {
        let appointment = Appointment {
            id: "apt-42".to_owned(),
            patient_email: "patient@example.test".to_owned(),
            state: AppointmentState::Cancelled,
        };
        assert_eq!(notification_for(&appointment), NotificationDecision::SendCancellation);
        assert_eq!(email_payload(&appointment).unwrap()["subject"], "Appointment cancelled");
    }
}
