//! The profile settings route.
//!
//! `GET profile.xboxlive.com/users/me/profile/settings` returns the settings
//! that make up a gamercard. The response is always a `profileUsers` array.

use serde::Deserialize;

use crate::client::Client;
use crate::error::Error;

/// The contract version for the profile routes.
const CONTRACT: &str = "2";

/// The settings that Xbox asks for.
const SETTINGS: &str = concat!(
    "Gamertag,ModernGamertag,Gamerscore,GameDisplayPicRaw,",
    "AccountTier,Bio,Location,XboxOneRep"
);

/// The `profileUsers` envelope. Only the first entry belongs to the caller.
#[derive(Debug, Deserialize)]
struct ProfileResponse {
    #[serde(rename = "profileUsers")]
    profile_users: Vec<Profile>,
}

/// One entry from `profileUsers`.
///
/// The settings are a list, so a field can be absent when the service does not
/// return it. Read a value with [`Profile::setting`] instead of assuming it
/// exists.
#[derive(Debug, Deserialize)]
pub struct Profile {
    /// The xuid of the account.
    pub id: String,
    /// The xuid of the host that served the profile.
    #[serde(rename = "hostId")]
    pub host_id: String,
    /// The requested settings, in the order the service returned them.
    pub settings: Vec<ProfileSetting>,
    /// Whether the account is a sponsored user.
    #[serde(rename = "isSponsoredUser")]
    pub is_sponsored_user: bool,
}

/// One `{ "id": ..., "value": ... }` entry from [`Profile::settings`].
#[derive(Debug, Deserialize)]
pub struct ProfileSetting {
    /// The setting name, e.g. `Gamertag`.
    pub id: String,
    /// The setting value.
    pub value: String,
}

impl Profile {
    /// Returns the value of one setting, or `None` if the service did not
    /// return it.
    #[must_use]
    pub fn setting(&self, id: &str) -> Option<&str> {
        self.settings
            .iter()
            .find(|setting| setting.id == id)
            .map(|setting| setting.value.as_str())
    }

    /// The `Gamertag` setting.
    #[must_use]
    pub fn gamertag(&self) -> Option<&str> {
        self.setting("Gamertag")
    }

    /// The `Gamerscore` setting as a decimal string.
    #[must_use]
    pub fn gamerscore(&self) -> Option<&str> {
        self.setting("Gamerscore")
    }
}

impl Client {
    /// Fetches the profile settings of the signed-in account.
    ///
    /// Sends contract 2 and asks for the eight settings that the Xbox app uses.
    /// Returns [`Error::EmptyProfileUsers`] when the service answers 200 with an
    /// empty `profileUsers` list.
    pub async fn profile(&self) -> Result<Profile, Error> {
        let path = format!("/users/me/profile/settings?settings={SETTINGS}");
        let response: ProfileResponse = self.get_json(&path, CONTRACT).await?;

        response
            .profile_users
            .into_iter()
            .next()
            .ok_or(Error::EmptyProfileUsers)
    }
}
