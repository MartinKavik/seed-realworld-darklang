use indexmap::IndexMap;
use std::borrow::Cow;

pub mod article_editor;
pub mod login;
pub mod register;
pub mod settings;

const MIN_PASSWORD_LENGTH: usize = 8;

// ------ FormField ------

type FieldKey = &'static str;

#[allow(clippy::module_name_repetitions)]
pub trait FormField: Clone {
    fn value(&self) -> &str;
    fn value_mut(&mut self) -> &mut String;
    fn key(&self) -> &'static str;
    fn validate(&self) -> Option<Problem>;
}

// ------ Form ------

pub struct Form<T: FormField>(IndexMap<FieldKey, T>);

impl<T: FormField> Form<T> {
    pub fn new(fields: impl IntoIterator<Item = T>) -> Self {
        Self(fields.into_iter().map(|field| (field.key(), field)).collect())
    }

    pub fn trim_fields(&self) -> TrimmedForm<T> {
        TrimmedForm(
            self.0
                .iter()
                .map(|(key, field)| {
                    let mut field = field.clone();
                    *field.value_mut() = field.value().trim().into();
                    (*key, field)
                })
                .collect(),
        )
    }

    pub fn iter_fields(&self) -> indexmap::map::Values<FieldKey, T> {
        self.0.values()
    }

    pub fn upsert_field(&mut self, field: T) {
        self.0.insert(field.key(), field);
    }
}

// ------ TrimmedForm ------

#[allow(clippy::module_name_repetitions)]
pub struct TrimmedForm<T: FormField>(IndexMap<FieldKey, T>);

impl<T: FormField> TrimmedForm<T> {
    pub fn validate(self) -> Result<ValidForm<T>, Vec<Problem>> {
        let invalid_entries = self
            .0
            .iter()
            .filter_map(|(_, field)| field.validate())
            .collect::<Vec<Problem>>();

        if invalid_entries.is_empty() {
            Ok(ValidForm(self.0))
        } else {
            Err(invalid_entries)
        }
    }
}

// ------ ValidForm ------

#[allow(clippy::module_name_repetitions)]
pub struct ValidForm<T: FormField>(IndexMap<FieldKey, T>);

impl<T: FormField> ValidForm<T> {
    pub fn iter_keys_and_fields(&self) -> indexmap::map::Iter<FieldKey, T> {
        self.0.iter()
    }
}

// ------ Problem ------

#[derive(Clone)]
// `#[allow(dead_code)]` because compiler has problems with constructed variants using `Self::`
// https://github.com/rust-lang/rust/pull/64424
#[allow(dead_code)]
pub enum Problem {
    InvalidField {
        field_key: &'static str,
        message: Cow<'static, str>,
    },
    ServerError {
        message: Cow<'static, str>,
    },
}

impl Problem {
    pub fn new_invalid_field(
        field_key: &'static str,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self::InvalidField {
            field_key,
            message: message.into(),
        }
    }

    pub fn new_server_error(message: impl Into<Cow<'static, str>>) -> Self {
        Self::ServerError {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        match self {
            Self::InvalidField {
                message,
                ..
            }
            | Self::ServerError {
                message,
            } => message,
        }
    }
}
