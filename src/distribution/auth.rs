use crate::distribution::RegistryError;

use chrono::{DateTime, Utc};
use hyperx::header::Header;
use reqwest::{self, Client};
use www_authenticate::{RawChallenge, WwwAuthenticate};

use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Credential {
    Token(Token),
}

pub trait Authenticate {
    fn authenticate(self, auth: &Credential) -> Self;
}

impl Authenticate for reqwest::RequestBuilder {
    fn authenticate(self, auth: &Credential) -> Self {
        match auth {
            Credential::Token(t) => self.bearer_auth(t),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct BearerChallenge {
    pub realm: Option<String>,
    pub service: Option<String>,
    pub scopes: Option<Vec<String>>,
}

impl www_authenticate::Challenge for BearerChallenge {
    fn challenge_name() -> &'static str {
        "Bearer"
    }

    fn from_raw(raw: RawChallenge) -> Option<BearerChallenge> {
        match raw {
            RawChallenge::Token68(_) => None,
            RawChallenge::Fields(mut map) => {
                let realm = map.remove("realm");
                let service = map.remove("service");
                let scopes: Option<Vec<String>> = map.remove("scope").map(|scopes| {
                    scopes
                        .split(' ')
                        .map(std::string::ToString::to_string)
                        .collect()
                });

                Some(BearerChallenge {
                    realm,
                    service,
                    scopes,
                })
            }
        }
    }

    fn into_raw(self) -> www_authenticate::RawChallenge {
        let mut map = www_authenticate::ChallengeFields::new();
        if let Some(realm) = self.realm {
            map.insert_static("realm", realm);
        }

        if let Some(service) = self.service {
            map.insert_static("service", service);
        }

        if let Some(scope) = self.scopes {
            map.insert_static("scope", scope.join(" "));
        }

        www_authenticate::RawChallenge::Fields(map)
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Token {
    // FIXME: allow accesss_token here.
    //
    // From the spec (https://docs.docker.com/registry/spec/auth/token/):
    // For compatibility with OAuth 2.0, we will also accept token under the
    // name access_token. At least one of these fields must be specified, but
    // both may also appear (for compatibility with older clients).
    // When both are specified, they should be equivalent; if they differ
    // the client's choice is undefined.
    pub token: String,
    pub expires_in: Option<u64>,
    pub issued_at: Option<DateTime<Utc>>,
    pub refresh_token: Option<String>,
}

impl Token {
    fn get(client: &Client, chall: &BearerChallenge) -> Result<Token, RegistryError> {
        #[allow(clippy::or_fun_call)]
        let realm = chall
            .realm
            .clone()
            .ok_or(RegistryError::InvalidAuthenticationChallenge(
                "No Realm provided".into(),
            ))?;

        let request = client.get(&realm);

        let mut query_params: Vec<(&str, &str)> = vec![];

        let mut scopes: Vec<(&str, &str)> = chall
            .scopes
            .iter()
            .flat_map(|some| some.iter())
            .map(|scope| ("scope", scope.as_str()))
            .collect();

        query_params.append(&mut scopes);

        if let Some(ref service) = chall.service {
            query_params.push(("service", &service));
        }

        let request = request.query(&query_params);

        let mut response = request.send().map_err(RegistryError::ReqwestError)?;

        let status = response.status();
        if !status.is_success() {
            return Err(RegistryError::CouldNotGetToken(status));
        }

        let token: Token = response.json().map_err(RegistryError::ReqwestError)?;

        Ok(token)
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.token)
    }
}

pub fn do_challenge(
    client: &Client,
    authenticate: &reqwest::header::HeaderValue,
) -> Result<Vec<Credential>, RegistryError> {
    let raw: hyperx::header::Raw = authenticate.as_bytes().into();

    #[allow(clippy::or_fun_call)]
    let challenges = WwwAuthenticate::parse_header(&raw)
        .map_err(|_| RegistryError::InvalidAuthenticationChallenge(format!("{:?}", authenticate)))?
        .get::<BearerChallenge>()
        .ok_or(RegistryError::InvalidAuthenticationChallenge(
            "No Bearer Challenge provided".into(),
        ))?;

    let auths: Vec<Credential> = challenges
        .iter()
        .map(|c| Token::get(&client, c))
        .filter_map(Result::ok)
        .map(Credential::Token)
        .collect();

    info!("got credentials: {:?}", auths);

    Ok(auths)
}
