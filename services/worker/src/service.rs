use std::sync::Arc;

use crate::{job, prelude::*, rpc, RpcWorker};
use rpc::worker_server::Worker;
use tonic::{Request, Response, Status};

#[tonic::async_trait]
impl Worker for RpcWorker {
    async fn add_game(
        &self,
        request: Request<rpc::AddGameRequest>,
    ) -> StdResult<Response<()>, Status> {
        let rpc::AddGameRequest { game_id, game_name } = request.into_inner();
        debug!("Add game {game_id} ({game_name})");

        let db = self.g.db.lock().await;
        db::game::insert(&db, &(game_id.into()), game_name)?;

        Ok(Response::new(()))
    }

    async fn delete_game(
        &self,
        request: Request<rpc::DeleteGameRequest>,
    ) -> StdResult<Response<()>, Status> {
        let rpc::DeleteGameRequest { game_id } = request.into_inner();
        debug!("Delete game {game_id}");

        let db = self.g.db.lock().await;
        db::game::delete(&db, &(game_id.into()))?;

        Ok(Response::new(()))
    }

    async fn set_is_game_paused(
        &self,
        request: Request<rpc::SetIsGamePausedRequest>,
    ) -> StdResult<Response<()>, Status> {
        let rpc::SetIsGamePausedRequest { game_id, is_paused } =
            request.into_inner();
        debug!("Set paused to {is_paused} for game {game_id}");

        let db = self.g.db.lock().await;
        db::game::set_is_paused(&db, &(game_id.into()), is_paused)?;

        Ok(Response::new(()))
    }

    async fn trigger_fetch_new_game_clips(
        &self,
        request: Request<rpc::TriggerFetchNewGameClipsRequest>,
    ) -> StdResult<Response<()>, Status> {
        let rpc::TriggerFetchNewGameClipsRequest {
            game_id,
            recorded_at_most_hours_ago,
            recorded_at_least_hours_ago,
        } = request.into_inner();

        let conf = job::fetch_new_game_clips::Conf {
            recorded_at_most_ago: if recorded_at_most_hours_ago == 0 {
                None
            } else {
                Some(chrono::Duration::hours(recorded_at_most_hours_ago))
            },
            recorded_at_least_ago: if recorded_at_least_hours_ago == 0 {
                None
            } else {
                Some(chrono::Duration::hours(recorded_at_least_hours_ago))
            },
        };

        let db = Arc::clone(&self.g.db);
        let tc = Arc::clone(&self.g.twitch);

        if let Some(game_id) = game_id {
            debug!("Trigger fetch new game clips for game {game_id}");

            tokio::spawn(job::fetch_new_game_clips::once(
                db,
                tc,
                conf,
                game_id.into(),
            ));
        } else {
            debug!("Trigger fetch new game clips for all active games");

            tokio::spawn(job::fetch_new_game_clips::once_for_all(db, tc, conf));
        };

        Ok(Response::new(()))
    }

    async fn list_clips(
        &self,
        request: Request<rpc::ListClipsRequest>,
    ) -> StdResult<Response<rpc::ListClipsResponse>, Status> {
        debug!("List clips request {request:?}");

        let db = self.g.db.lock().await;
        let (total_count, clips) = db::clip::list(&db, request.into_inner())?;

        Ok(Response::new(rpc::ListClipsResponse {
            total_count: total_count as i64,
            clips: clips.into_iter().map(From::from).collect(),
        }))
    }

    async fn dev_reset(
        &self,
        _request: Request<()>,
    ) -> StdResult<Response<()>, Status> {
        debug!("Dev reset");

        let mut db = self.g.db.lock().await;
        db::down(&mut db)
            .map_err(|e| Status::internal(format!("Failed to down db: {e}")))?;

        db::up(&mut db)
            .map_err(|e| Status::internal(format!("Failed to up db: {e}")))?;

        Ok(Response::new(()))
    }
}
