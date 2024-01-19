use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    sync::mpsc,
};

use crate::{
    cmd::{CmdRes, Executor},
    session::CURRENT_SESSION,
    Result, USER_CHATTING_NAME_SHORT,
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

                let mut output: Vec<String> = Default::default();
                while let Some(cmd_res) = rx.recv().await {
                    match cmd_res {
                        CmdRes::Content(content) => {
                            output.push(content.clone());
                        }
                        CmdRes::Over => {
                            let output_str = output.join("").replace(USER_CHATTING_NAME_SHORT, "");
                            let output_str = output_str.trim();
                            CURRENT_SESSION
                                .lock()
                                .await
                                .append(user_input.as_str(), output_str);
                            output.clear();
                            println!("{}", output_str);
                            break;
                        }
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
