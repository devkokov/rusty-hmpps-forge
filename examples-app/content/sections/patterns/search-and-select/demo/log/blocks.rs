use forge_core::authoring::*;
use forge_core::{Value, obj};
use forge_govuk_components::blocks as govuk;
use forge_govuk_components::props::ButtonProps;

pub fn heading() -> Value {
    govuk::HEADING.build(obj! {
        "text": data("stationName"),
        "size": "l",
    })
}

pub fn details() -> Value {
    govuk::SUMMARY_LIST.build(obj! {
        "rows": [
            obj! {
                "key": obj! { "text": "Lines" },
                "value": obj! { "text": data("stationLines") },
            },
            obj! {
                "key": obj! { "text": "Zone" },
                "value": obj! { "text": data("stationZone") },
            },
            obj! {
                "key": obj! { "text": "Opened" },
                "value": obj! { "text": data("stationOpened") },
            },
        ],
    })
}

pub fn description() -> Value {
    govuk::BODY.build(obj! {
        "text": data("stationDescription"),
    })
}

pub fn back_button() -> Value {
    govuk::LINK_BUTTON.build(
        ButtonProps::link("/forge-developer-guide/patterns/demos/search-and-select/search")
            .text("Back to search")
            .classes("govuk-button--secondary"),
    )
}
