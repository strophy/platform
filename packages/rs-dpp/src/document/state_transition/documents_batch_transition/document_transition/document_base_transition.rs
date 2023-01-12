use std::convert::TryFrom;

use anyhow::bail;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
pub use serde_json::Value as JsonValue;
use serde_repr::*;

use crate::{
    data_contract::DataContract,
    errors::ProtocolError,
    identifier::Identifier,
    util::json_value::{JsonValueExt, ReplaceWith},
};

pub const IDENTIFIER_FIELDS: [&str; 2] = ["$id", "$dataContractId"];

#[derive(
    Debug,
    Serialize_repr,
    Deserialize_repr,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    TryFromPrimitive,
    IntoPrimitive,
)]
#[repr(u8)]
pub enum Action {
    Create = 0,
    Replace = 1,
    // 2 - reserved for update
    Delete = 3,
}

impl Default for Action {
    fn default() -> Action {
        Action::Create
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TryFrom<&str> for Action {
    type Error = anyhow::Error;

    fn try_from(name: &str) -> Result<Action, Self::Error> {
        match name {
            "create" => Ok(Action::Create),
            "replace" => Ok(Action::Replace),
            "delete" => Ok(Action::Delete),
            _ => {
                bail!("unknown action type: '{}'", name);
            }
        }
    }
}

impl TryFrom<String> for Action {
    type Error = anyhow::Error;
    fn try_from(name: String) -> Result<Action, Self::Error> {
        Action::try_from(name.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DocumentBaseTransition {
    /// The document ID
    #[serde(rename = "$id")]
    pub id: Identifier,
    /// Name of document type found int the data contract associated with the `data_contract_id`
    #[serde(rename = "$type")]
    pub document_type: String,
    /// Action the platform should take for the associated document
    #[serde(rename = "$action")]
    pub action: Action,
    /// Data contract ID generated from the data contract's `owner_id` and `entropy`
    #[serde(rename = "$dataContractId")]
    pub data_contract_id: Identifier,

    #[serde(skip)]
    pub data_contract: DataContract,
}

impl DocumentTransitionObjectLike for DocumentBaseTransition {
    fn from_json_object(
        json_value: JsonValue,
        data_contract: DataContract,
    ) -> Result<Self, ProtocolError> {
        let mut document: DocumentBaseTransition = serde_json::from_value(json_value)?;

        document.data_contract_id = data_contract.id.clone();
        document.data_contract = data_contract;
        Ok(document)
    }

    fn from_raw_object(
        mut raw_transition: JsonValue,
        data_contract: DataContract,
    ) -> Result<DocumentBaseTransition, ProtocolError> {
        raw_transition.replace_identifier_paths(IDENTIFIER_FIELDS, ReplaceWith::Base58)?;
        let mut document: DocumentBaseTransition = serde_json::from_value(raw_transition)?;

        document.data_contract_id = data_contract.id.clone();
        document.data_contract = data_contract;

        Ok(document)
    }

    fn to_object(&self) -> Result<JsonValue, ProtocolError> {
        let mut object = serde_json::to_value(self)?;

        object.replace_identifier_paths(IDENTIFIER_FIELDS, ReplaceWith::Bytes)?;
        Ok(object)
    }

    fn to_json(&self) -> Result<JsonValue, ProtocolError> {
        let value = serde_json::to_value(self)?;
        Ok(value)
    }
}

pub trait DocumentTransitionObjectLike {
    /// Creates the Document Transition from JSON representation. The JSON representation contains
    /// binary data encoded in base64, Identifiers encoded in base58
    fn from_json_object(
        json_str: JsonValue,
        data_contract: DataContract,
    ) -> Result<Self, ProtocolError>
    where
        Self: std::marker::Sized;
    /// Creates the document transition from Raw Object
    fn from_raw_object(
        raw_transition: JsonValue,
        data_contract: DataContract,
    ) -> Result<Self, ProtocolError>
    where
        Self: std::marker::Sized;
    /// Object is an [`serde_json::Value`] instance that preserves the `Vec<u8>` representation
    /// for Identifiers and binary data
    fn to_object(&self) -> Result<JsonValue, ProtocolError>;
    /// Object is an [`serde_json::Value`] instance that replaces the binary data with
    ///  - base58 string for Identifiers
    ///  - base64 string for other binary data
    fn to_json(&self) -> Result<JsonValue, ProtocolError>;
}
