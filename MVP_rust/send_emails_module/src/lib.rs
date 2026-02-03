pub mod send_emails;

pub use send_emails::{
    send_email, SendEmailError, SendEmailRequest, SendEmailResponse, DEFAULT_POSTMARK_API_BASE,
};
