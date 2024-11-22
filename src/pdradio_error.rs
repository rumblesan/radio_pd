use std::error;
use std::fmt;

use shout;

#[derive(Debug)]
pub struct PDRadioError(pub String);

impl fmt::Display for PDRadioError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}
impl error::Error for PDRadioError {}

impl Into<PDRadioError> for shout::ShoutErr {
    fn into(self) -> PDRadioError {
        match self {
            shout::ShoutErr::Success => PDRadioError("Shout: Success".into()),
            shout::ShoutErr::Insane => PDRadioError("Shout: Insane".into()),
            shout::ShoutErr::NoConnect => PDRadioError("Shout: NoConnect".into()),
            shout::ShoutErr::NoLogin => PDRadioError("Shout: NoLogin".into()),
            shout::ShoutErr::Socket => PDRadioError("Shout: Socket".into()),
            shout::ShoutErr::Malloc => PDRadioError("Shout: Malloc".into()),
            shout::ShoutErr::Metadata => PDRadioError("Shout: Metadata".into()),
            shout::ShoutErr::Connected => PDRadioError("Shout: Connected".into()),
            shout::ShoutErr::Unconnected => PDRadioError("Shout: Unconnected".into()),
            shout::ShoutErr::Unsupported => PDRadioError("Shout: Unsupported".into()),
            shout::ShoutErr::Busy => PDRadioError("Shout: Busy".into()),
            shout::ShoutErr::NoTLS => PDRadioError("Shout: NoTLS".into()),
            shout::ShoutErr::TLSBadCert => PDRadioError("Shout: TLSBadCert".into()),
            shout::ShoutErr::Retry => PDRadioError("Shout: Retry".into()),
        }
    }
}

impl Into<PDRadioError> for shout::ShoutConnError {
    fn into(self) -> PDRadioError {
        match self {
            shout::ShoutConnError::ShoutError(err) => err.into(),
            shout::ShoutConnError::NulError(..) => PDRadioError("Shout: NulError".into()),
        }
    }
}
