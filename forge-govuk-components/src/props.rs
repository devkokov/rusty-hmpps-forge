//! Typed authoring-side building blocks for component props (see `GOVUK_COMPONENTS_TYPING.md` at
//! the repo root for the rationale and migration plan).
//!
//! Each type here implements [`IntoValue`], so it can be used directly as a field value inside
//! [`obj!`](forge_core::obj) or as a `component()` prop, and produces exactly the `Value` shape the
//! equivalent hand-written `obj!` literal would produce — adopting these types changes nothing
//! about the serialised JSON, only how it's authored (see "Does this preserve JSON serialisation of
//! the forge config?" in the doc).
//!
//! This module is purely additive: nothing here is wired into any component yet, and no existing
//! `obj!` call site is required to use it.

use forge_core::authoring::IntoValue;
use forge_core::obj;
use forge_core::value::Value;

/// A GOV.UK `{ text }` / `{ html }` content value — never both. Mirrors the shape
/// `normaliseGovukTextParam` (`text_param` in `lib.rs`) produces at render time, but is spelled
/// explicitly at authoring time instead of inferred from a loosely-typed input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Content {
    Text(String),
    Html(String),
}

impl Content {
    pub fn text(s: impl Into<String>) -> Self {
        Content::Text(s.into())
    }

    pub fn html(s: impl Into<String>) -> Self {
        Content::Html(s.into())
    }
}

impl From<&str> for Content {
    fn from(s: &str) -> Self {
        Content::Text(s.to_string())
    }
}

impl From<String> for Content {
    fn from(s: String) -> Self {
        Content::Text(s)
    }
}

impl IntoValue for Content {
    fn into_value(self) -> Value {
        match self {
            Content::Text(t) => obj! { "text": t },
            Content::Html(h) => obj! { "html": h },
        }
    }
}

/// A component `label` param (`{ text }` / `{ html }`).
#[derive(Clone, Debug)]
pub struct Label(pub Content);

impl Label {
    pub fn text(s: impl Into<String>) -> Self {
        Label(Content::text(s))
    }

    pub fn html(s: impl Into<String>) -> Self {
        Label(Content::html(s))
    }
}

impl IntoValue for Label {
    fn into_value(self) -> Value {
        self.0.into_value()
    }
}

/// A component `hint` param (`{ text }` / `{ html }`).
#[derive(Clone, Debug)]
pub struct Hint(pub Content);

impl Hint {
    pub fn text(s: impl Into<String>) -> Self {
        Hint(Content::text(s))
    }

    pub fn html(s: impl Into<String>) -> Self {
        Hint(Content::html(s))
    }
}

impl IntoValue for Hint {
    fn into_value(self) -> Value {
        self.0.into_value()
    }
}

/// A fieldset `legend`, matching `normaliseGovukFieldset`'s `{ legend: { text } }` output shape for
/// the common case (a fieldset with only a legend, no extra fieldset-level fields).
#[derive(Clone, Debug)]
pub struct Fieldset {
    pub legend: Content,
}

impl Fieldset {
    pub fn legend(legend: impl Into<Content>) -> Self {
        Fieldset {
            legend: legend.into(),
        }
    }
}

impl IntoValue for Fieldset {
    fn into_value(self) -> Value {
        obj! { "legend": self.legend }
    }
}

/// A field `errorMessage` param, matching `normaliseGovukErrorMessage`'s `{ text }` output shape.
#[derive(Clone, Debug)]
pub struct ErrorMessage(pub Content);

impl ErrorMessage {
    pub fn text(s: impl Into<String>) -> Self {
        ErrorMessage(Content::text(s))
    }
}

impl IntoValue for ErrorMessage {
    fn into_value(self) -> Value {
        self.0.into_value()
    }
}

/// `button.type` (`govukButton`, `lib.rs`'s `button()`). Default upstream fallback is `"submit"`
/// (see `or(p.get("buttonType"), Value::from("submit"))`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonType {
    Submit,
    Button,
    Reset,
}

impl std::fmt::Display for ButtonType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ButtonType::Submit => "submit",
            ButtonType::Button => "button",
            ButtonType::Reset => "reset",
        })
    }
}

impl IntoValue for ButtonType {
    fn into_value(self) -> Value {
        Value::from(self.to_string())
    }
}

/// `input.type` (`govukInput`, `text_input()`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputType {
    Text,
    Email,
    Tel,
    Number,
    Password,
    Search,
    Url,
}

impl std::fmt::Display for InputType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            InputType::Text => "text",
            InputType::Email => "email",
            InputType::Tel => "tel",
            InputType::Number => "number",
            InputType::Password => "password",
            InputType::Search => "search",
            InputType::Url => "url",
        })
    }
}

impl IntoValue for InputType {
    fn into_value(self) -> Value {
        Value::from(self.to_string())
    }
}

/// `input.inputmode` (`govukInput`, `text_input()`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputMode {
    None,
    Text,
    Decimal,
    Numeric,
    Tel,
    Search,
    Email,
    Url,
}

impl std::fmt::Display for InputMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            InputMode::None => "none",
            InputMode::Text => "text",
            InputMode::Decimal => "decimal",
            InputMode::Numeric => "numeric",
            InputMode::Tel => "tel",
            InputMode::Search => "search",
            InputMode::Email => "email",
            InputMode::Url => "url",
        })
    }
}

impl IntoValue for InputMode {
    fn into_value(self) -> Value {
        Value::from(self.to_string())
    }
}

/// `list.style` (JSX `list()` in `jsx.rs`): anything other than `"number"` renders as `<ul>`, so this
/// is the closed set that's actually meaningful.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListStyle {
    Bullet,
    Number,
}

impl std::fmt::Display for ListStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ListStyle::Bullet => "bullet",
            ListStyle::Number => "number",
        })
    }
}

impl IntoValue for ListStyle {
    fn into_value(self) -> Value {
        Value::from(self.to_string())
    }
}

/// `heading.size` (JSX `heading()` in `jsx.rs`); also drives the default `level` when one isn't set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadingSize {
    Xl,
    L,
    M,
    S,
}

impl std::fmt::Display for HeadingSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            HeadingSize::Xl => "xl",
            HeadingSize::L => "l",
            HeadingSize::M => "m",
            HeadingSize::S => "s",
        })
    }
}

impl IntoValue for HeadingSize {
    fn into_value(self) -> Value {
        Value::from(self.to_string())
    }
}

/// `sectionBreak.size` (JSX `section_break()` in `jsx.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionBreakSize {
    Xl,
    L,
    M,
    S,
}

impl std::fmt::Display for SectionBreakSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SectionBreakSize::Xl => "xl",
            SectionBreakSize::L => "l",
            SectionBreakSize::M => "m",
            SectionBreakSize::S => "s",
        })
    }
}

impl IntoValue for SectionBreakSize {
    fn into_value(self) -> Value {
        Value::from(self.to_string())
    }
}

/// `gridRow` column `width` (JSX `grid_row()` in `jsx.rs`): GOV.UK grid column width keywords.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridColumnWidth {
    OneQuarter,
    OneThird,
    OneHalf,
    TwoThirds,
    ThreeQuarters,
    Full,
}

impl std::fmt::Display for GridColumnWidth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            GridColumnWidth::OneQuarter => "one-quarter",
            GridColumnWidth::OneThird => "one-third",
            GridColumnWidth::OneHalf => "one-half",
            GridColumnWidth::TwoThirds => "two-thirds",
            GridColumnWidth::ThreeQuarters => "three-quarters",
            GridColumnWidth::Full => "full",
        })
    }
}

impl IntoValue for GridColumnWidth {
    fn into_value(self) -> Value {
        Value::from(self.to_string())
    }
}

/// `headingLevel` / `titleHeadingLevel`, shared across `ACCORDION`, `PANEL` and
/// `NOTIFICATION_BANNER` (`lib.rs`). GOV.UK Design System headings only go from `h1` to `h6`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadingLevel {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl HeadingLevel {
    fn as_u8(self) -> u8 {
        match self {
            HeadingLevel::H1 => 1,
            HeadingLevel::H2 => 2,
            HeadingLevel::H3 => 3,
            HeadingLevel::H4 => 4,
            HeadingLevel::H5 => 5,
            HeadingLevel::H6 => 6,
        }
    }
}

impl std::fmt::Display for HeadingLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_u8())
    }
}

impl IntoValue for HeadingLevel {
    fn into_value(self) -> Value {
        Value::from(self.as_u8() as i64)
    }
}

/// `notificationBanner.type` (`notification_banner()` in `lib.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationBannerType {
    Success,
    Default,
}

impl std::fmt::Display for NotificationBannerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            NotificationBannerType::Success => "success",
            NotificationBannerType::Default => "default",
        })
    }
}

impl IntoValue for NotificationBannerType {
    fn into_value(self) -> Value {
        Value::from(self.to_string())
    }
}

/// `notificationBanner.role` (`notification_banner()` in `lib.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationBannerRole {
    Region,
    Alert,
}

impl std::fmt::Display for NotificationBannerRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            NotificationBannerRole::Region => "region",
            NotificationBannerRole::Alert => "alert",
        })
    }
}

impl IntoValue for NotificationBannerRole {
    fn into_value(self) -> Value {
        Value::from(self.to_string())
    }
}

/// What a button does. Mirrors the branch in `button()` (`lib.rs`): a link when `href` is set,
/// otherwise a form button. Modelling it as an enum makes mixing the two shapes unrepresentable.
#[derive(Clone, Debug)]
pub enum ButtonKind {
    Link {
        href: String,
    },
    Action {
        name: Option<String>,
        button_type: Option<ButtonType>,
        value: Option<String>,
        disabled: Option<bool>,
        prevent_double_click: Option<bool>,
    },
}

impl Default for ButtonKind {
    fn default() -> Self {
        ButtonKind::Action {
            name: None,
            button_type: None,
            value: None,
            disabled: None,
            prevent_double_click: None,
        }
    }
}

/// Typed props for `govuk::BUTTON`. Unset fields are omitted from the output, so the `Value` only
/// contains keys the author set, the same as a hand-written `obj!` literal would.
#[derive(Clone, Debug, Default)]
pub struct ButtonProps {
    pub id: Option<String>,
    pub content: Option<Content>,
    pub classes: Option<String>,
    pub attributes: Option<Value>,
    pub is_start_button: Option<bool>,
    pub kind: ButtonKind,
}

impl ButtonProps {
    /// A form button. Chain setters to configure it.
    pub fn action() -> Self {
        Self::default()
    }

    /// A link styled as a button. Action-only setters (`name`, `disabled`, ...) are ignored on links.
    pub fn link(href: impl Into<String>) -> Self {
        Self {
            kind: ButtonKind::Link { href: href.into() },
            ..Self::default()
        }
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.content = Some(Content::text(text));
        self
    }

    pub fn html(mut self, html: impl Into<String>) -> Self {
        self.content = Some(Content::html(html));
        self
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn classes(mut self, classes: impl Into<String>) -> Self {
        self.classes = Some(classes.into());
        self
    }

    pub fn start_button(mut self, is_start_button: bool) -> Self {
        self.is_start_button = Some(is_start_button);
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        if let ButtonKind::Action { name: n, .. } = &mut self.kind {
            *n = Some(name.into());
        }
        self
    }

    pub fn button_type(mut self, button_type: ButtonType) -> Self {
        if let ButtonKind::Action { button_type: b, .. } = &mut self.kind {
            *b = Some(button_type);
        }
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        if let ButtonKind::Action { value: v, .. } = &mut self.kind {
            *v = Some(value.into());
        }
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        if let ButtonKind::Action { disabled: d, .. } = &mut self.kind {
            *d = Some(disabled);
        }
        self
    }

    pub fn prevent_double_click(mut self, prevent: bool) -> Self {
        if let ButtonKind::Action {
            prevent_double_click: p,
            ..
        } = &mut self.kind
        {
            *p = Some(prevent);
        }
        self
    }
}

impl IntoValue for ButtonProps {
    fn into_value(self) -> Value {
        let mut map = forge_core::value::Map::new();
        if let Some(id) = self.id {
            map.insert("id".into(), id.into_value());
        }
        if let Some(content) = self.content {
            if let Value::Object(o) = content.into_value() {
                for (k, v) in o.iter() {
                    map.insert(k.clone(), v.clone());
                }
            }
        }
        if let Some(classes) = self.classes {
            map.insert("classes".into(), classes.into_value());
        }
        if let Some(attributes) = self.attributes {
            map.insert("attributes".into(), attributes);
        }
        if let Some(is_start_button) = self.is_start_button {
            map.insert("isStartButton".into(), is_start_button.into_value());
        }
        match self.kind {
            ButtonKind::Link { href } => {
                map.insert("href".into(), href.into_value());
            }
            ButtonKind::Action {
                name,
                button_type,
                value,
                disabled,
                prevent_double_click,
            } => {
                if let Some(name) = name {
                    map.insert("name".into(), name.into_value());
                }
                if let Some(button_type) = button_type {
                    map.insert("buttonType".into(), button_type.into_value());
                }
                if let Some(value) = value {
                    map.insert("value".into(), value.into_value());
                }
                if let Some(disabled) = disabled {
                    map.insert("disabled".into(), disabled.into_value());
                }
                if let Some(prevent_double_click) = prevent_double_click {
                    map.insert(
                        "preventDoubleClick".into(),
                        prevent_double_click.into_value(),
                    );
                }
            }
        }
        Value::object(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_text_matches_obj_literal() {
        assert_eq!(
            Label::text("Postcode").into_value(),
            obj! { "text": "Postcode" }
        );
    }

    #[test]
    fn label_html_matches_obj_literal() {
        assert_eq!(
            Label::html("<b>Postcode</b>").into_value(),
            obj! { "html": "<b>Postcode</b>" }
        );
    }

    #[test]
    fn hint_text_matches_obj_literal() {
        assert_eq!(
            Hint::text("Try SW1A 1AA").into_value(),
            obj! { "text": "Try SW1A 1AA" }
        );
    }

    #[test]
    fn fieldset_legend_matches_normalise_govuk_fieldset_shape() {
        assert_eq!(
            Fieldset::legend("Full name").into_value(),
            obj! { "legend": obj! { "text": "Full name" } }
        );
    }

    #[test]
    fn error_message_matches_normalise_govuk_error_message_shape() {
        assert_eq!(
            ErrorMessage::text("Enter a postcode").into_value(),
            obj! { "text": "Enter a postcode" }
        );
    }

    #[test]
    fn text_from_str_defaults_to_plain_text() {
        let t: Content = "hello".into();
        assert_eq!(t, Content::Text("hello".to_string()));
    }

    #[test]
    fn button_builder_matches_struct_literal() {
        let built = ButtonProps::action()
            .text("Save")
            .name("save")
            .button_type(ButtonType::Submit)
            .disabled(true)
            .into_value();
        let literal = ButtonProps {
            content: Some(Content::text("Save")),
            kind: ButtonKind::Action {
                name: Some("save".into()),
                button_type: Some(ButtonType::Submit),
                value: None,
                disabled: Some(true),
                prevent_double_click: None,
            },
            ..Default::default()
        }
        .into_value();
        assert_eq!(built, literal);
    }

    #[test]
    fn button_link_builder_matches_obj_literal() {
        assert_eq!(
            ButtonProps::link("/home")
                .text("Home")
                .start_button(true)
                .into_value(),
            obj! { "text": "Home", "href": "/home", "isStartButton": true }
        );
    }

    #[test]
    fn button_type_wire_strings() {
        assert_eq!(ButtonType::Submit.to_string(), "submit");
        assert_eq!(ButtonType::Button.to_string(), "button");
        assert_eq!(ButtonType::Reset.to_string(), "reset");
        assert_eq!(ButtonType::Submit.into_value(), Value::from("submit"));
    }

    #[test]
    fn input_type_wire_strings() {
        for (variant, expected) in [
            (InputType::Text, "text"),
            (InputType::Email, "email"),
            (InputType::Tel, "tel"),
            (InputType::Number, "number"),
            (InputType::Password, "password"),
            (InputType::Search, "search"),
            (InputType::Url, "url"),
        ] {
            assert_eq!(variant.to_string(), expected);
        }
    }

    #[test]
    fn input_mode_wire_strings() {
        for (variant, expected) in [
            (InputMode::None, "none"),
            (InputMode::Text, "text"),
            (InputMode::Decimal, "decimal"),
            (InputMode::Numeric, "numeric"),
            (InputMode::Tel, "tel"),
            (InputMode::Search, "search"),
            (InputMode::Email, "email"),
            (InputMode::Url, "url"),
        ] {
            assert_eq!(variant.to_string(), expected);
        }
    }

    #[test]
    fn list_style_wire_strings() {
        assert_eq!(ListStyle::Bullet.to_string(), "bullet");
        assert_eq!(ListStyle::Number.to_string(), "number");
    }

    #[test]
    fn heading_size_wire_strings() {
        assert_eq!(HeadingSize::Xl.to_string(), "xl");
        assert_eq!(HeadingSize::L.to_string(), "l");
        assert_eq!(HeadingSize::M.to_string(), "m");
        assert_eq!(HeadingSize::S.to_string(), "s");
    }

    #[test]
    fn section_break_size_wire_strings() {
        assert_eq!(SectionBreakSize::Xl.to_string(), "xl");
        assert_eq!(SectionBreakSize::L.to_string(), "l");
        assert_eq!(SectionBreakSize::M.to_string(), "m");
        assert_eq!(SectionBreakSize::S.to_string(), "s");
    }

    #[test]
    fn grid_column_width_wire_strings() {
        for (variant, expected) in [
            (GridColumnWidth::OneQuarter, "one-quarter"),
            (GridColumnWidth::OneThird, "one-third"),
            (GridColumnWidth::OneHalf, "one-half"),
            (GridColumnWidth::TwoThirds, "two-thirds"),
            (GridColumnWidth::ThreeQuarters, "three-quarters"),
            (GridColumnWidth::Full, "full"),
        ] {
            assert_eq!(variant.to_string(), expected);
        }
    }

    #[test]
    fn heading_level_wire_values() {
        assert_eq!(HeadingLevel::H1.to_string(), "1");
        assert_eq!(HeadingLevel::H6.to_string(), "6");
        assert_eq!(HeadingLevel::H2.into_value(), Value::from(2i64));
    }

    #[test]
    fn notification_banner_type_wire_strings() {
        assert_eq!(NotificationBannerType::Success.to_string(), "success");
        assert_eq!(NotificationBannerType::Default.to_string(), "default");
    }

    #[test]
    fn notification_banner_role_wire_strings() {
        assert_eq!(NotificationBannerRole::Region.to_string(), "region");
        assert_eq!(NotificationBannerRole::Alert.to_string(), "alert");
    }

    #[test]
    fn button_link_matches_obj_literal() {
        let props = ButtonProps {
            content: Some(Content::text("Start")),
            kind: ButtonKind::Link {
                href: "/start".into(),
            },
            is_start_button: Some(true),
            ..Default::default()
        };
        assert_eq!(
            props.into_value(),
            obj! { "text": "Start", "href": "/start", "isStartButton": true }
        );
    }

    #[test]
    fn button_action_matches_obj_literal() {
        let props = ButtonProps {
            id: Some("save".into()),
            content: Some(Content::text("Save")),
            classes: Some("govuk-button--secondary".into()),
            kind: ButtonKind::Action {
                name: Some("save".into()),
                button_type: Some(ButtonType::Submit),
                value: Some("1".into()),
                disabled: Some(false),
                prevent_double_click: Some(true),
            },
            ..Default::default()
        };
        assert_eq!(
            props.into_value(),
            obj! {
                "id": "save",
                "text": "Save",
                "classes": "govuk-button--secondary",
                "name": "save",
                "buttonType": "submit",
                "value": "1",
                "disabled": false,
                "preventDoubleClick": true,
            }
        );
    }

    #[test]
    fn button_default_action_emits_no_optional_keys() {
        let props = ButtonProps {
            content: Some(Content::html("<b>Go</b>")),
            ..Default::default()
        };
        assert_eq!(props.into_value(), obj! { "html": "<b>Go</b>" });
    }
}
