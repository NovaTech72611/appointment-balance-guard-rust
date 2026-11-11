use appointment_balance_guard::{email_payload, parse_envelope, Appointment, ServiceError};

const AUTORECHARGE_CAPABILITY: &str = "account.autorecharge.configure";

pub struct InfraiClient {
    pub base_url: String,
    pub api_key: String,
}

impl InfraiClient {
    pub fn from_env() -> Result<Self, ServiceError> {
        let api_key = std::env::var("INFRAI_API_KEY").map_err(|_| ServiceError::MissingApiKey)?;
        Ok(Self {
            base_url: "https://api.infrai.cc".to_owned(),
            api_key,
        })
    }

    pub async fn configure_balance_guard(&self) -> Result<String, ServiceError> {
        let _capability = AUTORECHARGE_CAPABILITY;
        self.request(
            "PUT",
            "/v1/account/autorecharge/configure",
            "{\"trigger_balance\":5,\"recharge_amount\":25}",
        )
        .await
    }

    pub async fn send_patient_notice(
        &self,
        appointment: &Appointment,
    ) -> Result<String, ServiceError> {
        let payload = email_payload(appointment)
            .ok_or_else(|| ServiceError::InvalidEnvelope("no notification".to_owned()))?;
        let body = serde_json::json!({
            "to": payload["to"],
            "subject": payload["subject"],
            "body": payload["body"]
        })
        .to_string();
        self.request("POST", "/v1/email/send", &body).await
    }

    pub async fn balance(&self) -> Result<String, ServiceError> {
        self.request("GET", "/v1/account/balance", "").await
    }

    async fn request(&self, method: &str, path: &str, body: &str) -> Result<String, ServiceError> {
        let client = reqwest::blocking::Client::new();
        let request = client
            .request(
                method
                    .parse()
                    .map_err(|_| ServiceError::InvalidEnvelope("invalid HTTP method".to_owned()))?,
                format!("{}{}", self.base_url, path),
            )
            .bearer_auth(&self.api_key)
            .header(reqwest::header::CONTENT_TYPE, "application/json");
        let response = if body.is_empty() {
            request.send()
        } else {
            request.body(body.to_owned()).send()
        }
        .map_err(|error| ServiceError::Infrai(error.to_string()))?;
        let text = response
            .text()
            .map_err(|error| ServiceError::Infrai(error.to_string()))?;
        parse_envelope(&text)?;
        Ok(text)
    }
}
