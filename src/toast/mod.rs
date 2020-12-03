//! Toasts are small alerts that get shown on the top right corner of the page.
//!
//! A toast can be triggered by every component. The toast fill get sent to an agent, the Toaster.
//! The toaster will delegate displaying the toast to an instance of a ToastViewer component.
//!
//! In order for Toasts being displayed your application must have exactly one ToastViewer **before**
//! creating the first Toast.
//!
//! For example:
//! ```
//! # use yew::prelude::*;
//! pub struct App;
//! pub enum Msg {
//!   Toast(Toast),
//! }
//! # impl Component for App {
//! type Message = Msg;
//! # type Properties = ();
//! # fn create(props: Self::Properties,link: ComponentLink<Self>) -> Self {
//! #   unimplemented!()
//! # }
//!
//! fn update(&mut self, msg: Self::Message) -> bool {
//!   match msg {
//!     Msg::Toast(toast) => {
//!       ToastDispatcher::new().toast(toast);
//!       false
//!     }
//!   }
//! }
//!
//! # fn change(&mut self,_props: Self::Properties) -> bool {
//! #   unimplemented!()
//! # }
//!
//! fn view(&self) -> Html {
//!  html!{
//!     <>
//!       <ToastViewer/>
//!       <div>
//!         <button onclick=self.link.callback(|_|{
//!             Msg::Toast("Toast Title".into())
//!         })
//!       </div>
//!     </>
//!   }
//! }
//! # }
//! ```

use crate::{Action, Alert, AlertGroup, Type};

use chrono::{DateTime, Utc};
use core::cmp::Reverse;
use std::collections::BinaryHeap;
use std::{collections::HashSet, time::Duration};
use yew::prelude::*;
use yew::services::timeout::*;
use yew::worker::*;
use yew::{agent::Dispatcher, utils::window, virtual_dom::VChild};

/// Definition of a toast.
#[derive(Clone, Debug, Default)]
pub struct Toast {
    pub title: String,
    pub r#type: Type,
    pub timeout: Option<Duration>,
    pub body: Html,
    pub actions: Vec<Action>,
}

impl<S: ToString> From<S> for Toast {
    fn from(message: S) -> Self {
        Toast {
            title: message.to_string(),
            timeout: None,
            body: Default::default(),
            r#type: Default::default(),
            actions: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum Request {
    Toast(Toast),
}

pub enum ToastAction {
    ShowToast(Toast),
}

/// An agent for displaying toasts.
pub struct Toaster {
    link: AgentLink<Self>,
    /// The toast viewer.
    ///
    /// While we can handle more than one, we will only send toasts to one viewer. Registering
    /// more than one viewer will produce unexpected results.
    viewer: HashSet<HandlerId>,
}

impl Agent for Toaster {
    type Reach = Context<Self>;
    type Message = ();
    type Input = Request;
    type Output = ToastAction;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            viewer: HashSet::new(),
        }
    }

    fn update(&mut self, _: Self::Message) {}

    fn connected(&mut self, id: HandlerId) {
        if id.is_respondable() {
            self.viewer.insert(id);
        }
    }

    fn handle_input(&mut self, msg: Self::Input, _: HandlerId) {
        match msg {
            Request::Toast(msg) => {
                self.show_toast(msg);
            }
        }
    }

    fn disconnected(&mut self, id: HandlerId) {
        if id.is_respondable() {
            self.viewer.remove(&id);
        }
    }
}

impl Toaster {
    fn show_toast(&self, toast: Toast) {
        let viewer = self.viewer.iter().next();
        if let Some(viewer) = viewer {
            self.link.respond(*viewer, ToastAction::ShowToast(toast));
        } else {
            window()
                .alert_with_message(&format!(
                    "Dropped toast. No toast component registered. Message was: {}",
                    toast.title
                ))
                .ok();
        }
    }
}

pub struct ToastDispatcher(Dispatcher<Toaster>);

impl ToastDispatcher {
    pub fn new() -> Self {
        ToastDispatcher(Toaster::dispatcher())
    }

    pub fn toast(&mut self, toast: Toast) {
        self.0.send(Request::Toast(toast))
    }
}

impl Default for ToastDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ToastBridge(Box<dyn Bridge<Toaster>>);

impl ToastBridge {
    pub fn new(callback: Callback<<Toaster as Agent>::Output>) -> Self {
        let router_agent = Toaster::bridge(callback);
        ToastBridge(router_agent)
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {}

pub struct ToastEntry {
    id: usize,
    alert: VChild<Alert>,
    timeout: Option<DateTime<Utc>>,
}

/// A component to view toast alerts.
pub struct ToastViewer {
    props: Props,
    link: ComponentLink<Self>,
    alerts: Vec<ToastEntry>,
    _bridge: ToastBridge,
    counter: usize,

    task: Option<TimeoutTask>,
    timeouts: BinaryHeap<Reverse<DateTime<Utc>>>,
}

pub enum ToastViewerMsg {
    Perform(ToastAction),
    Cleanup,
    Close(usize),
}

impl Component for ToastViewer {
    type Message = ToastViewerMsg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let bridge = ToastBridge::new(link.callback(|action| ToastViewerMsg::Perform(action)));
        Self {
            props,
            link,
            _bridge: bridge,
            alerts: Vec::new(),
            counter: 0,
            task: None,
            timeouts: BinaryHeap::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            ToastViewerMsg::Perform(action) => self.perform(action),
            ToastViewerMsg::Cleanup => self.cleanup(),
            ToastViewerMsg::Close(id) => self.remove_toast(id),
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props != props {
            self.props = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        html! {
            <AlertGroup toast=true>
                { for self.alerts.iter().map(|entry|entry.alert.clone()) }
            </AlertGroup>
        }
    }
}

impl ToastViewer {
    fn now() -> DateTime<Utc> {
        Utc::now()
    }

    fn perform(&mut self, action: ToastAction) -> ShouldRender {
        match action {
            ToastAction::ShowToast(toast) => self.add_toast(toast),
        }
        true
    }

    fn add_toast(&mut self, toast: Toast) {
        let now = Self::now();
        let timeout = toast
            .timeout
            .and_then(|timeout| chrono::Duration::from_std(timeout).ok())
            .map(|timeout| now + timeout);

        let id = self.counter;
        self.counter += 1;

        let onclose = match toast.timeout {
            None => Some(self.link.callback(move |_| ToastViewerMsg::Close(id))),
            Some(_) => None,
        };

        self.alerts.push(ToastEntry {
            id,
            alert: html_nested! {
                <Alert r#type=toast.r#type title=toast.title onclose=onclose actions=toast.actions>
                    { toast.body }
                </Alert>
            },
            timeout,
        });

        if let Some(timeout) = timeout {
            self.schedule_cleanup(timeout);
        }
    }

    fn schedule_cleanup(&mut self, timeout: DateTime<Utc>) {
        log::debug!("Schedule cleanup: {:?}", timeout);

        self.timeouts.push(Reverse(timeout));
        self.trigger_next_cleanup();
    }

    fn trigger_next_cleanup(&mut self) {
        if self.task.is_some() {
            log::debug!("Already have a task");
            return;
        }

        // We poll timeouts from the heap until we find one that is in the future, or we run
        // out of candidates.
        while let Some(next) = self.timeouts.pop() {
            let timeout = next.0;
            log::debug!("Next timeout: {:?}", timeout);
            let duration = timeout - Self::now();
            let duration = duration.to_std();
            log::debug!("Duration: {:?}", duration);
            if let Ok(duration) = duration {
                self.task = Some(TimeoutService::spawn(
                    duration,
                    self.link.callback(|_| ToastViewerMsg::Cleanup),
                ));
                log::debug!("Scheduled cleanup: {:?}", duration);
                break;
            }
        }
    }

    fn remove_toast(&mut self, id: usize) -> ShouldRender {
        self.retain_alert(|entry| entry.id != id)
    }

    fn cleanup(&mut self) -> ShouldRender {
        self.task = None;
        self.trigger_next_cleanup();

        let now = Self::now();

        self.retain_alert(|alert| {
            if let Some(timeout) = alert.timeout {
                timeout > now
            } else {
                true
            }
        })
    }

    fn retain_alert<F>(&mut self, f: F) -> ShouldRender
    where
        F: Fn(&ToastEntry) -> bool,
    {
        let before = self.alerts.len();
        self.alerts.retain(f);
        before != self.alerts.len()
    }
}
