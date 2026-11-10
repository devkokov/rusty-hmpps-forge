use forge_core::authoring::*;
use forge_core::{Value, obj};
use forge_govuk_components::blocks as govuk;
use forge_govuk_components::props::ButtonProps;

pub fn heading() -> Value {
    govuk::HEADING.build(obj! { "text": "Contacts", "size": "l" })
}

pub fn role_message() -> Value {
    govuk::BODY.build(obj! {
        "text": format("Signed in as %1 (%2).", [session("demoUser.name").into(), session("demoUser.role").into()]),
    })
}

pub fn contacts_list() -> Value {
    govuk::SUMMARY_LIST.build(obj! {
        "rows": [
            obj! {
                "key": obj! { "text": answer("contacts.0.recordName") },
                "value": obj! { "text": answer("contacts.0.recordEmail") },
                "actions": obj! { "items": [obj! { "href": "record/0", "text": "View" }] },
            },
            obj! {
                "key": obj! { "text": answer("contacts.1.recordName") },
                "value": obj! { "text": answer("contacts.1.recordEmail") },
                "actions": obj! { "items": [obj! { "href": "record/1", "text": "View" }] },
            },
            obj! {
                "key": obj! { "text": answer("contacts.2.recordName") },
                "value": obj! { "text": answer("contacts.2.recordEmail") },
                "actions": obj! { "items": [obj! { "href": "record/2", "text": "View" }] },
            },
        ],
    })
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
