use forge_core::authoring::*;
use forge_core::{Value, obj};
use forge_govuk_components::blocks as govuk;
use forge_govuk_components::props::ButtonProps;

pub fn heading() -> Value {
    govuk::HEADING.build(obj! { "text": "Dashboard", "size": "l" })
}

pub fn welcome_message() -> Value {
    govuk::BODY.build(obj! {
        "text": format("Signed in as %1 (role: %2).", [session("demoUser.name").into(), session("demoUser.role").into()]),
    })
}

pub fn viewer_warning() -> Value {
    govuk::INSET_TEXT.build(obj! {
        "text": "You are signed in as a viewer. The admin panel will return a raw 403 error page. Use your browser's back button to return here.",
        "visibleWhen": session("demoUser.role").matches(condition::equals("viewer")),
    })
}

pub fn admin_link() -> Value {
    govuk::LINK_BUTTON.build(
        ButtonProps::link("/forge-developer-guide/patterns/demos/auth-role/admin-panel").text("Go to admin panel"),
    )
}

pub fn logout_button() -> Value {
    govuk::BUTTON.build(
        ButtonProps::action()
            .text("Log out")
            .classes("govuk-button--secondary")
            .name("action")
            .value("logout"),
    )
}
