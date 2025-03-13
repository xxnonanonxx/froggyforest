use console::{Key, Term};
use rand::Rng;
use std::char;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
pub struct KeyReader {
    jh: Option<tokio::task::JoinHandle<Key>>,
}
impl KeyReader {
    pub fn new() -> KeyReader {
        KeyReader {
            jh: Some(tokio::spawn(async { Self::await_key_press().await })),
        }
    }

    async fn await_key_press() -> Key {
        let term = Term::stdout();
        term.read_key().unwrap()
    }

    pub async fn read_key(&mut self) -> Option<Key> {
        if let Some(handle) = self.jh.take() {
            match handle.await {
                Ok(key) => {
                    self.jh = Some(tokio::spawn(async { Self::await_key_press().await }));
                    Some(key)
                }
                Err(_) => None,
            }
        } else {
            None
        }
    }
}

pub struct Row {
    objects: Vec<bool>,
    object_label: char,
    environment_label: char,
}

impl Row {
    pub fn new(objects: Vec<bool>, object_label: char, environment_label: char) -> Self {
        Self {
            objects,
            object_label,
            environment_label,
        }
    }
    pub fn new_random_row(object_label: char, environment_label: char) -> Self {
        let mut rng = rand::thread_rng();
        let mut objects = Vec::with_capacity(14);
        for _ in 0..14 {
            objects.push(rng.gen_bool(0.2));
        }
        Self {
            objects,
            object_label,
            environment_label,
        }
    }
}

pub struct GameState {
    gameboard: Vec<Row>,
    player: (usize, usize),
    keyreader: KeyReader,
    player_score: u32,
    debug_messages: Vec<String>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            gameboard: vec![
                Row::new_random_row('🌲', '🟩'),
                Row::new_random_row('🌲', '🟩'),
                Row::new_random_row('🌲', '🟩'),
                Row::new_random_row('🌲', '🟩'),
                Row::new_random_row('🌲', '🟩'),
                Row::new_random_row('🌲', '🟩'),
                Row::new_random_row('🌲', '🟩'),
            ],
            player: (7, 0),
            keyreader: KeyReader::new(),
            player_score: 0,
            debug_messages: Vec::new(),
        }
    }

    pub fn print_gameboard(&self) {
        let term = Term::stdout();
        term.clear_screen().unwrap();
        let player_row_index = self.player.1;

        for (row_index, row) in self.gameboard.iter().enumerate().rev() {
            for (col_index, &obj) in row.objects.iter().enumerate() {
                if row_index == player_row_index && col_index == self.player.0 {
                    print!("🐸");
                } else {
                    print!(
                        "{}",
                        if obj {
                            row.object_label
                        } else {
                            row.environment_label
                        }
                    );
                }
            }
            println!();
        }
        println!("Score: {}", self.player_score);
        for msg in &self.debug_messages {
            println!("{}", msg);
        }
    }

    pub async fn run(&mut self) {
        loop {
            self.print_gameboard();
            self.update_player().await;
            sleep(Duration::from_millis(50)).await;
        }
    }

    pub async fn update_player(&mut self) {
        if let Some(key) = self.keyreader.read_key().await {
            match key {
                Key::Char('w') | Key::ArrowUp => {
                    if self.player.1 < 3 {
                        let new_y = self.player.1 + 1;
                        let is_tree = self.gameboard[new_y].objects[self.player.0];
                        if !is_tree {
                            self.player.1 = new_y;
                            self.player_score += 1;
                        }
                    } else if self.player.1 == 3 {
                        let next_row_tree = self.gameboard[4].objects[self.player.0];
                        if !next_row_tree {
                            let new_row = Row::new_random_row('🌲', '🟩');
                            self.gameboard.remove(0);
                            self.gameboard.push(new_row);
                            self.player_score += 1;
                        }
                    }
                }
                Key::Char('a') | Key::ArrowLeft => {
                    if self.player.0 > 0 {
                        let new_x = self.player.0 - 1;
                        let is_tree = self.gameboard[self.player.1].objects[new_x];
                        if !is_tree {
                            self.player.0 = new_x;
                        }
                    }
                }
                Key::Char('s') | Key::ArrowDown => {
                    if self.player.1 > 0 {
                        let new_y = self.player.1 - 1;
                        let is_tree = self.gameboard[new_y].objects[self.player.0];
                        if !is_tree {
                            self.player.1 = new_y;
                        }
                    }
                }
                Key::Char('d') | Key::ArrowRight => {
                    if self.player.0 < 13 {
                        let new_x = self.player.0 + 1;
                        let is_tree = self.gameboard[self.player.1].objects[new_x];
                        if !is_tree {
                            self.player.0 = new_x;
                        }
                    }
                }
                Key::Escape => std::process::exit(0),
                _ => (),
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let mut game_state = GameState::new();
    game_state.run().await;
}