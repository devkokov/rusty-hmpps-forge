use forge_core::authoring::*;
use forge_core::{Value, obj};
use forge_govuk_components::blocks as govuk;
use forge_govuk_components::props::ButtonProps;

pub fn heading() -> Value {
    govuk::HEADING.build(obj! { "text": "Admin panel", "size": "l" })
}

pub fn body() -> Value {
    govuk::BODY.build(obj! {
        "text": "This page is only accessible to users with the admin role. Viewers who try to access it receive a 403 error.",
    })
}

pub fn back_link() -> Value {
    govuk::LINK_BUTTON.build(
        ButtonProps::link("/forge-developer-guide/patterns/demos/auth-role/dashboard")
            .text("Back to dashboard")
            .classes("govuk-button--secondary"),
    )
}
