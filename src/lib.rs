#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_column() {
        assert_eq!(Column::Five.to_usize(), 0x4);
    }
    #[test]
    fn test_move() {
        let mut cf = ConnectFour::new();
        let white = Player::White;
            
        let middle = Box::new(ConnectFourMove { data: Column::Four });
        assert_eq!(middle.data().to_usize(), 0x3);

        for i in 0..6 {
            
            let middle = Box::new(ConnectFourMove { data: Column::Four });

            match cf.make_move(&white, middle) {
                Ok(x) => match x {
                    Score::Undecided => {println!("{}",i); assert!(i<3)},
                    Score::Won => assert!(i>2),
                    _ => assert!(false),
                }
                _ => assert!(i>5),
            }
        }
    }
}


pub enum Player {
    Black,
    White,
}


pub trait Move<T> {
    fn data(&self) -> &T;
}

pub enum Score {
    Undecided,
    Remis,
    Won,
    Lost,
}

pub enum Withdraw {
    NotAllowed,
}

pub trait Game<T> {
    fn make_move(&mut self, p: &Player, m: Box<Move<T>>) -> Result<Score, Withdraw>;
    fn new() -> Self;
}


pub struct ConnectFour {
    field: [[i8;6];7],
}

pub enum Column {
    One, Two, Three, Four, Five, Six, Seven,
}

impl Column {
    fn to_usize(&self) -> usize {
        match &self {
            Column::One => 0x0,
            Column::Two => 0x1,
            Column::Three => 0x2,
            Column::Four => 0x3,
            Column::Five => 0x4,
            Column::Six => 0x5,
            Column::Seven => 0x6,
        }
    }
}

pub struct ConnectFourMove {
    data: Column,
}

impl Move<Column> for ConnectFourMove {
    fn data(&self) -> &Column {
        &self.data
    }
}

impl Game<Column> for ConnectFour {
    fn new() -> Self {
        ConnectFour{
            field: [
                [0,0,0,0,0,0],
                [0,0,0,0,0,0],
                [0,0,0,0,0,0],
                [0,0,0,0,0,0],
                [0,0,0,0,0,0],
                [0,0,0,0,0,0],
                [0,0,0,0,0,0]
            ]
        }
    }

    fn make_move(&mut self, p: &Player, m: Box<Move<Column>>) -> Result<Score, Withdraw> {
        let n = m.data().to_usize();
        
        // set the first free space in the column to 1 or -1, according to Player's color
        for m in 0..6 {
           if self.field[n][m] == 0 {
               self.field[n][m] = match p {
                   Player::White => 1,
                   Player::Black => -1,
               };
               
               return self.get_score(n, m)
           }
        }

        // column is obviously already filled to the top
        Err(Withdraw::NotAllowed)
    }
}

impl ConnectFour {
    fn get_score(&self, n: usize, m: usize) -> Result<Score, Withdraw> {
        let color = self.field[n][m];
        let mut below = 0;
        for i in (0..m).rev() {
            if color == self.field[n][i] { below += 1; }
            else { break; }
        }
        if below >= 3 {
            return Ok(Score::Won)
        }
        Ok(Score::Undecided)
    }
}
