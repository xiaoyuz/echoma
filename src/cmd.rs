use tokio::sync::mpsc;

use crate::{
    llama::{options::PredictOptions, LOCAL_LLAMA},
    session::CURRENT_SESSION,
    Result, USER_CHATTING_NAME, USER_CHATTING_NAME_SHORT,
};

pub enum CmdRes {
    Content(String),
    Over,
    Exit,
}

pub enum Cmd {
    Greeting,
    Exit,
    Message(String),
}

impl From<&str> for Cmd {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "hi echo" => Cmd::Greeting,
            "exit" => Cmd::Exit,
            _ => Cmd::Message(value.to_string()),
        }
    }
}

pub struct Executor {
    pub cmd: Cmd,
    pub result_sender: mpsc::Sender<CmdRes>,
}

impl Executor {
    pub fn new(user_input: &str, sender: mpsc::Sender<CmdRes>) -> Result<Self> {
        Ok(Self {
            cmd: user_input.into(),
            result_sender: sender,
        })
    }

    pub async fn apply(&self) -> Result<()> {
        match &self.cmd {
            Cmd::Greeting => {
                CURRENT_SESSION.lock().await.clear();
                self.result_sender
                    .send(CmdRes::Content("Hello, what can I do for you?".to_string()))
                    .await?;
                self.result_sender.send(CmdRes::Over).await
            }
            Cmd::Exit => self.result_sender.send(CmdRes::Exit).await,
            Cmd::Message(message) => {
                let prompt = CURRENT_SESSION.lock().await.gen_prompt(&message);

                let sender = self.result_sender.clone();
                let predict_options = PredictOptions {
                    token_callback: Some(Box::new(move |token| {
                        if !token.contains(USER_CHATTING_NAME_SHORT) {
                            let sender = sender.clone();
                            tokio::spawn(async move { sender.send(CmdRes::Content(token)).await });
                            true
                        } else {
                            false
                        }
                    })),
                    stop_prompts: vec![USER_CHATTING_NAME.to_string()],
                    ..Default::default()
                };
                LOCAL_LLAMA.get().await.predict(prompt, predict_options)?;
                self.result_sender.send(CmdRes::Over).await
            }
        }?;
        Ok(())
    }
}
