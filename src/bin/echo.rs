use anyhow::{Context, Result};
use flyio_dis_sys::{Body, Message, Payload};
use serde_json::Deserializer;
use std::io::{stdin, stdout, StdoutLock, Write};

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
            Payload::Generate => {}
            Payload::GenerateOk { .. } => {}
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
