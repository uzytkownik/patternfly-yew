mod cell;
mod column;
mod header;
mod model;
mod render;

pub use cell::*;
pub use column::*;
pub use header::*;
pub use model::*;
pub use render::*;

use super::{TableGridMode, TableMode};
use crate::prelude::{Dropdown, ExtendClasses, Icon, KebabToggle};
use std::rc::Rc;
use yew::{
    prelude::*,
    virtual_dom::{VChild, VNode},
};

/// Properties for [`Table`]
#[derive(Debug, PartialEq, Clone, Properties)]
pub struct TableProperties<C, M>
where
    C: Clone + Eq + 'static,
    M: PartialEq + TableModel<C> + 'static,
{
    #[prop_or_default]
    pub id: AttrValue,

    #[prop_or_default]
    pub caption: Option<String>,
    #[prop_or_default]
    pub mode: TableMode,
    /// Borders or borderless.
    ///
    /// Defaults to borders being enabled.
    #[prop_or(true)]
    pub borders: bool,
    #[prop_or_default]
    pub header: Option<VChild<TableHeader<C>>>,
    #[prop_or_default]
    pub full_width_details: bool,
    pub entries: M,

    /// When to switch to grid mode
    #[prop_or_default]
    pub grid: Option<TableGridMode>,

    #[prop_or_default]
    pub onexpand: Callback<(M::Key, bool)>,
}

#[function_component(Table)]
pub fn table<C, M>(props: &TableProperties<C, M>) -> Html
where
    C: Clone + Eq + 'static,
    M: PartialEq + TableModel<C> + 'static,
{
    let mut class = classes!("pf-c-table");

    if props
        .header
        .as_ref()
        .map_or(false, |header| header.props.sticky)
    {
        class.push(classes!("pf-m-sticky-header"));
    }

    class.extend_from(&props.grid);

    match props.mode {
        TableMode::Compact => {
            class.push(classes!("pf-m-compact"));
        }
        TableMode::CompactNoBorders => {
            class.push(classes!("pf-m-compact", "pf-m-no-border-rows"));
        }
        TableMode::CompactExpandable => {
            class.push(classes!("pf-m-compact"));
        }
        TableMode::Expandable => {
            class.push(classes!("pf-m-expandable"));
        }
        TableMode::Default => {}
    };

    if !props.borders {
        class.push(classes!("pf-m-no-border-rows"));
    }

    html! (
        <table
            id={&props.id}
            {class}
            role="grid"
        >
            if let Some(caption) = &props.caption {
                <caption>{caption}</caption>
            }
            { render_header(props) }
            { render_entries(props) }
        </table>
    )
}

fn is_expandable<C, M>(props: &TableProperties<C, M>) -> bool
where
    C: Clone + Eq + 'static,
    M: PartialEq + TableModel<C> + 'static,
{
    matches!(
        props.mode,
        TableMode::Expandable | TableMode::CompactExpandable
    )
}

fn render_header<C, M>(props: &TableProperties<C, M>) -> Html
where
    C: Clone + Eq + 'static,
    M: PartialEq + TableModel<C> + 'static,
{
    let expandable = is_expandable(props);
    match &props.header {
        Some(header) => {
            let mut header = header.clone();
            let props = Rc::make_mut(&mut header.props);
            props.expandable = expandable;
            VNode::VComp(yew::virtual_dom::VComp::from(header))
        }
        None => html!(),
    }
}

fn render_entries<C, M>(props: &TableProperties<C, M>) -> Html
where
    C: Clone + Eq + 'static,
    M: PartialEq + TableModel<C> + 'static,
{
    html!(if is_expandable(props) {
        { for props.entries.iter().map(|entry| render_expandable_entry(props, entry) )}
    } else {
        <tbody role="rowgroup">
            { for props.entries.iter().map(|entry| render_normal_entry(props, entry) )}
        </tbody>
    })
}

fn render_normal_entry<C, M>(
    props: &TableProperties<C, M>,
    entry: TableModelEntry<M::Item, M::Key>,
) -> Html
where
    C: Clone + Eq + 'static,
    M: PartialEq + TableModel<C> + 'static,
{
    html!(
        <tr role="row" key={entry.key}>
            { render_row(props, entry.value)}
        </tr>
    )
}

fn render_expandable_entry<C, M>(
    props: &TableProperties<C, M>,
    entry: TableModelEntry<M::Item, M::Key>,
) -> Html
where
    C: Clone + Eq + 'static,
    M: PartialEq + TableModel<C> + 'static,
{
    let expanded = entry.expanded;
    let key = entry.key;

    /*
    let onclick = match expanded {
        true => ctx.link().callback(move |_: MouseEvent| Msg::Collapse(idx)),
        false => ctx.link().callback(move |_: MouseEvent| Msg::Expand(idx)),
    };
     */

    let mut classes = classes!("pf-c-button");
    classes.push(classes!("pf-m-plain"));
    if expanded {
        classes.push(classes!("pf-m-expanded"));
    }

    let aria_expanded = match expanded {
        true => "true",
        false => "false",
    };

    let mut expanded_class = Classes::new();
    if expanded {
        expanded_class.push(classes!("pf-m-expanded"));
    }

    let mut cols = props
        .header
        .as_ref()
        .map_or(0, |header| header.props.children.len())
        + 1;

    let mut cells: Vec<Html> = Vec::with_capacity(cols);

    if !entry
        .value
        .is_full_width_details()
        .unwrap_or(props.full_width_details)
    {
        cells.push(html! {<td></td>});
        cols -= 1;
    }

    for cell in entry.value.render_details() {
        let mut classes = Classes::new();
        classes.extend_from(&cell.modifiers);

        cells.push(html! {
            <td class={classes} colspan={cell.cols.to_string()}>
                <div class="pf-c-table__expandable-row-content">
                    { cell.content }
                </div>
            </td>
        });

        if cols > cell.cols {
            cols -= cell.cols;
        } else {
            cols = 0;
        }
        if cols == 0 {
            break;
        }
    }

    if cols > 0 {
        cells.push(html! {
            <td colspan={cols.to_string()}></td>
        });
    }

    let mut tr_classes = classes!("pf-c-table__expandable-row");
    tr_classes.extend(expanded_class.clone());

    let onclick = props.onexpand.reform(move |_| (key.clone(), !expanded));

    html! (
        <tbody role="rowgroup" class={expanded_class}>
            <tr>
                <td class="pf-c-table__toggle">
                    <button class={classes} {onclick} aria-expanded={aria_expanded}>
                        <div class="pf-c-table__toggle-icon">
                            { Icon::AngleDown }
                        </div>
                    </button>
                </td>

                { render_row(props, entry.value) }
            </tr>

            <tr class={tr_classes}>
                { cells }
            </tr>
        </tbody>
    )
}

fn render_row<C, M>(props: &TableProperties<C, M>, entry: &M::Item) -> Vec<Html>
where
    C: Clone + Eq + 'static,
    M: PartialEq + TableModel<C> + 'static,
{
    let len = props
        .header
        .as_ref()
        .map_or(0, |header| header.props.children.len());

    let mut cells: Vec<Html> = Vec::with_capacity(len);

    for column in props
        .header
        .iter()
        .flat_map(|header| header.props.children.iter())
    {
        let cell = entry.render_cell(&CellContext {
            column: &column.props.index,
        });
        let mut class = match cell.center {
            true => classes!("pf-m-center"),
            false => Classes::new(),
        };
        class.extend_from(&cell.text_modifier);

        let label = column.props.label.clone();
        cells.push(html!(
            <td {class} data-label={label.unwrap_or_default()}>
                {cell.content}
            </td>
        ));
    }

    let actions = entry.actions();
    if !actions.is_empty() {
        cells.push(html!(
            <td class="pf-c-table__action">
                <Dropdown
                    plain=true
                    toggle={html!(<KebabToggle/>)}
                >
                    { actions }
                </Dropdown>
            </td>
        ));
    }

    cells
}
