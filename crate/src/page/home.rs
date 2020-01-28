use super::ViewPage;
use crate::{
    entity::{
        article::{self, Article},
        ErrorMessage, PageNumber, PaginatedList, Tag, Viewer,
    },
    loading, logger, page, request, GMsg, Session,
};
use seed::prelude::*;
use std::future::Future;

fn fetch_feed(
    viewer: Option<Viewer>,
    selected_feed: &SelectedFeed,
    page_number: PageNumber,
) -> impl Future<Output = Result<Msg, Msg>> {
    request::feed::load_for_home(
        viewer,
        selected_feed,
        page_number,
        Msg::FeedLoadCompleted,
    )
}

// ------ ------
//     Model
// ------ ------

// ------ Model ------

#[derive(Default)]
pub struct Model {
    session: Session,
    selected_feed: SelectedFeed,
    feed_page: PageNumber,
    tags: Status<Vec<Tag>>,
    feed: Status<article::feed::Model>,
}

impl Model {
    pub const fn session(&self) -> &Session {
        &self.session
    }
}

impl From<Model> for Session {
    fn from(model: Model) -> Self {
        model.session
    }
}

// ------ Status ------

enum Status<T> {
    Loading,
    LoadingSlowly,
    Loaded(T),
    Failed,
}

impl<T> Default for Status<T> {
    fn default() -> Self {
        Self::Loading
    }
}

// ------ SelectedFeed ------

#[derive(Clone)]
pub enum SelectedFeed {
    Your(Viewer),
    Global,
    Tag(Tag),
}

impl<'a> Default for SelectedFeed {
    fn default() -> Self {
        Self::Global
    }
}

// ------ ------
//     Init
// ------ ------

pub fn init(session: Session, orders: &mut impl Orders<Msg, GMsg>) -> Model {
    let selected_feed = session
        .viewer()
        .cloned()
        .map_or_else(SelectedFeed::default, SelectedFeed::Your);

    orders
        .perform_cmd(loading::notify_on_slow_load(Msg::SlowLoadThresholdPassed))
        .perform_cmd(request::tag::load_list(Msg::TagsLoadCompleted))
        .perform_cmd(fetch_feed(
            session.viewer().cloned(),
            &selected_feed,
            PageNumber::default(),
        ));

    Model {
        session,
        selected_feed,
        ..Model::default()
    }
}

// ------ ------
//     Sink
// ------ ------

pub fn sink(g_msg: GMsg, model: &mut Model) {
    match g_msg {
        GMsg::SessionChanged(session) => {
            model.session = session;
        },
        _ => (),
    }
}

// ------ ------
//    Update
// ------ ------

#[derive(Clone)]
#[allow(clippy::pub_enum_variant_names)]
pub enum Msg {
    TagClicked(Tag),
    TabClicked(SelectedFeed),
    FeedPageClicked(PageNumber),
    FeedLoadCompleted(Result<PaginatedList<Article>, Vec<ErrorMessage>>),
    TagsLoadCompleted(Result<Vec<Tag>, Vec<ErrorMessage>>),
    FeedMsg(article::feed::Msg),
    SlowLoadThresholdPassed,
}

pub fn update(
    msg: Msg,
    model: &mut Model,
    orders: &mut impl Orders<Msg, GMsg>,
) {
    match msg {
        Msg::TagClicked(tag) => {
            model.selected_feed = SelectedFeed::Tag(tag);
            model.feed_page = PageNumber::default();
            orders.perform_cmd(fetch_feed(
                model.session.viewer().cloned(),
                &model.selected_feed,
                model.feed_page,
            ));
        },
        Msg::TabClicked(selected_feed) => {
            model.selected_feed = selected_feed;
            model.feed_page = PageNumber::default();
            orders.perform_cmd(fetch_feed(
                model.session.viewer().cloned(),
                &model.selected_feed,
                model.feed_page,
            ));
        },
        Msg::FeedPageClicked(page_number) => {
            model.feed_page = page_number;
            orders.perform_cmd(fetch_feed(
                model.session.viewer().cloned(),
                &model.selected_feed,
                model.feed_page,
            ));
            page::scroll_to_top()
        },
        Msg::FeedLoadCompleted(Ok(paginated_list)) => {
            model.feed = Status::Loaded(article::feed::init(
                model.session.clone(),
                paginated_list,
            ));
        },
        Msg::FeedLoadCompleted(Err(errors)) => {
            model.feed = Status::Failed;
            logger::errors(errors);
        },
        Msg::TagsLoadCompleted(Ok(tags)) => {
            model.tags = Status::Loaded(tags);
        },
        Msg::TagsLoadCompleted(Err(errors)) => {
            model.tags = Status::Failed;
            logger::errors(errors);
        },
        Msg::FeedMsg(feed_msg) => match &mut model.feed {
            Status::Loaded(feed_model) => article::feed::update(
                feed_msg,
                feed_model,
                &mut orders.proxy(Msg::FeedMsg),
            ),
            _ => {
                logger::error("FeedMsg can be handled only if Status is Loaded")
            },
        },
        Msg::SlowLoadThresholdPassed => {
            if let Status::Loading = model.feed {
                model.feed = Status::LoadingSlowly
            }
            if let Status::Loading = model.tags {
                model.tags = Status::LoadingSlowly
            }
        },
    }
}

// ------ ------
//     View
// ------ ------

pub fn view<'a>(model: &Model) -> ViewPage<'a, Msg> {
    ViewPage::new("Conduit", view_content(model))
}

// ====== PRIVATE ======

fn view_content(model: &Model) -> Node<Msg> {
    div![class!["home-page"], view_banner(), view_feed(model),]
}

fn view_banner() -> Node<Msg> {
    div![
        class!["banner"],
        div![
            class!["container"],
            h1![class!["logo-font"], "conduit"],
            p!["A place to share your knowledge."]
        ]
    ]
}

// ------ view feed ------

fn view_feed(model: &Model) -> Node<Msg> {
    match &model.feed {
        Status::Loading => empty![],
        Status::LoadingSlowly => loading::view_icon(),
        Status::Failed => loading::view_error("feed"),
        Status::Loaded(feed_model) => div![
            class!["container", "page"],
            div![
                class!["row"],
                div![
                    class!["col-md-9"],
                    div![
                        class!["feed-toggle"],
                        view_tabs(model),
                        article::feed::view_articles(feed_model)
                            .els()
                            .map_msg(Msg::FeedMsg),
                        article::feed::view_pagination(
                            feed_model,
                            model.feed_page,
                            Msg::FeedPageClicked
                        )
                    ],
                ],
                div![class!["col-md-3"], view_tags(model)]
            ]
        ],
    }
}

fn view_tabs(model: &Model) -> Node<Msg> {
    use crate::entity::article::feed::{view_tabs, Tab};

    let viewer = model.session.viewer();

    // -- Tabs --

    let your_feed = |viewer: Viewer| {
        Tab::new("Your Feed", Msg::TabClicked(SelectedFeed::Your(viewer)))
    };

    let global_feed =
        Tab::new("Global Feed", Msg::TabClicked(SelectedFeed::Global));

    let tag_feed = |tag: Tag| {
        Tab::new(format!("#{}", tag), Msg::TabClicked(SelectedFeed::Tag(tag)))
    };

    // -- View --

    match &model.selected_feed {
        SelectedFeed::Your(viewer) => {
            view_tabs(vec![your_feed(viewer.clone()).activate(), global_feed])
        },
        SelectedFeed::Global => match viewer {
            Some(viewer) => view_tabs(vec![
                your_feed(viewer.clone()),
                global_feed.activate(),
            ]),
            None => view_tabs(vec![global_feed.activate()]),
        },
        SelectedFeed::Tag(tag) => match viewer {
            Some(viewer) => view_tabs(vec![
                your_feed(viewer.clone()),
                global_feed,
                tag_feed(tag.clone()).activate(),
            ]),
            None => {
                view_tabs(vec![global_feed, tag_feed(tag.clone()).activate()])
            },
        },
    }
}

fn view_tags(model: &Model) -> Node<Msg> {
    match &model.tags {
        Status::Loading => empty![],
        Status::LoadingSlowly => loading::view_icon(),
        Status::Failed => loading::view_error("tags"),
        Status::Loaded(tags) => div![
            class!["sidebar"],
            p!["Popular Tags"],
            div![class!["tag-list"], tags.clone().into_iter().map(view_tag)]
        ],
    }
}

fn view_tag(tag: Tag) -> Node<Msg> {
    a![
        class!["tag-pill", "tag-default"],
        attrs! {At::Href => ""},
        tag.to_string(),
        simple_ev(Ev::Click, Msg::TagClicked(tag))
    ]
}
