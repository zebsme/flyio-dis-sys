use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::io::{stdin, stdout, StdoutLock, Write};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Message {
    src: String,
    #[serde(rename = "dest")]
    dst: String,
    body: Body,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Body {
    #[serde(rename = "msg_id")]
    id: Option<usize>,
    in_reply_to: Option<usize>,
    #[serde(flatten)]
    payload: Payload,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
}

struct EchoNode {
    id: usize,
}

impl EchoNode {
    fn step(&mut self, input: Message, output: &mut StdoutLock<'static>) -> Result<()> {
        match input.body.payload {
            Payload::Echo { echo } => {
                let reply = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: Payload::EchoOk { echo },
                    },
                };
                // reply.serialize(output).context("Failed to serialize output")?;
                serde_json::to_writer(&mut *output, &reply)?;
                output.write_all(b"\n").context("Failed to write output")?;
                self.id += 1;
            }
            Payload::Init { .. } => {
                let reply = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: Payload::InitOk,
                    },
                };
                serde_json::to_writer(&mut *output, &reply)?;
                output.write_all(b"\n").context("Failed to write output")?;
                self.id += 1;
            }
            Payload::InitOk {} => {}
            Payload::EchoOk { .. } => {}
        };
        Ok(())
    }
}

fn main() -> Result<()> {
    let stdin_handle = stdin().lock();
    let mut output = stdout().lock();
    let inputs = Deserializer::from_reader(stdin_handle).into_iter::<Message>();
    let mut node = EchoNode { id: 1 };
    for input in inputs {
        let input = input.context("Failed to parse input")?;
        node.step(input, &mut output)
            .context("Failed to process input")?;
    }
    Ok(())
}
