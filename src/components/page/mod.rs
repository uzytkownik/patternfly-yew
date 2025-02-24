//! Full Page components
use std::rc::Rc;
use yew::prelude::*;

mod section;
mod sidebar;

pub use section::*;
pub use sidebar::*;

/// Properties for [`Page`]
#[derive(Clone, PartialEq, Properties)]
pub struct PageProperties {
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub sidebar: ChildrenWithProps<PageSidebar>,
    #[prop_or_default]
    pub tools: Children,
    #[prop_or_default]
    pub logo: Children,
    #[prop_or_default]
    pub nav: Children,
    #[prop_or(true)]
    pub open: bool,
    #[prop_or_default]
    pub full_height: bool,

    #[prop_or_default]
    pub id: AttrValue,
}

/// A full page
///
/// > The page component is used to define the basic layout of a page with either vertical or horizontal navigation.
///
/// See: <https://www.patternfly.org/v4/components/page>
///
/// ## Properties
///
/// Defined by [`PageProperties`].
///
/// ## Elements
///
/// * **Sidebar**: Contains a single [`PageSidebar`], hosting the main navigation.
/// * **Navigation**: The top header navigation section.
/// * **Tools**: Tools, shown in the header section of the page.
/// * **Logo**: A logo, show in the navigation header section.
/// * **Children**: The actual page content, probably wrapped into [`PageSection`] components.
///
#[function_component(Page)]
pub fn page(props: &PageProperties) -> Html {
    let open = use_state_eq(|| true);

    let onclick = {
        let open = open.clone();
        Callback::from(move |_| {
            open.set(!(*open));
        })
    };

    let mut class = classes!("pf-c-page");

    if props.full_height {
        class.push("pf-m-full-height");
    }

    html! (
        <div {class} id={&props.id}>
            <header class="pf-c-page__header">
                <div class="pf-c-page__header-brand">

                    if !props.sidebar.is_empty() {
                        <div class="pf-c-page__header-brand-toggle">
                            <button
                                aria-expanded={(*open).to_string()}
                                class="pf-c-button pf-m-plain"
                                type="button"
                                {onclick}
                                >
                                <i class="fas fa-bars" aria-hidden="true"/>
                            </button>
                        </div>
                    }

                    <a href="#" class="pf-c-page__header-brand-link"> {
                        for props.logo.iter()
                    } </a>

                </div>
                <div class="pf-c-page__header-nav">{for props.nav.iter()}</div>
                <div class="pf-c-page__header-tools"> { for props.tools.iter() }</div>
            </header>

            { for props.sidebar.iter().map(|mut s|{
                let props = Rc::make_mut(&mut s.props);
                props.open = *open;
                s
            }) }

            <main class="pf-c-page__main" tabindex="-1">
                { for props.children.iter() }
            </main>
        </div>
    )
}
