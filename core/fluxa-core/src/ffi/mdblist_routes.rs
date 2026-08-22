use super::*;

pub(super) fn route_mdblist(method: &str, args_json: &str) -> Outcome {
    match method {
        "mdblistBearer" => Ok(Value::String(mdblist_plan::mdblist_bearer(&arg_str(
            args_json, "token",
        )?))),
        "mdblistDevicePollOutcome" => Ok(Value::String(
            mdblist_plan::mdblist_device_poll_outcome(&arg_str(args_json, "bodyJson")?).to_string(),
        )),

        "mdblistMediaInfoUrl" => {
            let args = object(args_json)?;
            let append = args.get("appendToResponse").and_then(Value::as_str);
            Ok(Value::String(mdblist_plan::mdblist_media_info_url(
                field_str(&args, "provider")?,
                field_str(&args, "mediaType")?,
                field_str(&args, "mediaId")?,
                append,
            )))
        }
        "mdblistWatchproviderLinksUrl" => {
            let args = object(args_json)?;
            Ok(Value::String(
                mdblist_plan::mdblist_watchprovider_links_url(
                    field_str(&args, "provider")?,
                    field_str(&args, "mediaType")?,
                    field_str(&args, "mediaId")?,
                ),
            ))
        }
        "mdblistMediaInfoBatchPlan" => {
            let args = object(args_json)?;
            let ids: Vec<String> = field(&args, "ids")?
                .as_array()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "ids must be an array"))?
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect();
            opt_json(mdblist_plan::mdblist_media_info_batch_plan(
                field_str(&args, "provider")?,
                field_str(&args, "mediaType")?,
                &ids,
            ))
        }
        "mdblistRatingsBatchPlan" => {
            let args = object(args_json)?;
            let ids: Vec<String> = field(&args, "ids")?
                .as_array()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "ids must be an array"))?
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect();
            opt_json(mdblist_plan::mdblist_ratings_batch_plan(
                field_str(&args, "mediaType")?,
                field_str(&args, "returnRating")?,
                field_str(&args, "provider")?,
                &ids,
            ))
        }
        "mdblistMediaRatingsFromResponse" => opt_json(
            mdblist_plan::mdblist_media_ratings_from_response_json(args_json),
        ),
        "mdblistSearchUrl" => {
            let args = object(args_json)?;
            opt_str(mdblist_plan::mdblist_search_url(
                field_str(&args, "mediaType")?,
                args_json,
            ))
        }
        "mdblistGenresUrl" => {
            let args = object(args_json)?;
            Ok(Value::String(mdblist_plan::mdblist_genres_url(
                args.get("anime").and_then(Value::as_bool),
            )))
        }
        "mdblistCatalogUrl" => {
            let args = object(args_json)?;
            opt_str(mdblist_plan::mdblist_catalog_url(
                field_str(&args, "mediaType")?,
                args_json,
            ))
        }

        "mdblistListItemsUrl" => {
            let args = object(args_json)?;
            let list_ref = field(&args, "listRef")?.to_string();
            let query = args
                .get("query")
                .map(Value::to_string)
                .unwrap_or_else(|| "{}".to_string());
            opt_str(mdblist_plan::mdblist_list_items_url(&list_ref, &query))
        }
        "mdblistListItemsResponseToMetas" => opt_json(
            mdblist_plan::mdblist_list_items_response_to_metas_json(args_json),
        ),
        "mdblistListInfoUrl" => Ok(Value::String(mdblist_plan::mdblist_list_info_url(
            field_i64(&object(args_json)?, "listId")?,
        ))),
        "mdblistListByNameUrl" => {
            let args = object(args_json)?;
            Ok(Value::String(mdblist_plan::mdblist_list_by_name_url(
                field_str(&args, "username")?,
                field_str(&args, "listName")?,
            )))
        }
        "mdblistListUpdatePlan" => {
            let args = object(args_json)?;
            opt_json(mdblist_plan::mdblist_list_update_plan(
                field_i64(&args, "listId")?,
                args.get("name").and_then(Value::as_str),
                args.get("private").and_then(Value::as_bool),
            ))
        }
        "mdblistListDeletePlan" => opt_json(mdblist_plan::mdblist_list_delete_plan(field_i64(
            &object(args_json)?,
            "listId",
        )?)),
        "mdblistListCreatePlan" => {
            let args = object(args_json)?;
            opt_json(mdblist_plan::mdblist_list_create_plan(
                field_str(&args, "name")?,
                args.get("private").and_then(Value::as_bool),
            ))
        }
        "mdblistListChangesUrl" => Ok(Value::String(mdblist_plan::mdblist_list_changes_url(
            field_i64(&object(args_json)?, "listId")?,
        ))),
        "mdblistListLikePlan" => {
            let args = object(args_json)?;
            opt_json(mdblist_plan::mdblist_list_like_plan(
                field_i64(&args, "listId")?,
                field(&args, "liked")?
                    .as_bool()
                    .ok_or_else(|| fail(ErrorKind::InvalidArgs, "liked must be a bool"))?,
            ))
        }
        "mdblistListItemsMutatePlan" => {
            let args = object(args_json)?;
            let items_json = field(&args, "items")?.to_string();
            opt_json(mdblist_plan::mdblist_list_items_mutate_plan(
                field_i64(&args, "listId")?,
                field_str(&args, "action")?,
                &items_json,
            ))
        }
        "mdblistListMembershipUrl" => opt_str(mdblist_plan::mdblist_list_membership_url(args_json)),
        "mdblistListsSearchUrl" => opt_str(mdblist_plan::mdblist_lists_search_url(&arg_str(
            args_json, "query",
        )?)),
        "mdblistListsCuratedUrl" => Ok(Value::String(mdblist_plan::mdblist_lists_curated_url(
            args_json,
        ))),
        "mdblistListsTopUrl" => Ok(Value::String(mdblist_plan::mdblist_lists_top_url(
            args_json,
        ))),
        "mdblistListsLikedUrl" => Ok(Value::String(mdblist_plan::mdblist_lists_liked_url(
            args_json,
        ))),
        "mdblistListsOfficialUrl" => Ok(Value::String(mdblist_plan::mdblist_lists_official_url())),
        "mdblistListsRecommendedUrl" => {
            let args = object(args_json)?;
            Ok(Value::String(mdblist_plan::mdblist_lists_recommended_url(
                args.get("section").and_then(Value::as_str),
            )))
        }
        "mdblistListsUserUrl" => {
            let args = object(args_json)?;
            Ok(Value::String(mdblist_plan::mdblist_lists_user_url(
                args.get("userRef").and_then(Value::as_str),
                args_json,
            )))
        }

        "mdblistWatchlistItemsUrl" => {
            let args = object(args_json)?;
            Ok(Value::String(mdblist_plan::mdblist_watchlist_items_url(
                args.get("mediaType").and_then(Value::as_str),
                args_json,
            )))
        }
        "mdblistWatchlistMutatePlan" => {
            let args = object(args_json)?;
            let items_json = field(&args, "items")?.to_string();
            opt_json(mdblist_plan::mdblist_watchlist_mutate_plan(
                field_str(&args, "action")?,
                &items_json,
            ))
        }

        "mdblistSyncGetUrl" => {
            let args = object(args_json)?;
            opt_str(mdblist_plan::mdblist_sync_get_url(
                field_str(&args, "category")?,
                args_json,
            ))
        }
        "mdblistSyncMutatePlan" => {
            let args = object(args_json)?;
            let items_json = field(&args, "items")?.to_string();
            opt_json(mdblist_plan::mdblist_sync_mutate_plan(
                field_str(&args, "category")?,
                field(&args, "remove")?
                    .as_bool()
                    .ok_or_else(|| fail(ErrorKind::InvalidArgs, "remove must be a bool"))?,
                &items_json,
            ))
        }
        "mdblistUpnextUrl" => {
            let args = object(args_json)?;
            opt_str(mdblist_plan::mdblist_upnext_url(
                args.get("section").and_then(Value::as_str),
                args_json,
            ))
        }

        "mdblistScrobblePlan" => {
            let args = object(args_json)?;
            opt_json(mdblist_plan::mdblist_scrobble_plan(
                field_str(&args, "action")?,
                args_json,
            ))
        }
        "mdblistCheckinPlan" => {
            let args = object(args_json)?;
            opt_json(mdblist_plan::mdblist_checkin_plan(
                field_str(&args, "method")?,
                args_json,
            ))
        }

        "mdblistDiscussionUrl" => {
            let args = object(args_json)?;
            Ok(Value::String(mdblist_plan::mdblist_discussion_url(
                field_str(&args, "provider")?,
                field_str(&args, "targetType")?,
                field_i64(&args, "targetId")?,
            )))
        }
        "mdblistDiscussionSummaryUrl" => {
            let args = object(args_json)?;
            Ok(Value::String(mdblist_plan::mdblist_discussion_summary_url(
                field_str(&args, "provider")?,
                field_str(&args, "targetType")?,
                field_i64(&args, "targetId")?,
            )))
        }
        "mdblistDiscussionCreatePlan" => {
            let args = object(args_json)?;
            opt_json(mdblist_plan::mdblist_discussion_create_plan(
                field_str(&args, "provider")?,
                field_str(&args, "targetType")?,
                field_i64(&args, "targetId")?,
                field_str(&args, "comment")?,
            ))
        }
        "mdblistDiscussionHotUrl" => Ok(Value::String(mdblist_plan::mdblist_discussion_hot_url())),
        "mdblistDiscussionRepliesUrl" => {
            let args = object(args_json)?;
            Ok(Value::String(mdblist_plan::mdblist_discussion_replies_url(
                field_i64(&args, "commentId")?,
                args_json,
            )))
        }
        "mdblistDiscussionReplyCreatePlan" => {
            let args = object(args_json)?;
            opt_json(mdblist_plan::mdblist_discussion_reply_create_plan(
                field_i64(&args, "commentId")?,
                field_str(&args, "comment")?,
            ))
        }
        "mdblistDiscussionCommentUpdatePlan" => {
            let args = object(args_json)?;
            opt_json(mdblist_plan::mdblist_discussion_comment_update_plan(
                field_i64(&args, "commentId")?,
                field_str(&args, "comment")?,
            ))
        }
        "mdblistDiscussionCommentDeletePlan" => {
            opt_json(mdblist_plan::mdblist_discussion_comment_delete_plan(
                field_i64(&object(args_json)?, "commentId")?,
            ))
        }
        "mdblistDiscussionCommentLikePlan" => {
            opt_json(mdblist_plan::mdblist_discussion_comment_like_plan(
                field_i64(&object(args_json)?, "commentId")?,
            ))
        }
        "mdblistDiscussionReplyUpdatePlan" => {
            let args = object(args_json)?;
            opt_json(mdblist_plan::mdblist_discussion_reply_update_plan(
                field_i64(&args, "replyId")?,
                field_str(&args, "comment")?,
            ))
        }
        "mdblistDiscussionReplyDeletePlan" => {
            opt_json(mdblist_plan::mdblist_discussion_reply_delete_plan(
                field_i64(&object(args_json)?, "replyId")?,
            ))
        }
        "mdblistDiscussionReplyLikePlan" => {
            opt_json(mdblist_plan::mdblist_discussion_reply_like_plan(field_i64(
                &object(args_json)?,
                "replyId",
            )?))
        }

        "mdblistUserUrl" => Ok(Value::String(mdblist_plan::mdblist_user_url())),
        "mdblistUserStatsUrl" => Ok(Value::String(mdblist_plan::mdblist_user_stats_url())),
        "mdblistPublicUserUrl" => Ok(Value::String(mdblist_plan::mdblist_public_user_url(
            &arg_str(args_json, "username")?,
        ))),
        "mdblistUserFollowPlan" => {
            let args = object(args_json)?;
            opt_json(mdblist_plan::mdblist_user_follow_plan(
                field_str(&args, "username")?,
                field(&args, "follow")?
                    .as_bool()
                    .ok_or_else(|| fail(ErrorKind::InvalidArgs, "follow must be a bool"))?,
            ))
        }

        _ => Err(unknown_method()),
    }
}
