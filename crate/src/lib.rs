#![allow(
    clippy::single_match,
    clippy::large_enum_variant,
    clippy::must_use_candidate
)]
#![allow(clippy::default_trait_access)] // because of problem with `strum_macros::EnumIter`

#[macro_use]
extern crate seed;
use entity::{article, username};
use helper::take;
use seed::prelude::*;
use std::convert::TryInto;

pub use route::Route;
pub use session::Session;

mod coder;
mod entity;
mod helper;
mod loading;
mod logger;
mod page;
mod request;
mod route;
mod session;
mod storage;

// ------ ------
//     Model
// ------ ------

enum Model<'a> {
    Redirect(Session),
    NotFound(Session),
    Home(page::home::Model),
    Settings(page::settings::Model),
    Login(page::login::Model),
    Register(page::register::Model),
    Profile(page::profile::Model<'a>, username::Username<'a>),
    Article(page::article::Model),
    ArticleEditor(page::article_editor::Model, Option<article::slug::Slug>),
}

impl<'a> Default for Model<'a> {
    fn default() -> Self {
        Model::Redirect(Session::default())
    }
}

impl<'a> From<Model<'a>> for Session {
    fn from(model: Model<'a>) -> Self {
        use Model::*;
        match model {
            Redirect(session) | NotFound(session) => session,
            Home(model) => model.into(),
            Settings(model) => model.into(),
            Login(model) => model.into(),
            Register(model) => model.into(),
            Profile(model, _) => model.into(),
            Article(model) => model.into(),
            ArticleEditor(model, _) => model.into(),
        }
    }
}

// ------ ------
// Before Mount
// ------ ------

fn before_mount(_: Url) -> BeforeMount {
    // Since we have the "loading..." text in the app section of index.html,
    // we use MountType::Takover which will overwrite it with the seed generated html
    BeforeMount::new().mount_type(MountType::Takeover)
}

// ------ ------
//  After Mount
// ------ ------

fn after_mount(
    url: Url,
    orders: &mut impl Orders<Msg<'static>, GMsg>,
) -> AfterMount<Model<'static>> {
    orders.send_msg(Msg::RouteChanged(url.try_into().ok()));

    let model = Model::Redirect(Session::new(storage::load_viewer()));
    AfterMount::new(model).url_handling(UrlHandling::None)
}

// ------ ------
//     Sink
// ------ ------

pub enum GMsg {
    RoutePushed(Route<'static>),
    SessionChanged(Session),
}

fn sink<'a>(
    g_msg: GMsg,
    model: &mut Model<'a>,
    orders: &mut impl Orders<Msg<'static>, GMsg>,
) {
    if let GMsg::RoutePushed(ref route) = g_msg {
        orders.send_msg(Msg::RouteChanged(Some(route.clone())));
    }

    match model {
        Model::NotFound(_) | Model::Redirect(_) => {
            if let GMsg::SessionChanged(session) = g_msg {
                *model = Model::Redirect(session);
                route::go_to(Route::Home, orders);
            }
        },
        Model::Settings(model) => {
            page::settings::sink(
                g_msg,
                model,
                &mut orders.proxy(Msg::SettingsMsg),
            );
        },
        Model::Home(model) => {
            page::home::sink(g_msg, model);
        },
        Model::Login(model) => {
            page::login::sink(g_msg, model, &mut orders.proxy(Msg::LoginMsg));
        },
        Model::Register(model) => {
            page::register::sink(
                g_msg,
                model,
                &mut orders.proxy(Msg::RegisterMsg),
            );
        },
        Model::Profile(model, _) => {
            page::profile::sink(
                g_msg,
                model,
                &mut orders.proxy(Msg::ProfileMsg),
            );
        },
        Model::Article(model) => {
            page::article::sink(
                g_msg,
                model,
                &mut orders.proxy(Msg::ArticleMsg),
            );
        },
        Model::ArticleEditor(model, _) => {
            page::article_editor::sink(
                g_msg,
                model,
                &mut orders.proxy(Msg::ArticleEditorMsg),
            );
        },
    }
}

// ------ ------
//    Update
// ------ ------

#[allow(clippy::enum_variant_names)]
enum Msg<'a> {
    RouteChanged(Option<Route<'a>>),
    HomeMsg(page::home::Msg),
    SettingsMsg(page::settings::Msg),
    LoginMsg(page::login::Msg),
    RegisterMsg(page::register::Msg),
    ProfileMsg(page::profile::Msg),
    ArticleMsg(page::article::Msg),
    ArticleEditorMsg(page::article_editor::Msg),
}

fn update<'a>(
    msg: Msg<'a>,
    model: &mut Model<'a>,
    orders: &mut impl Orders<Msg<'static>, GMsg>,
) {
    match msg {
        Msg::RouteChanged(route) => {
            change_model_by_route(route, model, orders);
        },
        Msg::HomeMsg(module_msg) => {
            if let Model::Home(module_model) = model {
                page::home::update(
                    module_msg,
                    module_model,
                    &mut orders.proxy(Msg::HomeMsg),
                );
            }
        },
        Msg::SettingsMsg(module_msg) => {
            if let Model::Settings(module_model) = model {
                page::settings::update(
                    module_msg,
                    module_model,
                    &mut orders.proxy(Msg::SettingsMsg),
                );
            }
        },
        Msg::LoginMsg(module_msg) => {
            if let Model::Login(module_model) = model {
                page::login::update(
                    module_msg,
                    module_model,
                    &mut orders.proxy(Msg::LoginMsg),
                );
            }
        },
        Msg::RegisterMsg(module_msg) => {
            if let Model::Register(module_model) = model {
                page::register::update(
                    module_msg,
                    module_model,
                    &mut orders.proxy(Msg::RegisterMsg),
                );
            }
        },
        Msg::ProfileMsg(module_msg) => {
            if let Model::Profile(module_model, _) = model {
                page::profile::update(
                    module_msg,
                    module_model,
                    &mut orders.proxy(Msg::ProfileMsg),
                );
            }
        },
        Msg::ArticleMsg(module_msg) => {
            if let Model::Article(module_model) = model {
                page::article::update(
                    module_msg,
                    module_model,
                    &mut orders.proxy(Msg::ArticleMsg),
                );
            }
        },
        Msg::ArticleEditorMsg(module_msg) => {
            if let Model::ArticleEditor(module_model, _) = model {
                page::article_editor::update(
                    module_msg,
                    module_model,
                    &mut orders.proxy(Msg::ArticleEditorMsg),
                );
            }
        },
    }
}

fn change_model_by_route<'a>(
    route: Option<Route<'a>>,
    model: &mut Model<'a>,
    orders: &mut impl Orders<Msg<'static>, GMsg>,
) {
    let mut session = || Session::from(take(model));
    match route {
        None => *model = Model::NotFound(session()),
        Some(route) => match route {
            Route::Root => route::go_to(Route::Home, orders),
            Route::Logout => {
                storage::delete_app_data();
                orders.send_g_msg(GMsg::SessionChanged(Session::Guest));
                route::go_to(Route::Home, orders)
            },
            Route::NewArticle => {
                *model = Model::ArticleEditor(
                    page::article_editor::init_new(session()),
                    None,
                );
            },
            Route::EditArticle(slug) => {
                *model = Model::ArticleEditor(
                    page::article_editor::init_edit(
                        session(),
                        slug.clone(),
                        &mut orders.proxy(Msg::ArticleEditorMsg),
                    ),
                    Some(slug),
                );
            },
            Route::Settings => {
                *model = Model::Settings(page::settings::init(
                    session(),
                    &mut orders.proxy(Msg::SettingsMsg),
                ));
            },
            Route::Home => {
                *model = Model::Home(page::home::init(
                    session(),
                    &mut orders.proxy(Msg::HomeMsg),
                ));
            },
            Route::Login => {
                *model = Model::Login(page::login::init(session()));
            },
            Route::Register => {
                *model = Model::Register(page::register::init(session()));
            },
            Route::Profile(username) => {
                *model = Model::Profile(
                    page::profile::init(
                        session(),
                        username.to_static(),
                        &mut orders.proxy(Msg::ProfileMsg),
                    ),
                    username.into_owned(),
                );
            },
            Route::Article(slug) => {
                *model = Model::Article(page::article::init(
                    session(),
                    &slug,
                    &mut orders.proxy(Msg::ArticleMsg),
                ));
            },
        },
    };
}

// ------ ------
//     View
// ------ ------

fn view(model: &Model) -> impl View<Msg<'static>> {
    use page::Page;
    match model {
        Model::Redirect(session) => {
            Page::Other.view(page::blank::view(), session.viewer())
        },
        Model::NotFound(session) => {
            Page::Other.view(page::not_found::view(), session.viewer())
        },
        Model::Settings(model) => Page::Settings
            .view(page::settings::view(model), model.session().viewer())
            .map_msg(Msg::SettingsMsg),
        Model::Home(model) => Page::Home
            .view(page::home::view(model), model.session().viewer())
            .map_msg(Msg::HomeMsg),
        Model::Login(model) => Page::Login
            .view(page::login::view(model), model.session().viewer())
            .map_msg(Msg::LoginMsg),
        Model::Register(model) => Page::Register
            .view(page::register::view(model), model.session().viewer())
            .map_msg(Msg::RegisterMsg),
        Model::Profile(model, username) => Page::Profile(username)
            .view(page::profile::view(model), model.session().viewer())
            .map_msg(Msg::ProfileMsg),
        Model::Article(model) => Page::Other
            .view(page::article::view(model), model.session().viewer())
            .map_msg(Msg::ArticleMsg),
        Model::ArticleEditor(model, None) => Page::NewArticle
            .view(page::article_editor::view(model), model.session().viewer())
            .map_msg(Msg::ArticleEditorMsg),
        Model::ArticleEditor(model, Some(_)) => Page::Other
            .view(page::article_editor::view(model), model.session().viewer())
            .map_msg(Msg::ArticleEditorMsg),
    }
}

// ------ ------
//     Start
// ------ ------

#[wasm_bindgen(start)]
pub fn start() {
    App::builder(update, view)
        .before_mount(before_mount)
        .after_mount(after_mount)
        .routes(|url| Some(Msg::RouteChanged(url.try_into().ok())))
        .sink(sink)
        .build_and_start();
}
