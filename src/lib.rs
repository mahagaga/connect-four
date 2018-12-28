#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_column() {
        assert_eq!(Column::Five.to_usize(), 0x4);
    }
    #[test]
    fn test_move() {
        let white = Player::White;
            
        let middle = Box::new(ConnectFourMove { data: Column::Four });
        assert_eq!(middle.data().to_usize(), 0x3);

        // drop 7 white Stones in the middle column
        let mut cf = ConnectFour::new();
        for i in 0..7 {
            let middle = Box::new(ConnectFourMove { data: Column::Four });

            match cf.make_move(&white, middle) {
                Ok(x) => match x {
                    // should be undecided 3 times
                    Score::Undecided => assert!(i<3,i),
                    // then won 3 times
                    Score::Won => assert!(i>2,i),
                    _ => assert!(false),
                }
                // the 7th stone is one too many
                _ => assert!(i>5),
            }
        }

        // drop 4 stones in a row
        let mut cf = ConnectFour::new();
        match cf.make_move(&white, Box::new(ConnectFourMove { data: Column::Four })) {
            Ok(x) => if let Score::Undecided = x { () } else { assert!(false)},
            _ => assert!(false),
        }
        match cf.make_move(&white, Box::new(ConnectFourMove { data: Column::Two })) {
            Ok(x) => if let Score::Undecided = x { () } else { assert!(false)},
            _ => assert!(false),
        }
        match cf.make_move(&white, Box::new(ConnectFourMove { data: Column::Five })) {
            Ok(x) => if let Score::Undecided = x { () } else { assert!(false)},
            _ => assert!(false),
        }
        match cf.make_move(&white, Box::new(ConnectFourMove { data: Column::Three })) {
            Ok(x) => if let Score::Won = x { () } else { assert!(false)},
            _ => assert!(false),
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum Player {
    Black,
    White,
}

impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Player::Black => write!(f, "{}", String::from("Black")),
            Player::White => write!(f, "{}", String::from("White")),
        }
    }
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
    field: Vec<Vec<Option<Player>>>,
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
        let mut cf = ConnectFour{
            field: Vec::with_capacity(7),
        };
        for _coln in 0..7 {
            let mut col:Vec<Option<Player>> = Vec::with_capacity(6);
            for _celln in 0..6 {
                col.push(None);
            }
            cf.field.push(col);
        };
        cf
    }

    fn make_move(&mut self, p: &Player, mv: Box<Move<Column>>) -> Result<Score, Withdraw> {
        let n = mv.data().to_usize();
        let mut m: usize = 0;

        // set the first free space in the column to the Player's color
        for cell in self.field[n].iter() {
            //println!("{} {}", n, m);
            match cell {
                None => {
                    //println!("none at {}", m);
                    break;
                },
                Some(_other) => {
                    m += 1;
                    //println!("other {}", _other);
                },
           }
        }
        if m == self.field[n].len() {
            // column is obviously already filled to the top
            Err(Withdraw::NotAllowed)
        } else {
            self.field[n][m] = match p {
                Player::White => Some(Player::White),
                Player::Black => Some(Player::Black), 
            };
            self.get_score(p, n, m)
        }
    }
}

enum Step {
    Up,
    Down,
    Plane,
}
impl ConnectFour {
    fn get_score(&self, p: &Player, n: usize, m: usize) -> Result<Score, Withdraw> {
        // vertical
        let below = self.matching_distance(vec![n,n,n], m, Step::Down, p);
        if below >= 3 {
            //println!("below {}", below);
            return Ok(Score::Won)
        }

        // horizontal
        let iter:Vec<usize> = (0..n).rev().collect();
        let right = self.matching_distance(iter, m, Step::Plane, p);
        let iter:Vec<usize> = (n+1..self.field.len()).collect();
        let left = self.matching_distance(iter, m, Step::Plane, p);
        if left + right >= 3 {
            //println!("left {}, right {}", left, right);
            return Ok(Score::Won)
        }

        // diagonal (\)
        let iter:Vec<usize> = (0..n).rev().collect();
        let right = self.matching_distance(iter, m, Step::Up, p);
        let iter:Vec<usize> = (n+1..self.field.len()).collect();
        let left = self.matching_distance(iter, m, Step::Down, p);
        if left + right >= 3 {
            //println!("\\left {}, right {}", left, right);
            return Ok(Score::Won)
        }
        // diagonal (/)
        let iter:Vec<usize> = (0..n).rev().collect();
        let right = self.matching_distance(iter, m, Step::Down, p);
        let iter:Vec<usize> = (n+1..self.field.len()).collect();
        let left = self.matching_distance(iter, m, Step::Up, p);
        if left + right >= 3 {
            //println!("/left {}, right {}", left, right);
            return Ok(Score::Won)
        }

        Ok(Score::Undecided)
    }

    fn matching_distance(&self, 
            iter: Vec<usize>, 
            m: usize,
            step: Step,
            p: &Player) -> usize {
        let mut distance = 1;
        for i in iter.into_iter() {
            let j:usize = match step {
                Step::Up => { if m+distance>=self.field[i].len() { 
                                return distance-1; 
                            }
                            m+distance },
                Step::Down => { if distance>m {
                                return distance-1; 
                            }
                            m-distance },
                Step::Plane => m,
            };
            
            match &self.field[i][j] {
                Some(cp) => {
                    if *cp == *p {
                        //println!("{} {} matches, dist {} up", i, j, distance);
                        distance += 1;
                    } else {
                        break;
                    }
                },
                None => {
                    break;
                }
            }
        }
        distance-1
    }
}
