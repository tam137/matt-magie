use crate::fen_service::FenService;
use crate::move_gen_service::MoveGenService;

pub struct Service {
    pub fen: FenService,
    pub move_gen: MoveGenService,
}

impl Service {
    pub fn new() -> Self {
        Service {
            fen: FenService,
            move_gen: MoveGenService::new(),
        }
    }
}
