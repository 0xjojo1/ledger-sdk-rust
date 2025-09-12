use std::ops::Deref;

pub use async_trait::async_trait;
pub use ledger_apdu::{APDUAnswer, APDUCommand, APDUErrorCode};

/// Use to talk to the ledger device
#[async_trait]
pub trait Exchange {
    /// Error defined by Transport used
    type Error;

    /// The concrete type containing the APDUAnswer
    type AnswerType: Deref<Target = [u8]> + Send;

    /// Send a command with the given transport and retrieve an answer or a transport error
    async fn exchange<I>(
        &self,
        command: &APDUCommand<I>,
    ) -> Result<APDUAnswer<Self::AnswerType>, Self::Error>
    where
        I: Deref<Target = [u8]> + Send + Sync;
}
