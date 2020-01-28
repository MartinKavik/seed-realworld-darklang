use crate::{
    coder::encoder::form::register::ValidForm as ValidFormEncoder,
    entity::form::{self, FormField},
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use unicode_segmentation::UnicodeSegmentation;

// ------ Form ------

pub type Form = form::Form<Field>;

impl Default for Form {
    fn default() -> Self {
        Self::new(Field::iter())
    }
}

// ------ ValidForm ------

pub type ValidForm = form::ValidForm<Field>;

impl ValidForm {
    pub fn to_encoder(&self) -> ValidFormEncoder {
        ValidFormEncoder::new(self)
    }
}

// ------ Problem ------

pub type Problem = form::Problem;

// ------ Field ------

#[derive(Clone, EnumIter)]
pub enum Field {
    Username(String),
    Email(String),
    Password(String),
}

impl FormField for Field {
    fn value(&self) -> &str {
        use Field::*;
        match self {
            Username(value) | Email(value) | Password(value) => value,
        }
    }

    fn value_mut(&mut self) -> &mut String {
        use Field::*;
        match self {
            Username(value) | Email(value) | Password(value) => value,
        }
    }

    fn key(&self) -> &'static str {
        use Field::*;
        match self {
            Username(_) => "username",
            Email(_) => "email",
            Password(_) => "password",
        }
    }

    fn validate(&self) -> Option<form::Problem> {
        use Field::*;
        match self {
            Username(value) => {
                if value.is_empty() {
                    Some(form::Problem::new_invalid_field(
                        self.key(),
                        "username can't be blank",
                    ))
                } else {
                    None
                }
            },
            Email(value) => {
                if value.is_empty() {
                    Some(form::Problem::new_invalid_field(
                        self.key(),
                        "email can't be blank",
                    ))
                } else {
                    None
                }
            },
            Password(value) => match value.graphemes(true).count() {
                0 => Some(form::Problem::new_invalid_field(
                    self.key(),
                    "password can't be blank",
                )),
                // @TODO: use exclusive range pattern once stabilized
                // https://github.com/rust-lang/rust/issues/37854
                i @ 1..=form::MIN_PASSWORD_LENGTH
                    if i < form::MIN_PASSWORD_LENGTH =>
                {
                    Some(form::Problem::new_invalid_field(
                        self.key(),
                        format!(
                            "password is too short (minimum is {} characters)",
                            form::MIN_PASSWORD_LENGTH
                        ),
                    ))
                },
                _ => None,
            },
        }
    }
}

// ====== ====== TESTS ====== ======

#[cfg(test)]
pub mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn valid_form_test() {
        // ====== ARRANGE ======
        let mut form = Form::default();
        form.upsert_field(Field::Username("John".into()));
        form.upsert_field(Field::Email("john@example.com".into()));
        form.upsert_field(Field::Password("12345678".into()));

        // ====== ACT ======
        let result = form.trim_fields().validate();

        // ====== ASSERT ======
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    fn invalid_form_test() {
        // ====== ARRANGE ======
        let form = Form::default();

        // ====== ACT ======
        let result = form.trim_fields().validate();

        // ====== ASSERT ======
        assert!(if let Err(problems) = result {
            vec![
                "username can't be blank",
                "email can't be blank",
                "password can't be blank",
            ] == problems.iter().map(form::Problem::message).collect::<Vec<_>>()
        } else {
            false
        });
    }

    #[wasm_bindgen_test]
    fn short_password_test() {
        // ====== ARRANGE ======
        let mut form = Form::default();
        form.upsert_field(Field::Password("1234567".into()));

        // ====== ACT ======
        let result = form.trim_fields().validate();

        // ====== ASSERT ======
        assert!(if let Err(problems) = result {
            problems.iter().any(|problem| {
                problem.message()
                    == "password is too short (minimum is 8 characters)"
            })
        } else {
            false
        });
    }
}
