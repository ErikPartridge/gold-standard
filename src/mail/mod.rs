use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre::{EmailTransport, SmtpTransport};
use lettre_email::Email;
use lettre_email::EmailBuilder;

use model::submission::Submission;

pub trait Mailer {
    fn build(&self) -> EmailBuilder;

    fn send(&self) -> bool {
        let result = self.build().build();
        let built: Email;
        match result {
            Ok(x) => built = x,
            Err(_) => return false,
        }
        let mut mailer = SmtpTransport::simple_builder(env!("SMTP_HOST"))
            .unwrap()
            // Set the name sent during EHLO/HELO, default is `localhost`
            //.hello_name(ClientId::Domain("my.hostname.tld".to_string()))
            // Add credentials for authentication
            .credentials(Credentials::new(
                env!("SMTP_USERNAME").to_string(),
                env!("SMTP_PASSWORD").to_string(),
            )).smtp_utf8(true)
            // Configure expected authentication mechanism
            .authentication_mechanism(Mechanism::Plain)
            // Enable connection reuse
            .build();

        let sent = mailer.send(&built);
        match sent {
            Ok(_) => return true,
            _ => return false,
        }
    }
}

pub struct NewSubmissionEmail {
    pub identifier: String,
    pub submission: Submission,
}

impl Mailer for NewSubmissionEmail {
    fn build(&self) -> EmailBuilder {
        let subject = format!("Submission: #{}", self.identifier);
        let contents = format!(
            "Hey Erik,\n\n There's a new submission online:\n {:?}",
            self.submission
        );
        let email = EmailBuilder::new()
            .from("erik@erikpartridge.com")
            .to(("erik@erikpartridge.com", "Erik Partridge"))
            .subject(subject)
            .body(contents);
        return email;
    }
}
