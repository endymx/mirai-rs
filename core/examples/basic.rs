use mirai_rs::session::MiraiServer;
use mirai_rs::message::{MessagePackage, SingleMessage, Message, Permission};

use std::io::stdin;
use std::sync::{mpsc, Arc};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let server = MiraiServer::new("http://localhost:8080");
    let mut auth_key = String::new();
    let mut id = String::new();

    while let Err(_) = server.about().await {
        println!("Cannot connect to Server, try to reconnect...");
        std::thread::sleep(Duration::from_secs(1));
    }

    println!("Please input auth key: ");
    stdin().read_line(&mut auth_key).expect("input error");
    let session = server.auth(auth_key.trim()).await.unwrap();

    println!("Please input qq id: ");
    stdin().read_line(&mut id).expect("input error");
    session.verify(id.trim().parse().expect("wrong qq id format")).await.unwrap();

    println!("Done: {:?}", session);

    let (sc, rc) = mpsc::channel();
    let session = Arc::new(session);

    {
        let session = session.clone();
        let _job = tokio::spawn(async move {
            loop {
                let mps = session.fetch_newest_message(1).await;

                match mps {
                    Ok(mps) => {
                        let first = mps.get(0);
                        if let Some(mp) = first {
                            sc.send(mp.clone()).unwrap();
                        }
                    }

                    Err(e) => println!("{:?}", e)
                }
            }
        });
    }

    println!("{:?}", session.friend_list().await);
    println!("{:?}", session.group_list().await);

    for mp in rc.iter() {
        if let MessagePackage::GroupMessage {
            message_chain,
            sender
        } = &mp {
            let msg = message_chain.iter().fold(String::new(), |msg, elem| {
                if let SingleMessage::Plain { text } = elem {
                    msg + text
                } else {
                    msg
                }
            });

            match msg.trim() {
                "Hello" => {
                    session.send_group_message(&Message::new(
                        sender.group.id,
                        &vec![SingleMessage::Image { image_id: None, url: None, path: Some("nya.png".to_string()) }])
                    ).await.unwrap();
                }

                "mute me" => {
                    if let Permission::Administrator | Permission::Owner = sender.group.permission {
                        if let Permission::Administrator | Permission::Owner = sender.permission {
                            session.send_group_message(
                                &Message::new(sender.group.id, &vec!["You are too powerful to mute.".into()])
                            ).await.unwrap();
                        } else {
                            session.mute(sender.group.id, sender.id, 60 * 10).await.unwrap();

                            {
                                let session = session.clone();
                                let sender = sender.clone();
                                tokio::spawn(async move {
                                    std::thread::sleep(Duration::from_secs(10));
                                    session.unmute(sender.group.id, sender.id).await.unwrap();
                                });
                            }
                        }
                    } else {
                        session.send_group_message(
                            &Message::new(sender.group.id, &vec!["I have not enough permission QAQ.".into()])
                        ).await.unwrap();
                    }
                }
                _ => {}
            };
        }

        println!("{:?}", mp);
    }
}