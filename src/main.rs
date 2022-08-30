use serde::{Deserialize, Serialize};
use zenoh::prelude::r#async::*;
use std::fmt::Debug;
use futures::future;
use futures::stream::{Stream, StreamExt};

#[derive(Deserialize, Serialize, PartialEq, Debug)]
struct StdString {
    data: String,  // the field name can be anything...
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
struct StdUInt16 {
    data: u16,
}

// Compare to original ROS IDL file.
#[derive(Deserialize, Serialize, PartialEq, Debug)]
struct GeometryPose {
    xyz: [f64; 3],
    quat: [f64; 4],
}

#[tokio::main]
async fn main() {
    // zenoh setup
    let mut config = zenoh::config::Config::default();
    // -m client
    config.set_mode(Some("client".parse().unwrap())).unwrap();
    // -e tcp/...
    config.connect.endpoints.push("tcp/0.0.0.0:7447".parse().unwrap());
    // --no-multicast-scouting
    config.scouting.multicast.set_enabled(Some(false)).unwrap();

    println!("Opening zenoh session...");
    let session = zenoh::open(config).res().await.unwrap();

    // ros topic chatter. (talker example)
    let topic = "rt/chatter";
    println!("Creating zenoh subscriber on '{}'...", topic);
    let sub = RosSubscriber::<StdString>::subscribe(&session, topic).await;
    tokio::spawn(print_data(topic, sub.make_stream()));

    let topic = "rt/number";
    println!("Creating zenoh subscriber on '{}'...", topic);
    let sub = RosSubscriber::<StdUInt16>::subscribe(&session, topic).await;
    tokio::spawn(print_data(topic, sub.make_stream()));

    let topic = "rt/pose";
    println!("Creating zenoh subscriber on '{}'...", topic);
    let sub = RosSubscriber::<GeometryPose>::subscribe(&session, topic).await;
    tokio::spawn(print_data(topic, sub.make_stream()));

    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }
}

struct RosSubscriber<'b, T> {
    topic: String,
    zenoh_sub: zenoh::subscriber::Subscriber<'b, flume::Receiver<Sample>>,
    phantom: std::marker::PhantomData<T>,
}

impl<'a, T> RosSubscriber<'a, T> where T: Deserialize<'a> {
    async fn subscribe<'b>(session: &'b zenoh::Session, topic: &str) -> RosSubscriber<'b, T>
    {
        let subscriber = session.declare_subscriber(topic).res().await.unwrap();

        RosSubscriber {
            topic: topic.to_owned(),
            zenoh_sub: subscriber,
            phantom: std::marker::PhantomData::<T>,
        }
    }

    pub fn make_stream(&self) -> impl Stream<Item = T> + Unpin {
        let subscriber_task = self.zenoh_sub.to_owned().into_stream();

        let topic = self.topic.to_owned();
        subscriber_task.filter_map(move |s| {
            let cow = std::borrow::Cow::<[u8]>::try_from(&s.value).unwrap();
            if let Ok(data) = cdr::deserialize::<T>(&cow) {
                future::ready(Some(data))
            } else {
                println!("Warning: cannot deserialize on topic {}...", topic);
                future::ready(None)
            }
        })
    }
}

async fn print_data<T: Debug>(topic: &str, mut sub: impl Stream<Item = T> + Unpin) {
    loop {
        if let Some(data) = sub.next().await {
            println!("[{}] Received {:?}", topic, data);
        }
    }
}
