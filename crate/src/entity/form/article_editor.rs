use crate::{
    coder::encoder::form::article_editor::ValidForm as ValidFormEncoder,
    entity::form::{self, FormField},
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

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
    Title(String),
    Description(String),
    Body(String),
    Tags(String),
}

impl FormField for Field {
    fn value(&self) -> &str {
        use Field::*;
        match self {
            Title(value) | Description(value) | Body(value) | Tags(value) => {
                value
            },
        }
    }

    fn value_mut(&mut self) -> &mut String {
        use Field::*;
        match self {
            Title(value) | Description(value) | Body(value) | Tags(value) => {
                value
            },
        }
    }

    fn key(&self) -> &'static str {
        use Field::*;
        match self {
            Title(_) => "title",
            Description(_) => "description",
            Body(_) => "body",
            Tags(_) => "tags",
        }
    }

    fn validate(&self) -> Option<form::Problem> {
        use Field::*;
        match self {
            Title(value) => {
                if value.is_empty() {
                    Some(form::Problem::new_invalid_field(
                        self.key(),
                        "title can't be blank",
                    ))
                } else {
                    None
                }
            },
            Body(value) => {
                if value.is_empty() {
                    Some(form::Problem::new_invalid_field(
                        self.key(),
                        "body can't be blank",
                    ))
                } else {
                    None
                }
            },
            Tags(_) | Description(_) => None,
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
        form.upsert_field(Field::Title("I'm title".into()));
        form.upsert_field(Field::Body("I'm body".into()));

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
            vec!["title can't be blank", "body can't be blank"]
                == problems
                    .iter()
                    .map(form::Problem::message)
                    .collect::<Vec<_>>()
        } else {
            false
        });
    }
}
