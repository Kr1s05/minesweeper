use std::collections::{HashSet, VecDeque};
use std::fmt::{Debug, Formatter};
use rand::Rng;
use crate::Cell::*;
use crate::Item::*;
use iced::{color, Element, Font, Theme};
use iced::widget::{column, container, mouse_area, row, svg, text, MouseArea, Space};

// svg::Handle::from_memory(include_bytes!("../resources/mine.svg").as_slice())


// svg::Handle::from_memory(include_bytes!("../resources/flag.svg").as_slice())


struct Game {
    field: Vec<Vec<Cell>>,
    state: GameState,
    bombs: Vec<(usize, usize)>,
    pins: Vec<(usize, usize)>,
    hidden_empty: u32,
}
#[derive(Debug)]
enum GameState {
    Playing,
    Win,
    Loss,
}

#[derive(Clone, Debug)]
enum Item {
    Bomb,
    Empty(u8),
}
#[derive(Clone, Debug)]
enum Cell {
    Hidden(Item),
    Pin(Item),
    Discovered(Item),
}
#[derive(Debug, Clone)]
enum Message {}

impl Cell {
    fn get_item(&self) -> Item {
        match self {
            Hidden(item) | Pin(item) | Discovered(item) => item.clone()
        }
    }
    fn turn_discovered(&self) -> Cell {
        Discovered(self.get_item())
    }
    fn to_button(&self) -> MouseArea<Message> {
        let content = match self {
            Hidden(_) => {
                mouse_area(text("#")
                    .width(20)
                    .height(20)
                )
            }
            Pin(_) => {
                mouse_area(svg(svg::Handle::from_memory(include_bytes!("../resources/flag.svg").as_slice()))
                    .width(20)
                    .height(20)
                )
            }
            Discovered(item) => {
                match item {
                    Bomb => {
                        mouse_area(svg(svg::Handle::from_memory(include_bytes!("../resources/mine.svg").as_slice()))
                            .width(20)
                            .height(20)
                        )
                    }
                    Empty(count) => {
                        if *count == 0 {
                            mouse_area(Space::new(20, 20))
                        } else {
                            println!("{}",count.to_string());
                            mouse_area(text(count.to_string())
                                .width(20)
                                .height(20)
                            )
                        }
                    }
                }
            }
        };
        content
    }
}

impl Debug for Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut field_str = String::from("");
        for row in &self.field {
            for cell in row {
                match cell {
                    Hidden(_) => {
                        field_str.push_str(" _ ")
                    }
                    Pin(_) => {
                        field_str.push_str(" I ")
                    }
                    Discovered(item) => {
                        match item {
                            Bomb => {
                                field_str.push_str(" H ")
                            }
                            Empty(i) => {
                                field_str.push_str(" ");
                                field_str.push_str(&*i.to_string());
                                field_str.push_str(" ")
                            }
                        }
                    }
                }
            }
            field_str.push_str("\n");
        }
        write!(f, "State: {:?}; Field:\n{}", self.state, field_str)
    }
}

impl Default for Game {
    fn default() -> Self { 
        let mut g =  Game::new(16, 16);
        let _ = &g.discover(&7,&7);
        g
    }
}

impl Game {
    fn new(width: usize, height: usize) -> Game {
        let mut rng = rand::thread_rng();
        let bomb_budget = width * height * 15 / 100;
        let mut field = vec![vec![Hidden(Empty(0)); width]; height];
        let mut bombs: HashSet<(usize, usize)> = HashSet::new();
        while bombs.len() < bomb_budget {
            let x = rng.gen_range(0..width);
            let y = rng.gen_range(0..height);
            if bombs.insert((x, y)) {
                let row_start = if y > 0 { y - 1 } else { y };
                let row_end = (y + 1).min(height - 1);
                let col_start = if x > 0 { x - 1 } else { x };
                let col_end = (x + 1).min(width - 1);
                for i in row_start..row_end + 1 {
                    for j in col_start..col_end + 1 {
                        let c = &mut field[i][j];
                        if i == y && j == x {
                            *c = Hidden(Bomb);
                        } else if let Hidden(Empty(count)) = c {
                            *count += 1;
                        }
                    }
                }
            }
        }
        let pins = Vec::new();
        Game {
            field,
            state: GameState::Playing,
            bombs: bombs.into_iter().collect(),
            pins,
            hidden_empty: (height * width - bomb_budget) as u32,
        }
    }

    fn discover(&mut self, x: &usize, y: &usize) {
        let field = &mut self.field;
        match &field[*y][*x] {
            Hidden(item) => {
                match item {
                    Bomb => {
                        self.state = GameState::Loss;
                        for bomb in &self.bombs {
                            self.field[bomb.1][bomb.0] = Discovered(Bomb)
                        }
                    }
                    Empty(score) => {
                        if *score > 0 {
                            field[*y][*x] = field[*y][*x].turn_discovered();
                            self.hidden_empty -= 1;
                            return;
                        }
                        fill_area(field, *y, *x, &mut self.hidden_empty);
                    }
                }
            }
            _ => return
        }
    }

    fn pin(&mut self, x: usize, y: usize) {
        let cell = &mut self.field[y][x];
        if let Hidden(_) = cell {
            *cell = Pin(cell.get_item());
            self.pins.push((x, y));
        }
    }

    fn update_game_state(&mut self) {
        if self.hidden_empty == 0 {
            self.state = GameState::Win;
        }
    }
}

fn fill_area(field: &mut Vec<Vec<Cell>>, start_row: usize, start_col: usize, hidden_count: &mut u32) {
    let cols = field[0].len() as isize;
    let rows = field.len() as isize;
    let directions = [(1, 0), (0, 1), (-1, 0), (0, -1), (1, 1), (-1, -1), (1, -1), (-1, 1)];
    let mut explored = vec![vec![false; cols as usize]; rows as usize];
    let mut queue = VecDeque::new();
    queue.push_back((start_row, start_col));
    while let Some((row, col)) = queue.pop_front() {
        field[row][col] = field[row][col].turn_discovered();
        *hidden_count -= 1;
        if let Empty(count) = field[row][col].get_item() {
            if count > 0 {
                continue;
            }
            for &(r_offset, c_offset) in &directions {
                let row = row as isize + r_offset;
                let col = col as isize + c_offset;
                if row < 0 || row >= rows || col < 0 || col >= cols {
                    continue;
                }
                let (r, c) = (row as usize, col as usize);
                if let Hidden(Empty(_)) = field[r][c] {
                    if !explored[r][c] {
                        queue.push_back((r, c));
                        explored[r][c] = true;
                    }
                }
            }
        }
    }
}

fn update(game: &mut Game, message: Message) {}

fn view(game: &Game) -> Element<Message> {
    container(
        column(
            game.field.iter().map(|r| {
                row(r.iter().map(|cell| cell.to_button().into())).into()
            })
        )
    ).into()
}

fn main() -> iced::Result {
    iced::application("Minesweeper",update,view)
        .centered()
        .resizable(false)
        .window_size(iced::Size::new(600f32,800f32))
        .theme(|_| Theme::Light)
        .run()
}
