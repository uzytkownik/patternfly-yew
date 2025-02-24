//! Chip Group

use crate::{use_prop_id, Chip};
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct ChipGroupProperties {
    #[prop_or_default]
    pub children: ChildrenWithProps<Chip>,

    #[prop_or_default]
    pub id: Option<String>,

    #[prop_or_default]
    pub label: Option<String>,

    #[prop_or("Chip group list".into())]
    pub aria_label: AttrValue,
}

#[function_component(ChipGroup)]
pub fn chip_group(props: &ChipGroupProperties) -> Html {
    let id = use_prop_id(props.id.clone());

    let (aria_label, aria_labeled_by) = match props.label.is_some() {
        true => (AttrValue::default(), Some(id.to_string())),
        false => (props.aria_label.clone(), None),
    };

    let mut class = classes!("pf-c-chip-group");

    if props.label.is_some() {
        class.push(classes!("pf-m-category"));
    }

    html! (
        <div {class}>
            <div class="pf-c-chip-group__main">
                if let Some(label) = &props.label {
                    <span
                        class="pf-c-chip-group__label"
                        aria-hidden="true"
                        id={format!("{id}-label")}
                    >
                        { &label }
                    </span>
                }
                <ul
                    class="pf-c-chip-group__list"
                    role="list"
                    aria-label={aria_label}
                    aria-labeledby={aria_labeled_by}
                >
                    { for props.children.iter().map(|chip| {
                        html!(
                            <li class="pf-c-chip-group__list-item">
                                { chip }
                            </li>
                        )
                    })}
                </ul>
            </div>
        </div>
    )
}
