use base64;
use chrono::prelude::{DateTime, FixedOffset};
use failure::{Error, format_err};
use serde::ser::Serialize;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use url::{ParseError, Url};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
/// Possible extension values
pub enum ExtensionValue {
    /// Represents a [`String`] value.
    ///
    /// [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
    String(String),
    /// Represents a JSON [`Value`].
    ///
    /// [`Value`]: https://docs.serde.rs/serde_json/value/enum.Value.html
    Object(Value),
}

impl ExtensionValue {
    /// Create an [`ExtensionValue`] from a [`Into<String>`].
    ///
    /// # Example
    ///
    /// ```
    /// use cloudevents::v02::ExtensionValue;
    ///
    /// let value = ExtensionValue::from_string("value");
    /// assert_eq!(value, ExtensionValue::String("value".to_owned()));
    /// ```
    ///
    /// [`Into<String>`]: https://doc.rust-lang.org/std/convert/trait.Into.html
    /// [`ExtensionValue`]: enum.ExtensionValue.html
    pub fn from_string<S>(s: S) -> Self
    where
        S: Into<String>,
    {
        ExtensionValue::String(s.into())
    }

    /// Create an [`ExtensionValue`] from a [`Serialize`] object.
    ///
    /// # Example
    ///
    /// ```
    /// use cloudevents::v02::ExtensionValue;
    /// use serde_json::Value;
    /// use std::error::Error;
    ///
    /// fn main() -> Result<(), Box<Error>> {
    ///     let value = ExtensionValue::from_serializable("value")?;
    ///     assert_eq!(value, ExtensionValue::Object(Value::String("value".to_owned())));
    ///     Ok(())
    /// }
    /// ```
    ///
    /// [`Serialize`]: https://docs.serde.rs/serde/ser/trait.Serialize.html
    /// [`ExtensionValue`]: enum.ExtensionValue.html
    pub fn from_serializable<S>(s: S) -> Result<Self, Error>
    where
        S: Serialize,
    {
        Ok(ExtensionValue::Object(serde_json::to_value(s)?))
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
/// Possible data values
pub enum Data {
    /// Represents a string or binary value. As a binary value is base64 encoded,
    /// it is impossible to determine if the value is a string or a binary value.
    /// It is up to the client to determine the data type, and do the required processing
    /// of the [`String`] value.
    ///
    /// [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
    StringOrBinary(String),
    /// Represents a JSON [`Value`].
    ///
    /// [`Value`]: https://docs.serde.rs/serde_json/value/enum.Value.html
    Object(Value),
}

impl Data {
    /// Create a [`Data`] from a [`Into<String>`].
    ///
    /// # Example
    ///
    /// ```
    /// use cloudevents::v02::Data;
    ///
    /// let value = Data::from_string("value");
    /// assert_eq!(value, Data::StringOrBinary("value".to_owned()));
    /// ```
    ///
    /// [`Into<String>`]: https://doc.rust-lang.org/std/convert/trait.Into.html
    /// [`Data`]: enum.Data.html
    pub fn from_string<S>(s: S) -> Self
    where
        S: Into<String>,
    {
        Data::StringOrBinary(s.into())
    }

    /// Create a [`Data`] from a [`AsRef<[u8]>`].
    ///
    /// # Example
    ///
    /// ```
    /// use cloudevents::v02::Data;
    ///
    /// let value = Data::from_binary(b"value");
    /// assert_eq!(value, Data::StringOrBinary("dmFsdWU=".to_owned()));
    /// ```
    ///
    /// [`AsRef<[u8]>`]: https://doc.rust-lang.org/std/convert/trait.AsRef.html
    /// [`Data`]: enum.Data.html
    pub fn from_binary<I>(i: I) -> Self
    where
        I: AsRef<[u8]>,
    {
        Data::StringOrBinary(base64::encode(&i))
    }

    /// Create a [`Data`] from a [`Serialize`] object.
    ///
    /// # Example
    ///
    /// ```
    /// use cloudevents::v02::Data;
    /// use serde_json::Value;
    /// use std::error::Error;
    ///
    /// fn main() -> Result<(), Box<Error>> {
    ///     let value = Data::from_serializable("value")?;
    ///     assert_eq!(value, Data::Object(Value::String("value".to_owned())));
    ///     Ok(())
    /// }
    /// ```
    ///
    /// [`Serialize`]: https://docs.serde.rs/serde/ser/trait.Serialize.html
    /// [`Data`]: enum.Data.html
    pub fn from_serializable<T>(v: T) -> Result<Self, Error>
    where
        T: Serialize,
    {
        Ok(Data::Object(serde_json::to_value(v)?))
    }
}

/// Cloud event definition
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct CloudEvent {
    #[serde(rename = "type")]
    event_type: String,
    specversion: &'static str,
    source: String,
    id: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    time: Option<DateTime<FixedOffset>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    schemaurl: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    contenttype: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Data>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    extensions: Option<HashMap<String, ExtensionValue>>,
}

impl CloudEvent {
    /// Get the event type
    pub fn event_type(&self) -> &str {
        self.event_type.as_ref()
    }

    /// Get the source
    pub fn source(&self) -> &str {
        self.source.as_ref()
    }

    /// Get the event id
    pub fn event_id(&self) -> &str {
        self.id.as_ref()
    }

    /// Get the event time
    pub fn event_time(&self) -> Option<&DateTime<FixedOffset>> {
        self.time.as_ref()
    }

    /// Get the schemaurl
    pub fn schema_url(&self) -> Option<&str> {
        self.schemaurl.as_ref().map(|x| x.as_ref())
    }

    /// Get the data
    pub fn data(&self) -> Option<&Data> {
        self.data.as_ref()
    }

    /// Get the extensions
    pub fn extensions(&self) -> Option<&HashMap<String, ExtensionValue>> {
        self.extensions.as_ref()
    }
}

/// Create a new [`CloudEvent`].
///
/// # Example
/// use cloudevents::v02::CloudEvent;
/// use std::error::Error;
/// 
/// let event : Result<CloudEvent, Error> = CloudEventBuilder::default()
///     .event_id("id")
///     .source("http://www.google.com")
///     .event_type("test type")
///     .contenttype(Some("application/json")
///     .build();
///
/// [`CloudEvent`]: struct.CloudEvent.html
#[derive(Debug)]
pub struct CloudEventBuilder {
    event_type: Option<String>,
    source: Option<String>,
    id: Option<String>,
    time: Option<String>,
    schemaurl: Option<String>,
    contenttype: Option<String>,
    data: Option<Data>,
    extensions: Option<HashMap<String, ExtensionValue>>,
}

impl CloudEventBuilder {
    /// Set the event type.
    pub fn event_type<S: Into<String>>(mut self, s: S) -> Self {
        self.event_type = Some(s.into());
        self
    }

    /// Set the source.
    pub fn source<S: Into<String>>(mut self, s: S) -> Self {
        self.source = Some(s.into());
        self
    }

    /// Set the event id.
    pub fn event_id<S: Into<String>>(mut self, s: S) -> Self {
        self.id = Some(s.into());
        self
    }

    /// Set the time.
    pub fn time<S: Into<String>>(mut self, s: S) -> Self {
        self.time = Some(s.into());
        self
    }

    /// Set the schemaurl.
    pub fn schemaurl<S: Into<String>>(mut self, s: S) -> Self {
        self.schemaurl = Some(s.into());
        self
    }

    /// Set the content type.
    pub fn contenttype<S: Into<String>>(mut self, s: S) -> Self {
        self.contenttype = Some(s.into());
        self
    }

    /// Set the data.
    pub fn data(mut self, d: Data) -> Self {
        self.data = Some(d);
        self
    }

    /// Set the extensions.
    pub fn extensions(mut self, e: HashMap<String, ExtensionValue>) -> Self {
        self.extensions = Some(e);
        self
    }

    /// Build a [`CloudEvent`].
    ///
    /// # Errors
    ///
    /// An error is thrown if one of the required fields (event_type, id or source) is not populated,
    /// or if one of the validated fields (time, source and schemeurl) are populated with an invalid value.
    ///
    /// [`CloudEvent`]: struct.CloudEvent.html
    pub fn build(self) -> Result<CloudEvent, Error> {
        Ok(CloudEvent {
            event_type: self.event_type.ok_or(format_err!("Event type is required"))?,
            source: {
                if let Some(x) = self.source {
                    let source = x;
                    match Url::parse(&source) {
                        Ok(_) | Err(ParseError::RelativeUrlWithoutBase) => source,
                        Err(e) => return Err(format_err!("{}", e)),
                    }
                } else {
                    return Err(format_err!("Source is required"));
                }
            },
            id: self.id.ok_or(format_err!("Event id is required"))?,
            time: {
                if let Some(t) = self.time {
                    Some(DateTime::parse_from_rfc3339(&t)?)
                } else {
                    None
                }
            },
            contenttype: self.contenttype,
            data: self.data,
            extensions: self.extensions,
            schemaurl: {
                if let Some(x) = self.schemaurl {
                    let schemaurl = x;
                    match Url::parse(&schemaurl) {
                        Ok(_) | Err(ParseError::RelativeUrlWithoutBase) => Some(schemaurl),
                        Err(e) => return Err(format_err!("{}", e)),
                    }
                } else {
                    None
                }
            },
            specversion: "0.2",
        })
    }
}

impl Default for CloudEventBuilder {
    fn default() -> Self {
        CloudEventBuilder {
            event_type: None,
            id: None,
            schemaurl: None,
            source: None,
            extensions: None,
            data: None,
            contenttype: None,
            time: None,
        }
    }
}

/// Construct a [`CloudEvent`].
///
/// # Errors
///
/// If some of the required fields are missing, or if some of the fields
/// have invalid content an error is returned.
///
/// # Example
///
/// ```
/// use cloudevents::cloudevent_v02;
/// use std::error::Error;
///
/// fn main() -> Result<(), Box<Error>> {
///     let cloudevent = cloudevent_v02!(
///         event_type: "com.example.object.delete.v2",
///         source: "https://github.com/cloudevents/spec/pull/123",
///         event_id: "0e72b6bd-1341-46b5-9907-efde752682c4",
///         contenttype: "application/json"
///     )?;
///     Ok(())
/// }
///
/// ```
/// [ `CloudEvent`]: struct.CloudEvent.html
#[macro_export]
macro_rules! cloudevent_v02 {
    ($( $name:ident: $value:expr $(,)* )+) => {
        $crate::v02::CloudEventBuilder::default()
            $(
                .$name($value)
            )*
            .build()
    };
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn string_data_can_be_created_from_str() {
        let content = "string content";
        let data = Data::from_string(content);
        assert_eq!(data, Data::StringOrBinary(content.to_owned()));
    }

    #[test]
    fn binary_data_can_be_created_from_slice() {
        let data = Data::from_binary(b"this is binary");
        assert_eq!(
            data,
            Data::StringOrBinary("dGhpcyBpcyBiaW5hcnk=".to_owned())
        )
    }

    #[test]
    fn object_data_can_be_created_from_serializable() {
        #[derive(Serialize)]
        struct SerializableStruct {
            content: String,
        }

        let object = SerializableStruct {
            content: "content".to_owned(),
        };
        let data = Data::from_serializable(object).unwrap();
        let expected = json!({
            "content": "content",
        });
        assert_eq!(data, Data::Object(expected));
    }

    #[test]
    fn extension_string_data_can_be_created_from_str() {
        let content = "string content";
        let data = ExtensionValue::from_string(content);
        assert_eq!(data, ExtensionValue::String(content.to_owned()));
    }

    #[test]
    fn extension_object_data_can_be_created_from_serializable() {
        #[derive(Serialize)]
        struct SerializableStruct {
            content: String,
        }

        let object = SerializableStruct {
            content: "content".to_owned(),
        };
        let data = ExtensionValue::from_serializable(object).unwrap();
        let expected = json!({
            "content": "content",
        });
        assert_eq!(data, ExtensionValue::Object(expected));
    }

    #[test]
    fn builder_works() {
        let event = CloudEventBuilder::default()
            .event_id("id")
            .source("http://www.google.com")
            .event_type("test type")
            .contenttype("application/json")
            .build()
            .unwrap();

        assert_eq!(event.event_type, "test type");
        assert_eq!(event.source, "http://www.google.com");
        assert_eq!(event.id, "id");
        assert_eq!(event.specversion, "0.2");
        assert_eq!(event.extensions, None);
        assert_eq!(event.data, None);
        assert_eq!(event.time, None);
        assert_eq!(event.contenttype, Some("application/json".to_owned()));
        assert_eq!(event.schemaurl, None);
    }

    #[test]
    fn builder_macro_works() {
        let event = cloudevent_v02!(
            event_type: "test type",
            source: "http://www.google.com",
            event_id: "id",
            contenttype: "application/json",
            data: Data::from_string("test"),
        )
        .unwrap();

        assert_eq!(event.event_type, "test type");
        assert_eq!(event.source, "http://www.google.com");
        assert_eq!(event.id, "id");
        assert_eq!(event.specversion, "0.2");
        assert_eq!(event.extensions, None);
        assert_eq!(event.data, Some(Data::StringOrBinary("test".to_owned())));
        assert_eq!(event.time, None);
        assert_eq!(event.contenttype, Some("application/json".to_owned()));
        assert_eq!(event.schemaurl, None);
    }

    #[test]
    fn source_is_allowed_to_be_a_relative_uri() {
        let event = CloudEventBuilder::default()
            .event_id("id")
            .source("/cloudevents/spec/pull/123")
            .event_type("test type")
            .build()
            .unwrap();

        assert_eq!(event.source, "/cloudevents/spec/pull/123");
    }

    #[test]
    fn source_is_allowed_to_be_a_urn() {
        let event = CloudEventBuilder::default()
            .event_id("id")
            .source("urn:event:from:myapi/resourse/123")
            .event_type("test type")
            .build()
            .unwrap();

        assert_eq!(event.source, "urn:event:from:myapi/resourse/123");
    }

    #[test]
    fn source_is_allowed_to_be_a_mailto() {
        let event = CloudEventBuilder::default()
            .event_id("id")
            .source("mailto:cncf-wg-serverless@lists.cncf.io")
            .event_type("test type")
            .build()
            .unwrap();

        assert_eq!(
            event.source,
            "mailto:cncf-wg-serverless@lists.cncf.io"
        );
    }
}
