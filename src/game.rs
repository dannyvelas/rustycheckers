use super::board::{Coordinate, GamePiece, Move, PieceColor};

pub struct GameEngine {
    board: [[Option<GamePiece>; 8]; 8],
    current_turn: PieceColor,
    move_count: u32,
}

pub struct MoveResult {
    pub mv: Move,
    pub is_crowned: bool,
}


impl GameEngine {
    pub fn new() -> GameEngine {
        let mut engine = GameEngine {
            board: [[None; 8]; 8],
            current_turn: PieceColor::Black,
            move_count: 0,
        };
        engine.initialize_pieces();
        engine
    }

    pub fn initialize_pieces(&mut self) {
        [1, 3, 5, 7, 0, 2, 4, 6, 1, 3, 5, 7]
            .iter()
            .zip([0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2].iter())
            .map(|(a,b)| (*a as usize, *b as usize))
            .for_each(|(x,y)| {
                self.board[x][y] = Some(GamePiece::new(PieceColor::White));
            });

        [0, 2, 4, 6, 1, 3, 5, 7, 0, 2, 4, 6]
            .iter()
            .zip([5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7].iter())
            .map(|(a,b)| (*a as usize, *b as usize))
            .for_each(|(x,y)| {
                self.board[x][y] = Some(GamePiece::new(PieceColor::Black));
            });
    }

    pub fn move_piece(&mut self, mv: &Move) -> Result<MoveResult, ()> {
        let legal_moves = self.legal_moves();

        if !legal_moves.contains(mv) {
            return Err(());
        }

        let Coordinate(fx, fy) = mv.from;
        let Coordinate(tx, ty) = mv.to;
        let piece = self.board[fx][fy].unwrap();
        let midpiece_coordinate = self.midpiece_coordinate(fx, fy, tx, ty);

        if let Some(Coordinate(x,y)) = midpiece_coordinate {
            self.board[x][y] = None; // remove jumped piece
        }

        // Move piece from source to dest
        self.board[fx][fy] = None;
        self.board[tx][ty] = Some(piece);

        let is_crowned = if self.should_crown(piece, mv.to) {
            self.crown_piece(mv.to);
            true
        } else {
            false
        };

        self.advance_turn();

        Ok(MoveResult {
            mv: mv.clone(),
            is_crowned
        })
    }

    pub fn get_piece(&self, coord: Coordinate) -> Result<Option<GamePiece>, ()> {
        let Coordinate(x, y) = coord;
        if x <= 7 && y <= 7 {
            Ok(self.board[x][y])
        } else {
            Err(())
        }
    }

    pub fn get_current_turn(&self) -> PieceColor {
        self.current_turn
    }

    fn advance_turn(&mut self) {
        if self.current_turn == PieceColor::Black {
            self.current_turn = PieceColor::White
        } else {
            self.current_turn = PieceColor::Black
        }
        self.move_count += 1;
    }

    fn should_crown(&self, piece: GamePiece, coordinate: Coordinate) -> bool {
        let Coordinate(_, y) = coordinate;

        (y == 0 && piece.color == PieceColor::Black) || (y == 7 && piece.color == PieceColor::White)
    }

    fn crown_piece(&mut self, coordinate: Coordinate) -> bool {
        let Coordinate(x, y) = coordinate;

        if let Some(piece) = self.board[x][y] {
            self.board[x][y] = Some(GamePiece::crowned(piece));
            true
        } else {
            false
        }
    }

    fn legal_moves(&self) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();

        for col in 0..8 {
            for row in 0..8 {
                if let Some(piece) = self.board[col][row] {
                    if piece.color == self.current_turn {
                        let loc = Coordinate(col, row);
                        let mut vmoves = self.valid_moves_from(loc);
                        moves.append(&mut vmoves);
                    }
                }
            }
        }

        moves
    }

    fn valid_moves_from(&self, loc: Coordinate) -> Vec<Move> {
        let Coordinate(x, y) = loc;
        if let Some(p) = self.board[x][y] {
            let mut jumps = loc
                .jump_targets_from()
                .filter(|t| self.valid_jump(&p, &loc, &t))
                .map(|ref t| Move {
                    from: loc.clone(),
                    to: t.clone(),
                }).collect::<Vec<Move>>();

            let mut moves = loc
                .move_targets_from()
                .filter(|t| self.valid_move(&p, &loc, &t))
                .map(|ref t| Move {
                    from: loc.clone(),
                    to: t.clone(),
                }).collect::<Vec<Move>>();
            jumps.append(&mut moves);
            jumps
        } else {
            Vec::new()
        }
    }
    
    fn midpiece_coordinate(&self, fx: usize, fy: usize, tx: usize, ty: usize) -> Option<Coordinate> {
        if tx == fx + 2 && ty == ty + 2 {
            Some(Coordinate(fx + 1, ty + 1))
        } else if fx >= 2 && fy >= 2 && tx == fx - 2 && ty == fy - 2 {
            Some(Coordinate(fx - 1, fy - 1)) 
        } else if tx >= 2 && tx == fx - 2 && ty == fy + 2 {
            Some(Coordinate(tx - 1, ty + 1))
        } else if ty >= 2 && tx == tx + 2 && ty == ty - 2 {
            Some(Coordinate(tx + 1, ty - 1))
        } else {
            None
        }
    }

    fn midpiece(&self, fx: usize, fy: usize, tx: usize, ty: usize) -> Option<GamePiece> {
        match self.midpiece_coordinate(fx, fy, tx, ty) {
            Some(Coordinate(x, y)) => self.board[x][y],
            None => None,
        }
    }

    fn valid_jump(&self, piece: &GamePiece, from: &Coordinate, to: &Coordinate) -> bool {
        if !to.on_board() || !from.on_board() {
            false
        } else {
            let Coordinate(fx, fy) = *from;
            let Coordinate(tx, ty) = *to;

            let midpiece = self.midpiece(fx, fy, tx, ty);
            match midpiece {
                Some(mp) if mp.color != piece.color => true,
                _ => false,
            }
        }
    }

    fn valid_move(&self, piece: &GamePiece, from: &Coordinate, to: &Coordinate) -> bool {
        if !to.on_board() || !from.on_board() {
            false
        } else {
            let Coordinate(tx, ty) = *to;
            if let Some(_piece) = self.board[tx][ty] {
                false
            } else {
                let Coordinate(_, fy) = *from;
                if ty > fy && piece.color == PieceColor::White {
                    true
                } else if ty < fy && piece.color == PieceColor::Black {
                    // black moves up
                    true
                } else if ty > fy && piece.color == PieceColor::Black && piece.is_crowned {
                    // crowned black moves down
                    true
                } else if ty < fy && piece.color == PieceColor::White && piece.is_crowned {
                    // crowned white moves up
                    true
                } else {
                    false
                }
            }
        }
    }
}
