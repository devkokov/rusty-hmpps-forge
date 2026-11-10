use forge_core::authoring::*;
use forge_core::{Value, obj};
use forge_govuk_components::blocks as govuk;
use forge_govuk_components::props::ButtonProps;

pub fn panel() -> Value {
    govuk::PANEL.build(obj! {
        "titleText": "Address saved",
    })
}

pub fn next_steps() -> Value {
    govuk::BODY.build(obj! {
        "text": "Your address has been stored in the session. Reload any step to see the pre-filled values.",
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
