use crate::ListenScheme;
use crate::app_state::{AppState, DbTxn};
use crate::database::{Cache, DB};
use crate::entity::clients_dyn::ClientDyn;
use crate::entity::jwk::JwkKeyPairAlg;
use crate::entity::scopes::Scope;
use crate::entity::users::User;
use actix_web::http::header;
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::{HttpRequest, web};
use chrono::Utc;
use cryptr::{EncKeys, EncValue, utils};
use hiqlite::{Param, Params, params};
use rauthy_api_types::clients::{
    ClientResponse, DynamicClientRequest, DynamicClientResponse, EphemeralClientRequest,
    NewClientRequest,
};
use rauthy_common::constants::{
    ADDITIONAL_ALLOWED_ORIGIN_SCHEMES, ADMIN_FORCE_MFA, APPLICATION_JSON, CACHE_TTL_APP,
    CACHE_TTL_DYN_CLIENT, CACHE_TTL_EPHEMERAL_CLIENT, DYN_CLIENT_DEFAULT_TOKEN_LIFETIME,
    DYN_CLIENT_SECRET_AUTO_ROTATE, ENABLE_EPHEMERAL_CLIENTS, EPHEMERAL_CLIENTS_ALLOWED_FLOWS,
    EPHEMERAL_CLIENTS_ALLOWED_SCOPES, EPHEMERAL_CLIENTS_FORCE_MFA, PROXY_MODE, RAUTHY_VERSION,
};
use rauthy_common::is_hiqlite;
use rauthy_common::utils::{get_rand, real_ip_from_req};
use rauthy_error::{ErrorResponse, ErrorResponseType};
use reqwest::header::CONTENT_TYPE;
use reqwest::{Url, tls};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use std::fmt::Write;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;
use std::sync::OnceLock;
use std::time::Duration;
use tracing::{debug, error, trace, warn};
use validator::Validate;

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

/**
# OIDC Client

A few values here are saved as CSV Strings instead of having foreign keys and links to other
tables.
All deleting and modifying operations are a bit more expensive this way, but we gain a lot of
performance, when we do reads on clients, which we do most of the time.

`*_lifetime` values are meant to be in seconds.
 */
#[derive(Clone, PartialEq, Eq, FromRow, Deserialize, Serialize)]
pub struct Client {
    pub id: String,
    pub name: Option<String>,
    pub enabled: bool,
    pub confidential: bool,
    pub secret: Option<Vec<u8>>,
    pub secret_kid: Option<String>,
    pub redirect_uris: String,
    pub post_logout_redirect_uris: Option<String>,
    pub allowed_origins: Option<String>,
    pub flows_enabled: String,
    // Currently supported Algorithms: RS 256, 384, 512 and EdDSA
    pub access_token_alg: String,
    // Currently supported Algorithms: RS 256, 384, 512 and EdDSA
    pub id_token_alg: String,
    pub auth_code_lifetime: i32,
    pub access_token_lifetime: i32,
    pub scopes: String,
    pub default_scopes: String,
    pub challenge: Option<String>,
    pub force_mfa: bool,
    pub client_uri: Option<String>,
    pub contacts: Option<String>,
}

impl Debug for Client {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "id: {}, name: {:?}, endabled: {}, confidential: {}, secret: <hidden>, \
        redirect_uris: {}, post_logout_redirect_uris: {:?}, allowed_origins: {:?}, \
        flows_enabled: {}, access_token_alg: {}, id_token_alg: {}, auth_code_lifetime: {}, \
        access_token_lifetime: {}, scopes: {}, default_scopes: {}, challenge: {:?}, force_mfa: {}, \
        client_uri: {:?}, contacts: {:?}",
            self.id,
            self.name,
            self.enabled,
            self.confidential,
            self.redirect_uris,
            self.post_logout_redirect_uris,
            self.allowed_origins,
            self.flows_enabled,
            self.access_token_alg,
            self.id_token_alg,
            self.auth_code_lifetime,
            self.access_token_lifetime,
            self.scopes,
            self.default_scopes,
            self.challenge,
            self.force_mfa,
            self.client_uri,
            self.contacts
        )
    }
}

// CRUD
impl Client {
    #[inline]
    pub fn cache_idx(id: &str) -> String {
        format!("client_{}", id)
    }

    // have less cloning
    pub async fn create(mut client_req: NewClientRequest) -> Result<Self, ErrorResponse> {
        let kid = if client_req.confidential {
            let (_cleartext, enc) = Self::generate_new_secret()?;
            client_req.secret = Some(enc);
            Some(EncKeys::get_static().enc_key_active.clone())
        } else {
            None
        };
        let mut client = Client::try_from(client_req)?;
        client.secret_kid = kid;

        if is_hiqlite() {
            DB::client()
                .execute(
                    r#"
INSERT INTO clients (id, name, enabled, confidential, secret, secret_kid, redirect_uris,
post_logout_redirect_uris, allowed_origins, flows_enabled, access_token_alg, id_token_alg,
auth_code_lifetime, access_token_lifetime, scopes, default_scopes, challenge, force_mfa,
client_uri, contacts)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17,
$18, $19, $20)"#,
                    params!(
                        &client.id,
                        &client.name,
                        client.enabled,
                        client.confidential,
                        &client.secret,
                        &client.secret_kid,
                        &client.redirect_uris,
                        &client.post_logout_redirect_uris,
                        &client.allowed_origins,
                        &client.flows_enabled,
                        &client.access_token_alg,
                        &client.id_token_alg,
                        client.auth_code_lifetime,
                        client.access_token_lifetime,
                        &client.scopes,
                        &client.default_scopes,
                        &client.challenge,
                        client.force_mfa,
                        &client.client_uri,
                        &client.contacts
                    ),
                )
                .await?;
        } else {
            sqlx::query!(
                r#"
    INSERT INTO clients (id, name, enabled, confidential, secret, secret_kid, redirect_uris,
    post_logout_redirect_uris, allowed_origins, flows_enabled, access_token_alg, id_token_alg,
    auth_code_lifetime, access_token_lifetime, scopes, default_scopes, challenge, force_mfa,
    client_uri, contacts)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17,
    $18, $19, $20)"#,
                client.id,
                client.name,
                client.enabled,
                client.confidential,
                client.secret,
                client.secret_kid,
                client.redirect_uris,
                client.post_logout_redirect_uris,
                client.allowed_origins,
                client.flows_enabled,
                client.access_token_alg,
                client.id_token_alg,
                client.auth_code_lifetime,
                client.access_token_lifetime,
                client.scopes,
                client.default_scopes,
                client.challenge,
                client.force_mfa,
                client.client_uri,
                client.contacts,
            )
            .execute(DB::conn())
            .await?;
        }

        Ok(client)
    }

    pub async fn create_dynamic(
        data: &web::Data<AppState>,
        client_req: DynamicClientRequest,
    ) -> Result<DynamicClientResponse, ErrorResponse> {
        let token_endpoint_auth_method = client_req
            .token_endpoint_auth_method
            .clone()
            .unwrap_or_else(|| "client_secret_basic".to_string());

        let client = Self::try_from_dyn_reg(client_req)?;

        let created = Utc::now().timestamp();
        let (_secret_plain, registration_token) = Self::generate_new_secret()?;

        if is_hiqlite() {
            DB::client().txn([
                (r#"
INSERT INTO clients (id, name, enabled, confidential, secret, secret_kid, redirect_uris,
post_logout_redirect_uris, allowed_origins, flows_enabled, access_token_alg, id_token_alg,
auth_code_lifetime, access_token_lifetime, scopes, default_scopes, challenge, force_mfa,
client_uri, contacts)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)"#,
                 params!(
                    &client.id,
                    &client.name,
                    client.enabled,
                    client.confidential,
                    &client.secret,
                    &client.secret_kid,
                    &client.redirect_uris,
                    &client.post_logout_redirect_uris,
                    &client.allowed_origins,
                    &client.flows_enabled,
                    &client.access_token_alg,
                    &client.id_token_alg,
                    client.auth_code_lifetime,
                    client.access_token_lifetime,
                    &client.scopes,
                    &client.default_scopes,
                    &client.challenge,
                    client.force_mfa,
                    &client.client_uri,
                    &client.contacts
                )),
                (r#"
INSERT INTO
clients_dyn (id, created, registration_token, token_endpoint_auth_method)
VALUES ($1, $2, $3, $4)"#,
                 params!(
                    client.id.clone(),
                    created,
                    registration_token.clone(),
                    token_endpoint_auth_method.clone()
                ))
            ]).await?;
        } else {
            let mut txn = DB::txn().await?;

            sqlx::query!(
                r#"
INSERT INTO clients (id, name, enabled, confidential, secret, secret_kid, redirect_uris,
post_logout_redirect_uris, allowed_origins, flows_enabled, access_token_alg, id_token_alg,
auth_code_lifetime, access_token_lifetime, scopes, default_scopes, challenge, force_mfa,
client_uri, contacts)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)"#,
                client.id,
                client.name,
                client.enabled,
                client.confidential,
                client.secret,
                client.secret_kid,
                client.redirect_uris,
                client.post_logout_redirect_uris,
                client.allowed_origins,
                client.flows_enabled,
                client.access_token_alg,
                client.id_token_alg,
                client.auth_code_lifetime,
                client.access_token_lifetime,
                client.scopes,
                client.default_scopes,
                client.challenge,
                client.force_mfa,
                client.client_uri,
                client.contacts,
            )
                .execute(&mut *txn)
                .await?;

            sqlx::query!(
                r#"
INSERT INTO
clients_dyn (id, created, registration_token, token_endpoint_auth_method)
VALUES ($1, $2, $3, $4)"#,
                client.id,
                created,
                registration_token,
                token_endpoint_auth_method,
            )
            .execute(&mut *txn)
            .await?;

            txn.commit().await?;
        };

        let client_dyn = ClientDyn {
            id: client.id.clone(),
            created,
            last_used: None,
            registration_token,
            token_endpoint_auth_method,
        };

        client.into_dynamic_client_response(data, client_dyn, true)
    }

    // Deletes a client
    pub async fn delete(&self) -> Result<(), ErrorResponse> {
        if is_hiqlite() {
            DB::client()
                .execute("DELETE FROM clients WHERE id = $1", params!(&self.id))
                .await?;
        } else {
            sqlx::query!("DELETE FROM clients WHERE id = $1", self.id,)
                .execute(DB::conn())
                .await?;
        }

        self.delete_cache().await?;

        // We only clean up the cache. The database uses foreign key a cascade.
        if self.is_dynamic() {
            ClientDyn::delete_from_cache(&self.id).await?;
        }

        Ok(())
    }

    pub async fn delete_cache(&self) -> Result<(), ErrorResponse> {
        DB::client()
            .delete(Cache::App, Self::cache_idx(&self.id))
            .await?;

        Ok(())
    }

    pub async fn delete_cache_for(id: &str) -> Result<(), ErrorResponse> {
        DB::client().delete(Cache::App, Self::cache_idx(id)).await?;
        Ok(())
    }

    // Returns a client by id without its secret.
    pub async fn find(id: String) -> Result<Self, ErrorResponse> {
        let client = DB::client();
        if let Some(slf) = client.get(Cache::App, Self::cache_idx(&id)).await? {
            return Ok(slf);
        };

        let slf = if is_hiqlite() {
            client
                .query_as_one("SELECT * FROM clients WHERE id = $1", params!(id))
                .await?
        } else {
            sqlx::query_as::<_, Self>("SELECT * FROM clients WHERE id = $1")
                .bind(&id)
                .fetch_one(DB::conn())
                .await?
        };

        client
            .put(Cache::App, Self::cache_idx(&slf.id), &slf, CACHE_TTL_APP)
            .await?;

        Ok(slf)
    }

    pub async fn find_all() -> Result<Vec<Self>, ErrorResponse> {
        let clients = if is_hiqlite() {
            DB::client()
                .query_as("SELECT * FROM clients", params!())
                .await?
        } else {
            sqlx::query_as("SELECT * FROM clients")
                .fetch_all(DB::conn())
                .await?
        };

        Ok(clients)
    }

    /// Returns all registered `client_uri`s to be used during `USER_REG_OPEN_REDIRECT` checks.
    pub async fn find_all_client_uris() -> Result<Vec<String>, ErrorResponse> {
        let uris = if is_hiqlite() {
            DB::client()
                .query_raw(
                    "SELECT client_uri FROM clients WHERE client_uri IS NOT NULL",
                    params!(),
                )
                .await?
                .into_iter()
                .map(|mut r| r.get::<String>("client_uri"))
                .collect::<Vec<_>>()
        } else {
            sqlx::query("SELECT client_uri FROM clients WHERE client_uri IS NOT NULL")
                .fetch_all(DB::conn())
                .await?
                .into_iter()
                .map(|r| r.get::<String, _>("client_uri"))
                .collect::<Vec<_>>()
        };

        Ok(uris)
    }

    /// Accepts either a pre-registered client_id or a URL as such.
    /// If allowed, it will dynamically build an ephemeral client and cache it, it the client_id
    /// is a URL. Otherwise, it will do a classic fetch from the database.
    /// This function should be used in places where we would possibly accept an ephemeral client.
    pub async fn find_maybe_ephemeral(id: String) -> Result<Self, ErrorResponse> {
        if !*ENABLE_EPHEMERAL_CLIENTS || Url::from_str(&id).is_err() {
            return Self::find(id).await;
        }

        let client = DB::client();
        if let Some(slf) = client.get(Cache::ClientEphemeral, &id).await? {
            return Ok(slf);
        }

        let slf = Self::ephemeral_from_url(&id).await?;

        client
            .put(
                Cache::ClientEphemeral,
                id,
                &slf,
                *CACHE_TTL_EPHEMERAL_CLIENT,
            )
            .await?;

        Ok(slf)
    }

    /// This is an expensive query using `LIKE`, only use when necessary.
    pub async fn find_with_scope(scope_name: &str) -> Result<Vec<Self>, ErrorResponse> {
        let like = format!("%{scope_name}%");

        let clients = if is_hiqlite() {
            DB::client()
                .query_as(
                    "SELECT * FROM clients WHERE scopes LIKE $1 OR default_scopes LIKE $1",
                    params!(like),
                )
                .await?
        } else {
            sqlx::query_as("SELECT * FROM clients WHERE scopes LIKE $1 OR default_scopes LIKE $1")
                .bind(like)
                .fetch_all(DB::conn())
                .await?
        };

        Ok(clients)
    }

    pub fn save_txn_append(&self, txn: &mut Vec<(&str, Params)>) {
        let allowed_origins = self.allowed_origins.clone().filter(|o| !o.is_empty());
        let contacts = self.contacts.clone().filter(|c| !c.is_empty());
        let post_logout_redirect_uris = self
            .post_logout_redirect_uris
            .clone()
            .filter(|uris| !uris.is_empty());

        txn.push((
            r#"
UPDATE clients
SET name = $1, enabled = $2, confidential = $3, secret = $4, secret_kid = $5, redirect_uris = $6,
post_logout_redirect_uris = $7, allowed_origins = $8, flows_enabled = $9, access_token_alg = $10,
id_token_alg = $11, auth_code_lifetime = $12, access_token_lifetime = $13, scopes = $14,
default_scopes = $15, challenge = $16, force_mfa= $17, client_uri = $18, contacts = $19
WHERE id = $20"#,
            params!(
                &self.name,
                self.enabled,
                self.confidential,
                &self.secret,
                &self.secret_kid,
                &self.redirect_uris,
                post_logout_redirect_uris,
                allowed_origins,
                &self.flows_enabled,
                &self.access_token_alg,
                &self.id_token_alg,
                self.auth_code_lifetime,
                self.access_token_lifetime,
                &self.scopes,
                &self.default_scopes,
                &self.challenge,
                self.force_mfa,
                &self.client_uri,
                contacts,
                &self.id
            ),
        ));
    }

    pub async fn save_txn(&self, txn: &mut DbTxn<'_>) -> Result<(), ErrorResponse> {
        let allowed_origins = self.allowed_origins.clone().filter(|o| !o.is_empty());
        let contacts = self.contacts.clone().filter(|c| !c.is_empty());
        let post_logout_redirect_uris = self
            .post_logout_redirect_uris
            .clone()
            .filter(|uris| !uris.is_empty());

        sqlx::query!(
            r#"
UPDATE clients
SET name = $1, enabled = $2, confidential = $3, secret = $4, secret_kid = $5, redirect_uris = $6,
post_logout_redirect_uris = $7, allowed_origins = $8, flows_enabled = $9, access_token_alg = $10,
id_token_alg = $11, auth_code_lifetime = $12, access_token_lifetime = $13, scopes = $14,
default_scopes = $15, challenge = $16, force_mfa= $17, client_uri = $18, contacts = $19
WHERE id = $20"#,
            self.name,
            self.enabled,
            self.confidential,
            self.secret,
            self.secret_kid,
            self.redirect_uris,
            post_logout_redirect_uris,
            allowed_origins,
            self.flows_enabled,
            self.access_token_alg,
            self.id_token_alg,
            self.auth_code_lifetime,
            self.access_token_lifetime,
            self.scopes,
            self.default_scopes,
            self.challenge,
            self.force_mfa,
            self.client_uri,
            contacts,
            self.id,
        )
        .execute(&mut **txn)
        .await?;

        Ok(())
    }

    pub async fn save_cache(&self) -> Result<(), ErrorResponse> {
        DB::client()
            .put(Cache::App, Client::cache_idx(&self.id), self, CACHE_TTL_APP)
            .await?;
        Ok(())
    }

    pub async fn save(&self) -> Result<(), ErrorResponse> {
        let allowed_origins = self.allowed_origins.clone().filter(|o| !o.is_empty());
        let contacts = self.contacts.clone().filter(|c| !c.is_empty());
        let post_logout_redirect_uris = self
            .post_logout_redirect_uris
            .clone()
            .filter(|uris| !uris.is_empty());

        if is_hiqlite() {
            DB::client()
                .execute(
                    r#"
UPDATE clients
SET name = $1, enabled = $2, confidential = $3, secret = $4, secret_kid = $5, redirect_uris = $6,
post_logout_redirect_uris = $7, allowed_origins = $8, flows_enabled = $9, access_token_alg = $10,
id_token_alg = $11, auth_code_lifetime = $12, access_token_lifetime = $13, scopes = $14,
default_scopes = $15, challenge = $16, force_mfa= $17, client_uri = $18, contacts = $19
WHERE id = $20"#,
                    params!(
                        self.name.clone(),
                        self.enabled,
                        self.confidential,
                        self.secret.clone(),
                        self.secret_kid.clone(),
                        self.redirect_uris.clone(),
                        post_logout_redirect_uris,
                        allowed_origins,
                        self.flows_enabled.clone(),
                        self.access_token_alg.clone(),
                        self.id_token_alg.clone(),
                        self.auth_code_lifetime,
                        self.access_token_lifetime,
                        self.scopes.clone(),
                        self.default_scopes.clone(),
                        self.challenge.clone(),
                        self.force_mfa,
                        self.client_uri.clone(),
                        contacts,
                        self.id.clone()
                    ),
                )
                .await?;
        } else {
            sqlx::query!(
                r#"
UPDATE clients
SET name = $1, enabled = $2, confidential = $3, secret = $4, secret_kid = $5, redirect_uris = $6,
post_logout_redirect_uris = $7, allowed_origins = $8, flows_enabled = $9, access_token_alg = $10,
id_token_alg = $11, auth_code_lifetime = $12, access_token_lifetime = $13, scopes = $14,
default_scopes = $15, challenge = $16, force_mfa= $17, client_uri = $18, contacts = $19
WHERE id = $20"#,
                self.name,
                self.enabled,
                self.confidential,
                self.secret,
                self.secret_kid,
                self.redirect_uris,
                post_logout_redirect_uris,
                allowed_origins,
                self.flows_enabled,
                self.access_token_alg,
                self.id_token_alg,
                self.auth_code_lifetime,
                self.access_token_lifetime,
                self.scopes,
                self.default_scopes,
                self.challenge,
                self.force_mfa,
                self.client_uri,
                contacts,
                self.id,
            )
            .execute(DB::conn())
            .await?;
        }

        DB::client()
            .put(Cache::App, Client::cache_idx(&self.id), self, CACHE_TTL_APP)
            .await?;

        Ok(())
    }

    pub async fn update_dynamic(
        data: &web::Data<AppState>,
        client_req: DynamicClientRequest,
        mut client_dyn: ClientDyn,
    ) -> Result<DynamicClientResponse, ErrorResponse> {
        let token_endpoint_auth_method = client_req
            .token_endpoint_auth_method
            .clone()
            .unwrap_or_else(|| "client_secret_basic".to_string());

        let mut new_client = Self::try_from_dyn_reg(client_req)?;
        let current = Self::find(client_dyn.id.clone()).await?;
        if !current.is_dynamic() {
            return Err(ErrorResponse::new(
                ErrorResponseType::Forbidden,
                "Invalid request for non-dynamic client",
            ));
        }

        // we need to keep some old and possibly user-modified values
        new_client.id = current.id;
        new_client.force_mfa = current.force_mfa;
        new_client.scopes = current.scopes;
        new_client.default_scopes = current.default_scopes;

        client_dyn.token_endpoint_auth_method = token_endpoint_auth_method;
        client_dyn.last_used = Some(Utc::now().timestamp());

        if *DYN_CLIENT_SECRET_AUTO_ROTATE {
            let (_secret_plain, registration_token) = Client::generate_new_secret()?;
            client_dyn.registration_token = registration_token;
        }

        if is_hiqlite() {
            let mut txn = Vec::with_capacity(2);
            new_client.save_txn_append(&mut txn);
            txn.push((
                r#"
UPDATE clients_dyn
SET registration_token = $1, token_endpoint_auth_method = $2, last_used = $3
WHERE id = $4"#,
                params!(
                    client_dyn.registration_token.clone(),
                    client_dyn.token_endpoint_auth_method.clone(),
                    client_dyn.last_used,
                    client_dyn.id.clone()
                ),
            ));

            DB::client().txn(txn).await?;
        } else {
            let mut txn = DB::txn().await?;

            new_client.save_txn(&mut txn).await?;
            sqlx::query!(
                r#"
UPDATE clients_dyn
SET registration_token = $1, token_endpoint_auth_method = $2, last_used = $3
WHERE id = $4"#,
                client_dyn.registration_token,
                client_dyn.token_endpoint_auth_method,
                client_dyn.last_used,
                client_dyn.id,
            )
            .execute(&mut *txn)
            .await?;

            txn.commit().await?;
        }

        new_client.save_cache().await?;
        DB::client()
            .put(
                Cache::ClientDynamic,
                ClientDyn::get_cache_entry(&client_dyn.id),
                &client_dyn,
                *CACHE_TTL_DYN_CLIENT,
            )
            .await?;

        new_client.into_dynamic_client_response(data, client_dyn, *DYN_CLIENT_SECRET_AUTO_ROTATE)
    }
}

impl Client {
    pub fn allow_refresh_token(&self) -> bool {
        self.flows_enabled.contains("refresh_token")
    }

    // TODO make a generic 'delete_from_csv' function out of this and re-use it in some other places
    pub fn delete_scope(&mut self, scope: &str) {
        let len = scope.bytes().len();

        // find the scope via index in the string
        // first entry: delete scope + ',' if it exists
        // last entry: delete scope + ',' in front if it exists
        // middle: delete scope + ',' in front if it exists
        // --> 2 cases: first entry or else
        if let Some(i) = self.scopes.find(scope) {
            if i == 0 {
                // the scope is the first entry
                if self.scopes.len() > len {
                    let s = format!("{},", scope);
                    self.scopes = self.scopes.replace(&s, "");
                } else {
                    self.scopes = String::default();
                }
            } else {
                // the scope is at the end or in the middle
                let s = format!(",{}", scope);
                self.scopes = self.scopes.replace(&s, "");
            }
        }

        // check if the to-be-deleted scope is in the default scopes
        if let Some(i) = self.default_scopes.find(scope) {
            if i == 0 {
                // the scope is the first entry
                if self.default_scopes.len() > len {
                    let s = format!("{},", scope);
                    self.default_scopes = self.default_scopes.replace(&s, "");
                } else {
                    self.default_scopes = String::default();
                }
            } else {
                // the scope is at the end or in the middle
                let s = format!(",{}", scope);
                self.default_scopes = self.default_scopes.replace(&s, "");
            }
        }
    }

    #[inline(always)]
    pub fn force_mfa(&self) -> bool {
        self.force_mfa || self.id == "rauthy" && *ADMIN_FORCE_MFA
    }

    // Generates a new random 64 character long client secret and returns the cleartext and
    /// encrypted version
    /// # Panics
    /// The decryption depends on correctly set up `ENC_KEYS` and `ENC_KEY_ACTIVE` environment
    /// variables and panics, if this is not the case.
    #[inline(always)]
    pub fn generate_new_secret() -> Result<(String, Vec<u8>), ErrorResponse> {
        let rnd = utils::secure_random_alnum(64);
        let enc = EncValue::encrypt(rnd.as_bytes())?.into_bytes().to_vec();
        Ok((rnd, enc))
    }

    #[inline(always)]
    pub fn get_access_token_alg(&self) -> Result<JwkKeyPairAlg, ErrorResponse> {
        JwkKeyPairAlg::from_str(self.access_token_alg.as_str())
    }

    #[inline]
    pub fn get_allowed_origins(&self) -> Option<Vec<String>> {
        if let Some(origins) = &self.allowed_origins {
            let mut res = Vec::with_capacity(1);
            for o in origins.split(',') {
                if !o.is_empty() {
                    res.push(o.to_string());
                }
            }
            Some(res)
        } else {
            None
        }
    }

    #[inline]
    pub fn get_challenges(&self) -> Option<Vec<String>> {
        self.challenge.as_ref()?;

        let mut res = Vec::new();
        self.challenge
            .as_ref()
            .unwrap()
            .split(',')
            .for_each(|c| res.push(c.trim().to_owned()));
        Some(res)
    }

    #[inline]
    pub fn get_contacts(&self) -> Option<Vec<String>> {
        if let Some(contacts) = &self.contacts {
            let mut res = Vec::with_capacity(1);
            for c in contacts.split(',') {
                if !c.is_empty() {
                    res.push(c.to_string());
                }
            }
            Some(res)
        } else {
            None
        }
    }

    /// Decrypts the client secret (if it exists) and then returns it as clear text.
    pub fn get_secret_cleartext(&self) -> Result<Option<String>, ErrorResponse> {
        if let Some(secret) = self.secret.as_ref() {
            let bytes = EncValue::try_from(secret.clone())?.decrypt()?;
            let cleartext = String::from_utf8_lossy(bytes.as_ref()).to_string();
            Ok(Some(cleartext))
        } else {
            Ok(None)
        }
    }

    #[inline]
    pub fn get_default_scopes(&self) -> Vec<String> {
        let mut res = Vec::new();
        self.default_scopes
            .split(',')
            .for_each(|s| res.push(s.trim().to_owned()));
        res
    }

    #[inline]
    pub fn get_id_token_alg(&self) -> Result<JwkKeyPairAlg, ErrorResponse> {
        JwkKeyPairAlg::from_str(self.id_token_alg.as_str())
    }

    #[inline]
    pub fn get_flows(&self) -> Vec<String> {
        let mut res = Vec::new();
        self.flows_enabled
            .split(',')
            .map(|f| f.trim().to_owned())
            .for_each(|f| res.push(f));
        res
    }

    #[inline]
    pub fn get_post_logout_uris(&self) -> Option<Vec<String>> {
        if let Some(uris) = &self.post_logout_redirect_uris {
            let mut res = Vec::with_capacity(1);
            for uri in uris.split(',') {
                if !uri.is_empty() {
                    res.push(uri.to_string());
                }
            }
            Some(res)
        } else {
            None
        }
    }

    #[inline]
    pub fn get_redirect_uris(&self) -> Vec<String> {
        if self.redirect_uris.is_empty() {
            Vec::default()
        } else {
            self.redirect_uris
                .split(',')
                .map(|i| i.trim().to_string())
                .collect()
        }
    }

    #[inline]
    pub fn get_scopes(&self) -> Vec<String> {
        let mut res = Vec::new();
        self.scopes
            .split(',')
            .for_each(|s| res.push(s.trim().to_owned()));
        res
    }

    #[inline]
    pub fn get_scope_as_str(&self) -> String {
        self.scopes.replace(',', " ")
    }

    #[inline]
    pub fn is_dynamic(&self) -> bool {
        self.id.starts_with("dyn$")
    }

    #[inline]
    pub fn is_ephemeral(&self) -> bool {
        // A non-dynamic client_id can never be a valid URL because of payload validation
        Url::from_str(&self.id).is_ok()
    }

    /// Sanitizes the current scopes and deletes everything, which does not exist in the `scopes`
    /// table in the database
    pub async fn sanitize_scopes(scps: Vec<String>) -> Result<String, ErrorResponse> {
        let mut res = String::with_capacity(scps.len());
        Scope::find_all().await?.into_iter().for_each(|s| {
            if scps.contains(&s.name) {
                res.push_str(s.name.as_str());
                res.push(',');
            }
        });
        // remove the last comma
        if !res.is_empty() {
            res.remove(res.len() - 1);
        }
        // check for 'openid', which should always be there
        if res.is_empty() {
            res = "openid".to_string();
        } else if !res.contains("openid") {
            res = format!("openid,{}", res);
        }
        Ok(res)
    }

    /// Sanitizes the requested scopes on the authorization endpoint and matches them to the
    /// allowed scopes for this client.
    pub fn sanitize_login_scopes(
        &self,
        scopes: &Option<Vec<String>>,
    ) -> Result<Vec<String>, ErrorResponse> {
        if scopes.is_none() {
            return Ok(self
                .default_scopes
                .split(',')
                .map(|s| s.to_string())
                .collect());
        }

        let scopes = scopes.as_ref().unwrap();
        let mut res = Vec::with_capacity(scopes.len());

        // Always add the configured default scopes
        for s in self.default_scopes.split(',') {
            res.push(s.to_string());
        }

        for s in scopes {
            if self.default_scopes.contains(s) {
                continue;
            }

            if self.scopes.contains(s) {
                res.push(s.clone());
            }
        }

        Ok(res)
    }

    /// Validates the User's access to this client depending on the `force_mfa` setting.
    /// Do this check after a possible password hash to not leak information to unauthenticated users!
    ///
    /// The "rauthy" client is the exception for this check to makes logging into the account
    /// possible without MFA. The force MFA for the Rauthy admin UI is done in
    /// Principal::validate_admin_session() depending on the `ADMIN_FORCE_MFA` config variable.
    pub fn validate_mfa(&self, user: &User) -> Result<(), ErrorResponse> {
        if &self.id != "rauthy" && self.force_mfa && !user.has_webauthn_enabled() {
            trace!("MFA required for this client but the user has none");
            Err(ErrorResponse::new(
                ErrorResponseType::MfaRequired,
                "MFA is required for this client",
            ))
        } else {
            Ok(())
        }
    }

    // Validates the `Origin` HTTP Header from an incoming request and compares it to the
    // `allowed_origins`. If the Origin is an external one and allowed by the config, it returns
    // the correct `ACCESS_CONTROL_ALLOW_ORIGIN` header which can then be inserted into the
    // HttpResponse.
    pub fn validate_origin(
        &self,
        r: &HttpRequest,
        listen_scheme: &ListenScheme,
        pub_url: &str,
    ) -> Result<Option<(HeaderName, HeaderValue)>, ErrorResponse> {
        let (is_ext, origin) = is_origin_external(r, listen_scheme, pub_url)?;
        if !is_ext {
            return Ok(None);
        }

        let err_msg = || {
            debug!("Client request from invalid origin");
            Err(ErrorResponse::new(
                ErrorResponseType::BadRequest,
                format!(
                    "Coming from an external Origin '{}' which is not allowed",
                    origin
                ),
            ))
        };

        if self.allowed_origins.is_none() {
            debug!("Allowed origins is None");
            return err_msg();
        }

        let allowed_origins = self
            .allowed_origins
            .as_ref()
            .unwrap()
            .split(',')
            .filter(|&ao| {
                // in this case, we should accept http and https, so we just execute .ends_with
                if listen_scheme == &ListenScheme::HttpHttps {
                    ao.ends_with(origin)
                } else {
                    ao.eq(origin)
                }
            })
            .count();
        if allowed_origins == 0 {
            debug!("No match found for allowed origin");
            return err_msg();
        }

        Ok(Some((
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            HeaderValue::from_str(origin)?,
        )))
    }

    pub fn validate_redirect_uri(&self, redirect_uri: &str) -> Result<(), ErrorResponse> {
        let matching_uris = self
            .get_redirect_uris()
            .iter()
            .filter(|uri| {
                (uri.ends_with('*') && redirect_uri.starts_with(uri.split_once('*').unwrap().0))
                    || uri.as_str().eq(redirect_uri)
            })
            .count();
        if matching_uris == 0 {
            trace!("Invalid `redirect_uri`");
            Err(ErrorResponse::new(
                ErrorResponseType::BadRequest,
                "Invalid redirect uri",
            ))
        } else {
            Ok(())
        }
    }

    pub fn validate_code_challenge(
        &self,
        code_challenge: &Option<String>,
        code_challenge_method: &Option<String>,
    ) -> Result<(), ErrorResponse> {
        if self.challenge.is_some() {
            if code_challenge.is_none() {
                trace!("'code_challenge' is missing");
                return Err(ErrorResponse::new(
                    ErrorResponseType::BadRequest,
                    "'code_challenge' is missing",
                ));
            }

            if code_challenge_method.is_none() {
                trace!("'code_challenge_method' is missing");
                return Err(ErrorResponse::new(
                    ErrorResponseType::BadRequest,
                    "'code_challenge_method' is missing",
                ));
            }

            let method = code_challenge_method.as_ref().unwrap();
            if !self.challenge.as_ref().unwrap().contains(method) {
                trace!("given code_challenge_method is not allowed");
                Err(ErrorResponse::new(
                    ErrorResponseType::BadRequest,
                    format!("code_challenge_method '{}' is not allowed", method),
                ))
            } else {
                Ok(())
            }
        } else if code_challenge.is_some() || code_challenge_method.is_some() {
            trace!("'code_challenge' not enabled for this client");
            Err(ErrorResponse::new(
                ErrorResponseType::BadRequest,
                "'code_challenge' not enabled for this client",
            ))
        } else {
            Ok(())
        }
    }

    pub fn validate_challenge_method(
        &self,
        code_challenge_method: &str,
    ) -> Result<(), ErrorResponse> {
        if self.challenge.is_none() {
            return Err(ErrorResponse::new(
                ErrorResponseType::BadRequest,
                "'code_challenge' not allowed",
            ));
        }
        if code_challenge_method.is_empty()
            || !self
                .challenge
                .as_ref()
                .unwrap()
                .contains(code_challenge_method)
        {
            return Err(ErrorResponse::new(
                ErrorResponseType::BadRequest,
                format!(
                    "code_challenge_method '{}' is not allowed",
                    code_challenge_method
                ),
            ));
        }
        Ok(())
    }

    pub fn validate_flow(&self, flow: &str) -> Result<(), ErrorResponse> {
        if flow.is_empty() || !self.flows_enabled.contains(flow) {
            return Err(ErrorResponse::new(
                ErrorResponseType::BadRequest,
                format!("'{}' flow is not allowed for this client", flow),
            ));
        }
        Ok(())
    }

    pub fn validate_secret(&self, secret: &str, req: &HttpRequest) -> Result<(), ErrorResponse> {
        if !self.confidential {
            error!("Cannot validate 'client_secret' for public client");
            return Err(ErrorResponse::new(
                ErrorResponseType::Internal,
                "Cannot validate 'client_secret' for public client",
            ));
        }

        let secret_enc = self.secret.as_ref().ok_or_else(|| {
            ErrorResponse::new(
                ErrorResponseType::Internal,
                format!("'{}' has no secret while being confidential", &self.id),
            )
        })?;
        let cleartext = EncValue::try_from(secret_enc.clone())?.decrypt()?;

        if cleartext.as_ref() != secret.as_bytes() {
            drop(cleartext);
            warn!(
                "Invalid login for client '{}' from '{}'",
                self.id,
                real_ip_from_req(req)?
            );

            return Err(ErrorResponse::new(
                ErrorResponseType::Unauthorized,
                "Invalid 'client_secret'",
            ));
        }
        Ok(())
    }
}

impl Client {
    async fn ephemeral_from_url(value: &str) -> Result<Self, ErrorResponse> {
        let client = HTTP_CLIENT.get_or_init(|| {
            reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(10))
                .timeout(Duration::from_secs(10))
                .user_agent(format!(
                    "Rauthy v{} Ephemeral Client Resolver",
                    RAUTHY_VERSION
                ))
                .min_tls_version(tls::Version::TLS_1_2)
                .pool_idle_timeout(Duration::from_secs(600))
                .build()
                .unwrap()
        });

        let res = client
            .get(value)
            .header(CONTENT_TYPE, APPLICATION_JSON)
            .send()
            .await
            .map_err(|err| {
                ErrorResponse::new(
                    ErrorResponseType::BadRequest,
                    format!(
                        "Cannot fetch ephemeral client data from {}: {:?}",
                        value, err
                    ),
                )
            })?;

        if !res.status().is_success() {
            let msg = format!("Cannot fetch ephemeral client information from {}", value);
            error!("{}", msg);
            return Err(ErrorResponse::new(ErrorResponseType::Connection, msg));
        }

        let body = match res.json::<EphemeralClientRequest>().await {
            Ok(b) => b,
            Err(err) => {
                let msg = format!(
                    "Cannot deserialize into EphemeralClientRequest from {}: {:?}",
                    value, err,
                );
                error!("{}", msg);
                return Err(ErrorResponse::new(ErrorResponseType::BadRequest, msg));
            }
        };
        body.validate()?;

        let slf = Self::from(body);
        if slf.id != value {
            return Err(ErrorResponse::new(
                ErrorResponseType::BadRequest,
                format!(
                    "Client id from remote document {} does not match the given URL {}",
                    slf.id, value,
                ),
            ));
        }

        Ok(slf)
    }
}

impl From<Client> for ClientResponse {
    fn from(client: Client) -> Self {
        let redirect_uris = client.get_redirect_uris();
        let post_logout_redirect_uris = client.get_post_logout_uris();
        let allowed_origins = client.get_allowed_origins();
        let flows_enabled = client.get_flows();
        let scopes = client.get_scopes();
        let default_scopes = client.get_default_scopes();
        let challenges = client.get_challenges();
        let contacts = client.get_contacts();

        let access_token_alg = JwkKeyPairAlg::from_str(&client.access_token_alg)
            .expect("internal JwkKeyPairAlg conversion to always succeed")
            .into();
        let id_token_alg = JwkKeyPairAlg::from_str(&client.id_token_alg)
            .expect("internal JwkKeyPairAlg conversion to always succeed")
            .into();

        Self {
            id: client.id,
            name: client.name,
            enabled: client.enabled,
            confidential: client.confidential,
            redirect_uris,
            post_logout_redirect_uris,
            allowed_origins,
            flows_enabled,
            access_token_alg,
            id_token_alg,
            auth_code_lifetime: client.auth_code_lifetime,
            access_token_lifetime: client.access_token_lifetime,
            scopes,
            default_scopes,
            challenges,
            force_mfa: client.force_mfa,
            client_uri: client.client_uri,
            contacts,
        }
    }
}

impl From<EphemeralClientRequest> for Client {
    fn from(value: EphemeralClientRequest) -> Self {
        let scopes = EPHEMERAL_CLIENTS_ALLOWED_SCOPES.clone();

        Self {
            id: value.client_id,
            name: value.client_name,
            enabled: true,
            confidential: false,
            secret: None,
            secret_kid: None,
            redirect_uris: value.redirect_uris.join(","),
            post_logout_redirect_uris: value.post_logout_redirect_uris.map(|uris| uris.join(",")),
            allowed_origins: None,
            flows_enabled: EPHEMERAL_CLIENTS_ALLOWED_FLOWS.clone(),
            access_token_alg: value
                .access_token_signed_response_alg
                .unwrap_or_default()
                .to_string(),
            id_token_alg: value
                .id_token_signed_response_alg
                .unwrap_or_default()
                .to_string(),
            auth_code_lifetime: 60,
            access_token_lifetime: value.default_max_age.unwrap_or(1800),
            scopes: scopes.clone(),
            default_scopes: scopes,
            challenge: Some("S256".to_string()),
            force_mfa: *EPHEMERAL_CLIENTS_FORCE_MFA,
            client_uri: value.client_uri,
            contacts: value.contacts.map(|c| c.join(",")),
        }
    }
}

#[derive(Debug)]
pub struct NewClient {
    pub id: String,
    pub secret: Option<Vec<u8>>,
    pub name: Option<String>,
    pub confidential: bool,
    pub redirect_uris: String,
    pub post_logout_redirect_uris: Option<String>,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            id: String::default(),
            name: None,
            enabled: true,
            confidential: false,
            secret: None,
            secret_kid: None,
            redirect_uris: String::default(),
            post_logout_redirect_uris: None,
            allowed_origins: None,
            flows_enabled: "authorization_code".to_string(),
            access_token_alg: "EdDSA".to_string(),
            id_token_alg: "EdDSA".to_string(),
            auth_code_lifetime: 60,
            access_token_lifetime: 1800,
            scopes: "openid,email,profile,groups".to_string(),
            default_scopes: "openid".to_string(),
            challenge: Some("S256".to_string()),
            force_mfa: false,
            client_uri: None,
            contacts: None,
        }
    }
}

impl TryFrom<NewClientRequest> for Client {
    type Error = ErrorResponse;

    fn try_from(client: NewClientRequest) -> Result<Self, Self::Error> {
        let mut redirect_uris = String::with_capacity(24);
        for uri in client.redirect_uris {
            let trimmed = uri.trim();
            if !trimmed.is_empty() {
                write!(redirect_uris, "{},", trimmed)?;
            }
        }
        redirect_uris.pop();

        let post_logout_redirect_uris =
            if let Some(post_logout_redirect_uris) = client.post_logout_redirect_uris {
                let mut uris = String::with_capacity(24);
                for uri in post_logout_redirect_uris {
                    let trimmed = uri.trim();
                    if !trimmed.is_empty() {
                        write!(uris, "{},", trimmed)?;
                    }
                }
                uris.pop();
                if uris.is_empty() { None } else { Some(uris) }
            } else {
                None
            };

        Ok(Self {
            id: client.id,
            secret: client.secret,
            name: client.name,
            confidential: client.confidential,
            redirect_uris,
            post_logout_redirect_uris,
            ..Default::default()
        })
    }
}

impl Client {
    fn try_from_dyn_reg(req: DynamicClientRequest) -> Result<Self, ErrorResponse> {
        let id = format!("dyn${}", get_rand(16));

        let confidential = req.token_endpoint_auth_method.as_deref() != Some("none");
        let (secret, secret_kid, _secret_plain) = if confidential {
            let (plain, enc) = Self::generate_new_secret()?;
            (
                Some(enc),
                Some(EncKeys::get_static().enc_key_active.clone()),
                Some(plain),
            )
        } else {
            (None, None, None)
        };

        let access_token_alg = req
            .token_endpoint_auth_signing_alg
            .unwrap_or_default()
            .to_string();
        let id_token_alg = req
            .id_token_signed_response_alg
            .unwrap_or_default()
            .to_string();

        Ok(Self {
            id,
            name: req.client_name,
            enabled: true,
            confidential,
            secret,
            secret_kid,
            redirect_uris: req.redirect_uris.join(","),
            post_logout_redirect_uris: req.post_logout_redirect_uri.filter(|uri| !uri.is_empty()),
            allowed_origins: None,
            flows_enabled: req.grant_types.join(","),
            access_token_alg,
            id_token_alg,
            access_token_lifetime: *DYN_CLIENT_DEFAULT_TOKEN_LIFETIME,
            challenge: confidential.then_some("S256".to_string()),
            force_mfa: false,
            client_uri: req.client_uri,
            contacts: req.contacts.map(|c| c.join(",")).filter(|c| !c.is_empty()),
            ..Default::default()
        })
    }

    pub fn into_dynamic_client_response(
        self,
        data: &web::Data<AppState>,
        client_dyn: ClientDyn,
        map_registration_client_uri: bool,
    ) -> Result<DynamicClientResponse, ErrorResponse> {
        let contacts = self.get_contacts();

        let redirect_uris = self.get_redirect_uris();
        let grant_types = self.get_flows();
        let post_logout_redirect_uri = self.get_redirect_uris().first().cloned();

        let client_secret = self.get_secret_cleartext()?;
        let (registration_access_token, registration_client_uri) = if map_registration_client_uri {
            (
                Some(client_dyn.registration_token_plain()?),
                Some(ClientDyn::registration_client_uri(data, &client_dyn.id)),
            )
        } else {
            (None, None)
        };

        Ok(DynamicClientResponse {
            client_id: self.id,
            client_name: self.name,
            client_uri: self.client_uri,
            contacts,
            client_secret,
            // TODO check if we can make sure that a client will renew the secret properly -> let it expire then
            client_secret_expires_at: 0,
            redirect_uris,
            post_logout_redirect_uri,
            registration_access_token,
            registration_client_uri,
            grant_types,
            id_token_signed_response_alg: self.id_token_alg,
            token_endpoint_auth_method: client_dyn.token_endpoint_auth_method,
            token_endpoint_auth_signing_alg: self.access_token_alg,
        })
    }
}

/**
Checks if the HttpRequest's `Origin` Header is an external one, which needs to be validated with
the *Allowed-Origins* setting of the current client. Returns the origin as a `&str` if the Origin
is external and needs further validation.
 */
pub fn is_origin_external<'a>(
    req: &'a HttpRequest,
    listen_scheme: &'a ListenScheme,
    pub_url: &'a str,
) -> Result<(bool, &'a str), ErrorResponse> {
    let opt = req.headers().get(header::ORIGIN);
    if opt.is_none() {
        return Ok((false, ""));
    }
    let origin = opt.unwrap().to_str().unwrap_or("");
    debug!(origin, "External Origin header found:");

    let (scheme, url) = origin.split_once("://").ok_or_else(|| {
        ErrorResponse::new(
            ErrorResponseType::BadRequest,
            "Cannot parse ORIGIN header".to_string(),
        )
    })?;

    let scheme_ok = if *PROXY_MODE && scheme == "https" {
        true
    } else {
        (match listen_scheme {
            ListenScheme::Http => scheme == "http",
            ListenScheme::Https => scheme == "https",
            ListenScheme::HttpHttps => scheme == "http" || scheme == "https",
            #[cfg(not(target_os = "windows"))]
            ListenScheme::UnixHttp => scheme == "http",
            #[cfg(not(target_os = "windows"))]
            ListenScheme::UnixHttps => scheme == "https",
        } || ADDITIONAL_ALLOWED_ORIGIN_SCHEMES
            .iter()
            .any(|s| s.as_str() == scheme))
    };
    if !scheme_ok {
        warn!(pub_url, "Not matching scheme for HttpHeader::ORIGIN");
        return Err(ErrorResponse::new(
            ErrorResponseType::BadRequest,
            "The scheme of the ORIGIN header does not match".to_string(),
        ));
    }

    debug!("pub_url: {}", pub_url);
    debug!("origin: {}", origin);

    if pub_url.eq(url) {
        return Ok((false, origin));
    }
    Ok((true, origin))
}

#[cfg(test)]
mod tests {
    use std::thread::JoinHandle;
    use std::time::Duration;
    use std::{env, thread};

    use actix_web::http::header;
    use actix_web::test::TestRequest;
    use actix_web::{App, HttpResponse, HttpServer};
    use pretty_assertions::assert_eq;
    use rauthy_common::constants::APPLICATION_JSON;
    use validator::Validate;

    use super::*;

    #[test]
    fn test_client_impl() {
        let mut client = Client {
            id: "123".to_string(),
            name: Some("Client123".to_string()),
            enabled: true,
            confidential: false,
            secret: None,
            secret_kid: None,
            redirect_uris: "".to_string(),
            post_logout_redirect_uris: None,
            allowed_origins: Some(
                "http://localhost:8081,http://localhost:8082,sample://localhost".to_string(),
            ),
            flows_enabled: "authorization_code,password".to_string(),
            access_token_alg: "EdDSA".to_string(),
            id_token_alg: "RS256".to_string(),
            auth_code_lifetime: 0,
            access_token_lifetime: 0,
            scopes: "openid,email,profile,groups".to_string(),
            default_scopes: "openid,email,profile,groups".to_string(),
            challenge: Some("S256,plain".to_string()),
            force_mfa: false,
            client_uri: Some("http://localhost:1337".to_string()),
            contacts: Some("batman@localhost.de,@alfred:matrix.org".to_string()),
        };

        assert_eq!(client.get_access_token_alg().unwrap(), JwkKeyPairAlg::EdDSA);
        assert_eq!(client.get_id_token_alg().unwrap(), JwkKeyPairAlg::RS256);

        assert_eq!(
            client.get_challenges(),
            Some(vec!["S256".to_string(), "plain".to_string()])
        );

        // scopes testing
        assert_eq!(
            client.get_scopes(),
            vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
                "groups".to_string()
            ]
        );
        assert_eq!(
            client.get_scope_as_str(),
            "openid email profile groups".to_string()
        );
        assert_eq!(
            client.get_default_scopes(),
            vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
                "groups".to_string()
            ]
        );
        // delete scope at the end
        client.delete_scope("groups");
        assert_eq!(
            client.get_scopes(),
            vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
            ]
        );
        assert_eq!(
            client.get_scope_as_str(),
            "openid email profile".to_string()
        );
        assert_eq!(
            client.get_default_scopes(),
            vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
            ]
        );
        // delete scope in the middle
        client.delete_scope("email");
        assert_eq!(
            client.get_scopes(),
            vec!["openid".to_string(), "profile".to_string(),]
        );
        assert_eq!(client.get_scope_as_str(), "openid profile".to_string());
        assert_eq!(
            client.get_default_scopes(),
            vec!["openid".to_string(), "profile".to_string(),]
        );
        // delete scope in the beginning
        client.delete_scope("openid");
        assert_eq!(client.get_scopes(), vec!["profile".to_string(),]);
        assert_eq!(client.get_scope_as_str(), "profile".to_string());
        assert_eq!(client.get_default_scopes(), vec!["profile".to_string(),]);

        assert_eq!(client.validate_challenge_method("S256"), Ok(()));
        assert_eq!(client.validate_challenge_method("plain"), Ok(()));
        assert!(client.validate_challenge_method("blabla").is_err());
        assert!(client.validate_challenge_method("").is_err());

        assert_eq!(client.validate_flow("authorization_code"), Ok(()));
        assert_eq!(client.validate_flow("password"), Ok(()));
        assert!(client.validate_flow("blabla").is_err());
        assert!(client.validate_flow("").is_err());

        // contacts
        assert_eq!(
            client.get_contacts().expect("contacts to be set"),
            vec![
                "batman@localhost.de".to_string(),
                "@alfred:matrix.org".to_string(),
            ]
        );

        // validate origin
        let listen_scheme = ListenScheme::Http;
        let pub_url = "localhost:8080";
        let origin = format!("{}://{}", listen_scheme, pub_url);

        // same origin first
        let req = TestRequest::default()
            .insert_header((header::ORIGIN, origin))
            .to_http_request();
        let res = client.validate_origin(&req, &listen_scheme, pub_url);
        assert!(res.is_ok());
        assert!(res.unwrap().is_none());

        // now other origins
        let req = TestRequest::default()
            .insert_header((header::ORIGIN, "http://localhost:8081"))
            .to_http_request();
        let res = client.validate_origin(&req, &listen_scheme, pub_url);
        assert!(res.is_ok());
        let header = res.unwrap().unwrap();
        assert_eq!(header.0, header::ACCESS_CONTROL_ALLOW_ORIGIN);
        assert_eq!(header.1, "http://localhost:8081");

        let req = TestRequest::default()
            .insert_header((header::ORIGIN, "http://localhost:8082"))
            .to_http_request();
        let res = client.validate_origin(&req, &listen_scheme, pub_url);
        assert!(res.is_ok());
        let header = res.unwrap().unwrap();
        assert_eq!(header.0, header::ACCESS_CONTROL_ALLOW_ORIGIN);
        assert_eq!(header.1, "http://localhost:8082");

        let req = TestRequest::default()
            .insert_header((header::ORIGIN, "http://localhost:8083"))
            .to_http_request();
        let res = client.validate_origin(&req, &listen_scheme, pub_url);
        assert!(res.is_err());

        let req = TestRequest::default()
            .insert_header((header::ORIGIN, "sample://localhost"))
            .to_http_request();
        let res = client.validate_origin(&req, &listen_scheme, pub_url);
        assert!(res.is_err());
    }

    #[test]
    fn test_is_origin_external() {
        let pub_url = "localhost:8443";
        unsafe { env::set_var("PROXY_MODE", "false") };

        // err without ORIGIN header
        let req = TestRequest::default().to_http_request();
        let (is_ext, origin) = is_origin_external(&req, &ListenScheme::Http, pub_url).unwrap();
        assert_eq!(is_ext, false);
        assert_eq!(origin, "");

        // should return true -> url is external
        let req = TestRequest::default()
            .insert_header((header::ORIGIN, "http://localhost:8082"))
            .to_http_request();
        let (is_ext, origin) = is_origin_external(&req, &ListenScheme::Http, pub_url).unwrap();
        assert_eq!(is_ext, true);
        assert_eq!(origin, "http://localhost:8082");

        // different protocol
        let req = TestRequest::default()
            .insert_header((header::ORIGIN, "https://localhost:8443"))
            .to_http_request();
        let res = is_origin_external(&req, &ListenScheme::Http, pub_url);
        // scheme does not match
        assert!(res.is_err());

        // different protocol vice versa
        let req = TestRequest::default()
            .insert_header((header::ORIGIN, "http://localhost:8443"))
            .to_http_request();
        let res = is_origin_external(&req, &ListenScheme::Https, pub_url);
        // scheme does not match
        assert!(res.is_err());

        // should return false -> url is NOT external
        let req = TestRequest::default()
            .insert_header((header::ORIGIN, format!("http://{}", pub_url)))
            .to_http_request();
        let (is_ext, _origin) = is_origin_external(&req, &ListenScheme::Http, pub_url).unwrap();
        assert_eq!(is_ext, false);

        // should return false -> url is NOT external
        let req = TestRequest::default()
            .insert_header((header::ORIGIN, format!("https://{}", pub_url)))
            .to_http_request();
        let (is_ext, _origin) = is_origin_external(&req, &ListenScheme::Https, pub_url).unwrap();
        assert_eq!(is_ext, false);

        // should return false -> url is NOT external
        let req = TestRequest::default()
            .insert_header((header::ORIGIN, format!("http://{}", pub_url)))
            .to_http_request();
        let (is_ext, _origin) =
            is_origin_external(&req, &ListenScheme::HttpHttps, pub_url).unwrap();
        assert_eq!(is_ext, false);

        // should return false -> url is NOT external
        let req = TestRequest::default()
            .insert_header((header::ORIGIN, format!("https://{}", pub_url)))
            .to_http_request();
        let (is_ext, _origin) =
            is_origin_external(&req, &ListenScheme::HttpHttps, pub_url).unwrap();
        assert_eq!(is_ext, false);
    }

    #[test]
    fn test_from_ephemeral_client() {
        let example_client_res_resp = r#"{
          "@context": [ "https://www.w3.org/ns/solid/oidc-context.jsonld" ],

          "client_id": "https://decentphtos.example/webid#this",
          "client_name": "DecentPhotos",
          "redirect_uris": [ "https://decentphotos.example/callback" ],
          "post_logout_redirect_uris": [ "https://decentphotos.example/logout" ],
          "client_uri": "https://decentphotos.example/",
          "logo_uri": "https://decentphotos.example/logo.png",
          "tos_uri": "https://decentphotos.example/tos.html",
          "scope": "openid webid offline_access",
          "grant_types": [ "refresh_token", "authorization_code" ],
          "response_types": [ "code" ],
          "default_max_age": 3600,
          "require_auth_time": true
        }"#;
        let payload: EphemeralClientRequest =
            serde_json::from_str(example_client_res_resp).unwrap();
        // make sure our validation is good
        payload.validate().unwrap();

        // try build a client from it
        let client = Client::from(payload);
        println!("Client from EphemeralClientRequest:\n{:?}", client);

        assert_eq!(client.id.as_str(), "https://decentphtos.example/webid#this");
        assert_eq!(client.name.as_deref(), Some("DecentPhotos"));

        let uris = client.get_redirect_uris();
        assert_eq!(uris.len(), 1);
        assert_eq!(
            uris.get(0).unwrap().as_str(),
            "https://decentphotos.example/callback",
        );

        let uris = client.get_post_logout_uris().unwrap();
        assert_eq!(uris.len(), 1);
        assert_eq!(
            uris.get(0).unwrap().as_str(),
            "https://decentphotos.example/logout",
        );
    }

    #[tokio::test]
    async fn test_ephemeral_from_url() {
        let handle = serve_ephemeral_client();

        // make sure the http server starts and keeps running
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(!handle.is_finished());

        // try to build up the whole client from the url
        let client_id = "http://127.0.0.1:10080/client";
        let client = Client::ephemeral_from_url(client_id).await.unwrap();

        // only id assertion here, the rest has been validated above in test_from_ephemeral_client()
        assert_eq!(client.id.as_str(), "http://127.0.0.1:10080/client");
    }

    fn serve_ephemeral_client() -> JoinHandle<()> {
        thread::spawn(move || {
            let actix_system = actix_web::rt::System::new();
            actix_system.block_on(async {
                HttpServer::new(|| {
                    App::new().route(
                        "/client",
                        web::get().to(|| async {
                            // Serves the example client response from the Solid OIDC primer
                            // https://solidproject.org/TR/oidc-primer
                            HttpResponse::Ok().content_type(APPLICATION_JSON).body(r#"{
                              "@context": [ "https://www.w3.org/ns/solid/oidc-context.jsonld" ],

                              "client_id": "http://127.0.0.1:10080/client",
                              "client_name": "DecentPhotos",
                              "redirect_uris": [ "https://decentphotos.example/callback" ],
                              "post_logout_redirect_uris": [ "https://decentphotos.example/logout" ],
                              "client_uri": "https://decentphotos.example/",
                              "logo_uri": "https://decentphotos.example/logo.png",
                              "tos_uri": "https://decentphotos.example/tos.html",
                              "scope": "openid webid offline_access",
                              "grant_types": [ "refresh_token", "authorization_code" ],
                              "response_types": [ "code" ],
                              "default_max_age": 60,
                              "require_auth_time": true
                            }"#,
                            )
                        }),
                    )
                })
                    .bind(("127.0.0.1", 10080))
                    .expect("port 10080 to be free for testing")
                    .run()
                    .await
                    .expect("ephemeral client test http server to start")
            })
        })
    }

    #[test]
    fn test_delete_client_custom_scope() {
        let mut client = Client::default();
        client.scopes = "email,openid,profile,groups".to_string();
        client.default_scopes = "email,openid,cust_scope".to_string();

        client.delete_scope("profile");
        assert_eq!(&client.scopes, "email,openid,groups");
        assert_eq!(&client.default_scopes, "email,openid,cust_scope");

        client.delete_scope("cust_scope");
        assert_eq!(&client.scopes, "email,openid,groups");
        assert_eq!(&client.default_scopes, "email,openid");

        client.delete_scope("email");
        assert_eq!(&client.scopes, "openid,groups");
        assert_eq!(&client.default_scopes, "openid");

        client.delete_scope("groups");
        assert_eq!(&client.scopes, "openid");
        assert_eq!(&client.default_scopes, "openid");
    }
}
