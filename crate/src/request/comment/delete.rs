use std::future::Future;

use indexmap::IndexMap;
use seed::fetch::{Method, ResponseDataResult};

use crate::{
    entity::{CommentId, ErrorMessage, Slug, Viewer},
    request,
};

type RootDecoder = IndexMap<(), ()>;

pub fn delete<Ms: 'static>(
    viewer: Option<&Viewer>,
    slug: &Slug,
    comment_id: CommentId,
    f: fn(Result<CommentId, Vec<ErrorMessage>>) -> Ms,
) -> impl Future<Output = Result<Ms, Ms>> {
    request::new(
        &format!("articles/{}/comments/{}", slug.as_str(), comment_id.as_str()),
        viewer,
    )
    .method(Method::Delete)
    .fetch_json_data(
        move |data_result: ResponseDataResult<RootDecoder>| {
            f(data_result
                .map(move |_| comment_id)
                .map_err(request::fail_reason_into_errors))
        },
    )
}
