use color_eyre::eyre::Result;
use conf::ClientConfig;
use irc::client::{prelude::*, ClientStream};

pub mod conf;

#[derive(Default)]
pub struct ClientBuffer {}

pub struct ConnectedClient {
        config: ClientConfig,
        client: Client,
        sender: Sender,
        stream: ClientStream,
        buf: ClientBuffer,
}

impl ConnectedClient {
    pub async fn disconnect(self) -> Result<DisconnectedClient> {
        self.client.send_quit(self.config.default_quit.unwrap_or("eesh.rsrvc.org".to_owned()))?;

        todo!()
    }
}

pub struct DisconnectedClient {
        config: ClientConfig,
        buf: ClientBuffer,
}

impl DisconnectedClient {
    pub fn new(config: ClientConfig) -> DisconnectedClient {
        DisconnectedClient { config, buf: ClientBuffer::default() }
    }

    pub async fn connect(self) -> Result<ConnectedClient> {
        let mut client = Client::from_config(self.config.irc.clone()).await?;
        let sender = client.sender();
        let stream = client.stream()?;

        Ok(ConnectedClient { config: self.config, client, sender, stream, buf: self.buf })
    }
}

/*
impl ClientBridge {
    async fn connect() -> color_eyre::Result<ClientBridge> {
        let config = Config {
            nickname: Some("pickles".to_owned()),
            server: Some("chat.freenode.net".to_owned()),
            channels: vec!["#rust-spam".to_owned()],
            ..Default::default()
        };

        let mut client = Client::from_config(config).await?;
        client.identify()?;

        let mut stream = client.stream()?;
        let sender = client.sender();

        while let Some(message) = stream.next().await.transpose()? {
            print!("{}", message);

            if let Command::PRIVMSG(ref target, ref msg) = message.command {
                if msg.contains(client.current_nickname()) {
                    sender.send_privmsg(target, "Hi!")?;
                }
            }
        }
        Ok(todo!())
    }
}
*/
