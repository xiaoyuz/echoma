use lazy_static::lazy_static;
use tokio::sync::Mutex;

use crate::USER_CHATTING_NAME;

lazy_static! {
    pub(crate) static ref CURRENT_SESSION: Mutex<Session> = Mutex::new(Session::new());
}

#[derive(Default)]
struct IOPair {
    input: String,
    output: String,
}

#[derive(Default)]
pub(crate) struct Session {
    pairs: Vec<IOPair>,
}

impl Session {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub(crate) fn append(&mut self, input: &str, output: &str) {
        let pair = IOPair {
            input: input.to_string(),
            output: output.to_string(),
        };
        self.pairs.push(pair);
    }

    #[allow(dead_code)]
    pub(crate) fn gen_prompt(&mut self, prompt: &str) -> String {
        return format!("Instruct: {}\nOutput:", prompt);
    }

    #[allow(dead_code)]
    pub(crate) fn gen_prompt_with_context(&mut self, prompt: &str) -> String {
        let prefix = format!(
            "{}: Hi.\nAI: Hi, what can I do for you?\n",
            USER_CHATTING_NAME
        );
        if self.pairs.is_empty() {
            return format!("{}{}: {}\nAI:", prefix, USER_CHATTING_NAME, prompt);
        }
        let recent = if self.pairs.len() < 1 {
            self.pairs.as_slice()
        } else {
            &self.pairs[self.pairs.len() - 1..]
        };
        let mut context = String::default();
        for pair in recent {
            let chat = format!(
                "{}: {}\nAI:{}\n",
                USER_CHATTING_NAME, pair.input, pair.output
            );
            context = format!("{}{}", context, chat);
        }
        return format!(
            "{}{}{}: {}\nAI:",
            prefix, context, USER_CHATTING_NAME, prompt
        );
    }

    pub(crate) fn clear(&mut self) {
        self.pairs.clear();
    }
}
