// SPDX-License-Identifier: Apache-2.0

//! GET APP CONFIGURATION command implementation

use async_trait::async_trait;
use ledger_sdk_device_base::{App, AppExt};
use ledger_sdk_transport::{APDUCommand, Exchange};

use crate::errors::{EthAppError, EthAppResult};
use crate::instructions::ins;
use crate::types::{AppConfiguration, AppVersion, ConfigFlags};
use crate::EthApp;

#[async_trait]
pub trait GetConfiguration<E>
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    /// Get Ethereum application configuration
    async fn get_configuration(transport: &E) -> EthAppResult<AppConfiguration, E::Error>;
}

#[async_trait]
impl<E> GetConfiguration<E> for EthApp
where
    E: Exchange + Send + Sync,
    E::Error: std::error::Error,
{
    async fn get_configuration(transport: &E) -> EthAppResult<AppConfiguration, E::Error> {
        // Build APDU command
        let command = APDUCommand {
            cla: Self::CLA,
            ins: ins::GET_APP_CONFIGURATION,
            p1: 0x00,
            p2: 0x00,
            data: Vec::new(),
        };

        // Send command and get response
        let response = transport
            .exchange(&command)
            .await
            .map_err(|e| EthAppError::Transport(e.into()))?;

        // Handle APDU response
        <EthApp as AppExt<E>>::handle_response_error(&response).map_err(EthAppError::Transport)?;

        // Parse response data
        parse_get_configuration_response::<E::Error>(response.data())
    }
}

/// Parse GET APP CONFIGURATION response data
fn parse_get_configuration_response<E: std::error::Error>(
    data: &[u8],
) -> EthAppResult<AppConfiguration, E> {
    if data.len() < 4 {
        return Err(EthAppError::InvalidResponseData(format!(
            "Configuration response too short: {} bytes (expected 4)",
            data.len()
        )));
    }

    // Parse flags (1 byte)
    let flags_byte = data[0];
    let flags = ConfigFlags::from_byte(flags_byte);

    // Parse version (3 bytes)
    let major = data[1];
    let minor = data[2];
    let patch = data[3];

    let version = AppVersion {
        major,
        minor,
        patch,
    };

    Ok(AppConfiguration { flags, version })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::config_flags;

    #[test]
    fn test_parse_get_configuration_response() {
        // Mock response: flags(1) + major(1) + minor(1) + patch(1)
        let response_data = vec![
            config_flags::ARBITRARY_DATA_SIGNATURE | config_flags::ERC20_EXTERNAL_INFO,
            1, // major
            2, // minor
            3, // patch
        ];

        let result = parse_get_configuration_response::<std::io::Error>(&response_data);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(config.flags.arbitrary_data_signature);
        assert!(config.flags.erc20_external_info);
        assert!(!config.flags.transaction_check_enabled);
        assert!(!config.flags.transaction_check_opt_in);

        assert_eq!(config.version.major, 1);
        assert_eq!(config.version.minor, 2);
        assert_eq!(config.version.patch, 3);
    }

    #[test]
    fn test_parse_get_configuration_response_all_flags() {
        // Mock response with all flags set
        let response_data = vec![
            config_flags::ARBITRARY_DATA_SIGNATURE
                | config_flags::ERC20_EXTERNAL_INFO
                | config_flags::TRANSACTION_CHECK_ENABLED
                | config_flags::TRANSACTION_CHECK_OPT_IN,
            2, // major
            1, // minor
            0, // patch
        ];

        let result = parse_get_configuration_response::<std::io::Error>(&response_data);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(config.flags.arbitrary_data_signature);
        assert!(config.flags.erc20_external_info);
        assert!(config.flags.transaction_check_enabled);
        assert!(config.flags.transaction_check_opt_in);

        assert_eq!(config.version.major, 2);
        assert_eq!(config.version.minor, 1);
        assert_eq!(config.version.patch, 0);
    }

    #[test]
    fn test_parse_get_configuration_response_no_flags() {
        // Mock response with no flags set
        let response_data = vec![
            0x00, // no flags
            0,    // major
            9,    // minor
            15,   // patch
        ];

        let result = parse_get_configuration_response::<std::io::Error>(&response_data);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(!config.flags.arbitrary_data_signature);
        assert!(!config.flags.erc20_external_info);
        assert!(!config.flags.transaction_check_enabled);
        assert!(!config.flags.transaction_check_opt_in);

        assert_eq!(config.version.major, 0);
        assert_eq!(config.version.minor, 9);
        assert_eq!(config.version.patch, 15);
    }

    #[test]
    fn test_parse_get_configuration_response_too_short() {
        let response_data = vec![0x01, 1, 2]; // Missing patch version

        let result = parse_get_configuration_response::<std::io::Error>(&response_data);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            EthAppError::InvalidResponseData(_)
        ));
    }

    #[test]
    fn test_config_flags_conversion() {
        let flags = ConfigFlags {
            arbitrary_data_signature: true,
            erc20_external_info: false,
            transaction_check_enabled: true,
            transaction_check_opt_in: false,
        };

        let byte = flags.to_byte();
        let parsed = ConfigFlags::from_byte(byte);

        assert_eq!(flags, parsed);
        assert_eq!(
            byte,
            config_flags::ARBITRARY_DATA_SIGNATURE | config_flags::TRANSACTION_CHECK_ENABLED
        );
    }
}
