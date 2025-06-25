use std::path::PathBuf;

use crossbeam_channel::{Receiver, Sender};
use ratatui::{style::Color, widgets::*};
use reqwest::Url;

use crate::parser::types::*;

pub struct App {
    pub worker_sender: Sender<WorkerCommands>,
    pub worker_receiver: Receiver<WorkerResults>,

    // Args
    pub source: Source,

    pub state: State,

    // Parsed state
    pub spec: Option<OpenAPI>,
    pub selected_tab: usize,
    // Endpoints tab
    pub endpoints_list_state: ListState,
    pub endpoints_details_focused: bool,
    pub endpoints_details_layout: usize,
    pub endpoints_details_focused_chunk: usize,
    pub endpoints_details_fullscreen: bool,
    pub endpoints_details_path_selected_tab_idx: usize,
    pub endpoints_details_path_selected_tab: EndpointTab,
    pub endpoints_details_path_tabs_count: usize,
    pub endpoints_details_body_scroll_pos: u16,
    pub endpoints_details_parameters_scroll_pos: u16,
    pub endpoints_details_responses_scroll_pos: u16,
    pub endpoints_selected_index: Option<usize>,

    // Components tab
    pub components_navigation_list_state: ListState,
    pub components_list_state: ListState,
    pub components_focused_region: ComponentsRegion,
    pub components_selected_index: Option<usize>,
    pub components_details_scroll_pos: u16,

    // Details tab
    pub details_scroll_pos: u16,
    pub details_viewport_height: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    File(PathBuf),
    Url(Url),
}

#[derive(Debug, PartialEq, Eq)]
pub enum State {
    Initial,
    Parsing(String),
    Parsed,
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointTab {
    Parameters,
    Responses,
    Body,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentsRegion {
    Navigation,
    List,
    Details,
}

#[derive(Debug, PartialEq)]
pub enum WorkerCommands {
    ParseSpec(Source),
    Shutdown,
}

#[derive(Debug, PartialEq)]
pub enum WorkerResults {
    ParsingSpec(Source),
    SpecParsed(OpenAPI),
    Error(String),
}

pub type EndpointListItem<'a> = (Color, String, &'static str, &'a PathItem, &'a Operation);

impl App {
    pub fn new(
        source: Source,
        worker_sender: Sender<WorkerCommands>,
        worker_receiver: Receiver<WorkerResults>,
    ) -> Self {
        Self {
            worker_sender,
            worker_receiver,
            source,
            state: State::Initial,
            selected_tab: 0,
            spec: None,
            // Endpoints tab
            endpoints_list_state: ListState::default(),
            endpoints_details_focused: false,
            endpoints_details_layout: 0,
            endpoints_details_focused_chunk: 0,
            endpoints_details_fullscreen: false,
            endpoints_details_path_selected_tab_idx: 0,
            endpoints_details_path_selected_tab: EndpointTab::Parameters,
            endpoints_details_path_tabs_count: 0,
            endpoints_details_body_scroll_pos: 0,
            endpoints_details_parameters_scroll_pos: 0,
            endpoints_details_responses_scroll_pos: 0,
            endpoints_selected_index: None,
            // Components tab
            components_navigation_list_state: ListState::default(),
            components_list_state: ListState::default(),
            components_focused_region: ComponentsRegion::Navigation,
            components_selected_index: None,
            components_details_scroll_pos: 0,
            // Details tab
            details_scroll_pos: 0,
            details_viewport_height: 0,
        }
    }

    pub fn reset(&mut self) {
        self.state = State::Initial;
        self.spec = None;
        self.selected_tab = 0;

        // Endpoints tab
        self.endpoints_list_state = ListState::default();
        self.endpoints_details_focused = false;
        self.endpoints_details_layout = 0;
        self.endpoints_details_focused_chunk = 0;
        self.endpoints_details_fullscreen = false;
        self.endpoints_details_path_selected_tab_idx = 0;
        self.endpoints_details_path_selected_tab = EndpointTab::Parameters;
        self.endpoints_details_path_tabs_count = 0;
        self.endpoints_details_body_scroll_pos = 0;
        self.endpoints_details_parameters_scroll_pos = 0;
        self.endpoints_details_responses_scroll_pos = 0;
        self.endpoints_selected_index = None;

        // Components tab
        self.components_navigation_list_state = ListState::default();
        self.components_list_state = ListState::default();
        self.components_focused_region = ComponentsRegion::Navigation;
        self.components_selected_index = None;
        self.components_details_scroll_pos = 0;

        // Details tab
        self.details_scroll_pos = 0;
        self.details_viewport_height = 0;
    }

    pub fn set_parsed(&mut self, spec: OpenAPI) {
        self.state = State::Parsed;
        self.spec = Some(spec);
        self.selected_tab = 0;

        // Reset the endpoints list state
        self.endpoints_list_state = ListState::default();
        self.endpoints_details_focused = false;
        self.endpoints_details_layout = 0;
        self.endpoints_details_focused_chunk = 0;
        self.endpoints_details_fullscreen = false;
        self.endpoints_details_path_selected_tab_idx = 0;
        self.endpoints_details_path_selected_tab = EndpointTab::Parameters;
        self.endpoints_details_path_tabs_count = 0;
        self.endpoints_details_body_scroll_pos = 0;
        self.endpoints_details_parameters_scroll_pos = 0;
        self.endpoints_details_responses_scroll_pos = 0;
        self.endpoints_selected_index = None;

        // Reset the components list state
        self.components_navigation_list_state = ListState::default();
        self.components_list_state = ListState::default();
        self.components_focused_region = ComponentsRegion::Navigation;
        self.components_selected_index = None;
        self.components_details_scroll_pos = 0;

        // Reset the details scroll position
        self.details_scroll_pos = 0;
        self.details_viewport_height = 0;

        if !self.spec.as_ref().unwrap().paths.is_empty() {
            self.endpoints_list_state.select(Some(0));
        }
    }

    // Endpoints Functions
    pub fn get_endpoints_list_items(&self) -> Vec<EndpointListItem> {
        let Some(spec) = &self.spec else {
            return vec![];
        };

        let tags_with_sortid = spec.tags_with_sort_id();

        let mut items = spec
            .paths
            .iter()
            .flat_map(|(path, endpoint)| {
                let mut result = vec![];

                if let Some(get) = &endpoint.get {
                    result.push((Color::LightBlue, path.clone(), "GET", endpoint, get));
                }

                if let Some(post) = &endpoint.post {
                    result.push((Color::Green, path.clone(), "POST", endpoint, post));
                }

                if let Some(put) = &endpoint.put {
                    result.push((Color::Yellow, path.clone(), "PUT", endpoint, put));
                }

                if let Some(delete) = &endpoint.delete {
                    result.push((Color::Red, path.clone(), "DELETE", endpoint, delete));
                }

                result
            })
            .collect::<Vec<_>>();

        items.sort_by(|a, b| {
            // Sort by tag first, then by method
            let a_sort_id = tags_with_sortid.iter().find_map(|(i, t)| {
                if t.name.as_str() == a.4.tags.first().cloned().unwrap_or_default() {
                    Some(*i)
                } else {
                    None
                }
            });
            let b_sort_id = tags_with_sortid.iter().find_map(|(i, t)| {
                if t.name.as_str() == b.4.tags.first().cloned().unwrap_or_default() {
                    Some(*i)
                } else {
                    None
                }
            });

            a_sort_id.cmp(&b_sort_id).then_with(|| a.2.cmp(&b.2))
        });

        items
    }

    pub fn get_endpoint_at(&self, index: usize) -> Option<EndpointListItem> {
        let items = self.get_endpoints_list_items();

        items.get(index).cloned()
    }

    // Components Functions
    pub fn get_schemas_list_items(&self) -> Vec<(&String, &Schema)> {
        let Some(spec) = &self.spec else {
            return vec![];
        };

        let Some(components) = spec.components.as_ref() else {
            return vec![];
        };

        let mut items = components
            .schemas
            .iter()
            .filter_map(|(name, schema)| match schema {
                SchemaOrRef::Item(schema) => Some((name, schema)),
                SchemaOrRef::Ref { reference } => spec.resolve_schema(reference).map(|s| (name, s)),
            })
            .collect::<Vec<_>>();

        items.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

        items
    }

    pub fn get_schema_at(&self, index: usize) -> Option<(&String, &Schema)> {
        let items = self.get_schemas_list_items();

        items.get(index).cloned()
    }

    pub fn get_parameter_list_items(&self) -> Vec<(&String, &Parameter)> {
        let Some(spec) = &self.spec else {
            return vec![];
        };

        let Some(components) = spec.components.as_ref() else {
            return vec![];
        };

        let mut items = components
            .parameters
            .iter()
            .filter_map(|(name, param)| match param {
                ParameterOrRef::Item(param) => Some((name, param)),
                ParameterOrRef::Ref { reference } => {
                    spec.resolve_parameter(reference).map(|s| (name, s))
                }
            })
            .collect::<Vec<_>>();

        items.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

        items
    }

    pub fn get_parameter_at(&self, index: usize) -> Option<(&String, &Parameter)> {
        let items = self.get_parameter_list_items();

        items.get(index).cloned()
    }

    pub fn get_request_bodies_list_items(&self) -> Vec<(&String, &RequestBody)> {
        let Some(spec) = &self.spec else {
            return vec![];
        };

        let Some(components) = spec.components.as_ref() else {
            return vec![];
        };

        let mut items = components
            .request_bodies
            .iter()
            .filter_map(|(name, request_body)| match request_body {
                RequestBodyOrRef::Item(request_body) => Some((name, request_body)),
                RequestBodyOrRef::Ref { reference } => {
                    spec.resolve_request_body(reference).map(|s| (name, s))
                }
            })
            .collect::<Vec<_>>();

        items.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

        items
    }

    pub fn get_request_body_at(&self, index: usize) -> Option<(&String, &RequestBody)> {
        let items = self.get_request_bodies_list_items();

        items.get(index).cloned()
    }

    pub fn get_response_list_items(&self) -> Vec<(&String, &Response)> {
        let Some(spec) = &self.spec else {
            return vec![];
        };

        let Some(components) = spec.components.as_ref() else {
            return vec![];
        };

        let mut items = components
            .responses
            .iter()
            .filter_map(|(name, response)| match response {
                ResponseOrRef::Item(response) => Some((name, response)),
                ResponseOrRef::Ref { reference } => {
                    spec.resolve_response(reference).map(|s| (name, s))
                }
            })
            .collect::<Vec<_>>();

        items.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

        items
    }

    pub fn get_response_at(&self, index: usize) -> Option<(&String, &Response)> {
        let items = self.get_response_list_items();

        items.get(index).cloned()
    }

    pub fn get_headers_list_items(&self) -> Vec<(&String, &Header)> {
        let Some(spec) = &self.spec else {
            return vec![];
        };

        let Some(components) = spec.components.as_ref() else {
            return vec![];
        };

        let mut items = components
            .headers
            .iter()
            .filter_map(|(name, header)| match header {
                HeaderOrRef::Item(header) => Some((name, header)),
                HeaderOrRef::Ref { reference } => spec.resolve_header(reference).map(|s| (name, s)),
            })
            .collect::<Vec<_>>();

        items.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

        items
    }

    pub fn get_header_at(&self, index: usize) -> Option<(&String, &Header)> {
        let items = self.get_headers_list_items();

        items.get(index).cloned()
    }
}

impl From<String> for Source {
    fn from(value: String) -> Self {
        Url::parse(&value).map_or_else(
            |_| Source::File(PathBuf::from(value)),
            |url| Source::Url(url),
        )
    }
}

impl ToString for Source {
    fn to_string(&self) -> String {
        match self {
            Source::File(path) => path.to_string_lossy().to_string(),
            Source::Url(url) => url.to_string(),
        }
    }
}

impl EndpointTab {
    pub fn as_str(&self) -> &'static str {
        match self {
            EndpointTab::Parameters => "Parameters",
            EndpointTab::Responses => "Responses",
            EndpointTab::Body => "Body",
        }
    }
}
