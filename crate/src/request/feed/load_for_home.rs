use crate::{
    coder::decoder,
    entity::{Article, ErrorMessage, PageNumber, PaginatedList, Viewer},
    logger,
    page::home::SelectedFeed,
    request,
};
use lazy_static::lazy_static;
use seed::fetch::ResponseDataResult;
use serde::Deserialize;
use std::{borrow::Cow, future::Future, num::NonZeroUsize};

lazy_static! {
    static ref ARTICLES_PER_PAGE: NonZeroUsize = NonZeroUsize::new(10).unwrap();
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RootDecoder {
    articles: Vec<decoder::Article>,
    articles_count: usize,
}

impl RootDecoder {
    fn into_paginated_list(
        self,
        viewer: &Option<Viewer>,
    ) -> PaginatedList<Article> {
        PaginatedList {
            items: self
                .articles
                .into_iter()
                .filter_map(|article_decoder| {
                    match article_decoder
                        .try_into_article(viewer.as_ref().map(Cow::Borrowed))
                    {
                        Ok(article) => Some(article),
                        Err(error) => {
                            logger::error(error);
                            None
                        },
                    }
                })
                .collect(),
            per_page: *ARTICLES_PER_PAGE,
            total: self.articles_count,
        }
    }
}

pub fn request_url(
    selected_feed: &SelectedFeed,
    page_number: PageNumber,
) -> String {
    let (path, tag_param) = match selected_feed {
        SelectedFeed::Your(_) => (Some("/feed"), None),
        SelectedFeed::Global => (None, None),
        SelectedFeed::Tag(tag) => (None, Some(format!("tag={}", tag))),
    };

    let mut parameters = vec![
        format!("limit={}", *ARTICLES_PER_PAGE),
        format!("offset={}", (*page_number - 1) * ARTICLES_PER_PAGE.get()),
    ];
    if let Some(tag_param) = tag_param {
        parameters.push(tag_param)
    }
    format!("articles{}?{}", path.unwrap_or_default(), parameters.join("&"))
}

pub fn load_for_home<Ms: 'static>(
    viewer: Option<Viewer>,
    selected_feed: &SelectedFeed,
    page_number: PageNumber,
    f: fn(Result<PaginatedList<Article>, Vec<ErrorMessage>>) -> Ms,
) -> impl Future<Output = Result<Ms, Ms>> {
    request::new(&request_url(selected_feed, page_number), viewer.as_ref())
        .fetch_json_data(move |data_result: ResponseDataResult<RootDecoder>| {
            f(data_result
                .map(move |root_decoder| {
                    root_decoder.into_paginated_list(&viewer)
                })
                .map_err(request::fail_reason_into_errors))
        })
}
