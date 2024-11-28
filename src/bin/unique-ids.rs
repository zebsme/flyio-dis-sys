use anyhow::{Context, Result};
use flyio_dis_sys::{Body, Message, Payload};
use serde_json::Deserializer;
use std::io::{stdin, stdout, StdoutLock, Write};
pub struct UniqueIdsNode {
    pub node_id: String,
    pub msg_id: usize,
    pub id: usize,
}

impl UniqueIdsNode {
    fn step(&mut self, input: Message, output: &mut StdoutLock<'static>) -> Result<()> {
        match input.body.payload {
            Payload::Init { node_id, .. } => {
                self.node_id = node_id;
                let reply = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body {
                        id: Some(self.msg_id),
                        in_reply_to: input.body.id,
                        payload: Payload::InitOk,
                    },
                };
                serde_json::to_writer(&mut *output, &reply)?;
                output.write_all(b"\n").context("Failed to write output")?;
                self.msg_id += 1;
            }
            Payload::InitOk {} => {}
            Payload::Generate => {
                let reply = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body {
                        id: Some(self.msg_id),
                        in_reply_to: input.body.id,
                        payload: Payload::GenerateOk {
                            id: format!("{}-{}", self.node_id, self.id),
                        },
                    },
                };
                serde_json::to_writer(&mut *output, &reply)?;
                output.write_all(b"\n").context("Failed to write output")?;
                self.msg_id += 1;
                self.id += 1;
            }
            Payload::GenerateOk { .. } => {}
            Payload::Echo { .. } => {}
            Payload::EchoOk { .. } => {}
        };
        Ok(())
    }
}

fn main() -> Result<()> {
    let stdin_handle = stdin().lock();
    let mut output = stdout().lock();
    let inputs = Deserializer::from_reader(stdin_handle).into_iter::<Message>();
    let mut node = UniqueIdsNode {
        msg_id: 1,
        node_id: "".to_string(),
        id: 1,
    };
    for input in inputs {
        let input = input.context("Failed to parse input")?;
        node.step(input, &mut output)
            .context("Failed to process input")?;
    }
    Ok(())
}
