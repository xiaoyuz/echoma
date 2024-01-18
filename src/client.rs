use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    sync::mpsc,
};

use crate::{
    cmd::{CmdRes, Executor},
    Result,
};

pub struct Client {}

impl Client {
    pub async fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub async fn start(self) -> Result<()> {
        let stdin = io::stdin();
        let mut lines = BufReader::new(stdin).lines();

        'outer: loop {
            println!("User:");

            if let Some(line) = lines.next_line().await.expect("Can't read the line") {
                let user_input = line.trim().to_string();

                let (tx, mut rx) = mpsc::channel(5);
                let executor = Executor::new(user_input.as_str(), tx)?;

                tokio::spawn(async move {
                    let _ = executor.apply().await;
                });

                println!("Echo:");
                while let Some(cmd_res) = rx.recv().await {
                    match cmd_res {
                        CmdRes::Content(content) => print!("{}", content),
                        CmdRes::Over => break,
                        CmdRes::Exit => {
                            println!("Bye Bye!");
                            break 'outer;
                        }
                    }
                }
                println!();
            }
        }

        Ok(())
    }
}
