use forge_core::authoring::*;
use forge_core::{Value, obj};
use forge_govuk_components::blocks as govuk;
use forge_govuk_components::props::ButtonProps;

pub fn panel() -> Value {
    govuk::PANEL.build(obj! {
        "titleText": "Plan agreed",
    })
}

pub fn next_steps() -> Value {
    govuk::BODY.build(obj! {
        "text": "The sentence plan has been agreed. Restart the pattern to clear the session and try again.",
    })
}

pub fn restart_button() -> Value {
    govuk::BUTTON.build(
        ButtonProps::action()
            .text("Restart pattern")
            .classes("govuk-button--secondary")
            .name("action")
            .value("restart"),
    )
}
