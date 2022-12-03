use deadpool_lapin::{Manager, Pool, PoolError};
pub use lapin::message::Delivery;
use lapin::{options::*, publisher_confirm::PublisherConfirm, types::FieldTable, *};
use std::error::Error;
use tokio_amqp::*;

use futures::{StreamExt};
use std::{
    sync::{Arc},
};

use amq_protocol_types::ShortString;


type Connection = deadpool::managed::Object<deadpool_lapin::Manager>;

pub trait MessageConsumer {
    fn consume(&self, delivery: &Delivery) -> Option<Vec<u8>>;
}

/// Generic type for instances that do not consume
impl MessageConsumer for () {
    fn consume(&self, _delivery: &Delivery) -> Option<Vec<u8>> {
        None
    }
}

#[derive(Debug)]
pub struct Rabbit {
    pool: Pool,
}

impl Rabbit {
    pub async fn new(url: String) -> Self {
        let options = ConnectionProperties::default().with_tokio();
        let manager = Manager::new(url, options);

        let pool: Pool = deadpool::managed::Pool::builder(manager)
            .max_size(10)
            .build()
            .unwrap();

        Self { pool }
    }

    pub async fn publish(
        &self,
        queue_name: &str,
        payload: &[u8],
    ) -> std::result::Result<PublisherConfirm, Box<dyn Error>> {
        let connection = get_rmq_con(self.pool.clone()).await?;

        let channel = connection.create_channel().await?;

        let _queue = channel
            .queue_declare(
                queue_name,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;

        let pub_confirm = channel
            .basic_publish(
                "",
                queue_name,
                BasicPublishOptions::default(),
                payload,
                BasicProperties::default(),
            )
            .await?;

        Ok(pub_confirm)
    }

    pub async fn publish_and_await_reply(
        &self,
        publish_queue: &str,
        consumer_name: &str,
        payload: &[u8],
    ) -> std::result::Result<Vec<u8>, Box<dyn Error>> {
        let connection = get_rmq_con(self.pool.clone()).await?;

        let channel = connection.create_channel().await?;

        let reply_queue = "amq.rabbitmq.reply-to";
        let exchange = "";

        let consume_properties = BasicConsumeOptions {
            no_local: false,
            no_ack: true, // Important. Reply consumer cannot ACK
            exclusive: false,
            nowait: false,
        };
        let mut consumer = channel
            .basic_consume(
                reply_queue,
                consumer_name,
                consume_properties,
                FieldTable::default(),
            )
            .await?;

        let basic_properties =
            BasicProperties::default().with_reply_to(ShortString::from(reply_queue));
        let _pub_confirm = channel
            .basic_publish(
                exchange,
                publish_queue,
                BasicPublishOptions::default(),
                payload,
                basic_properties,
            )
            .await?;

        loop {
            if let Some(delivery) = consumer.next().await {
                if let Ok(delivery) = delivery {
                    return Ok(delivery.data);
                }
            }
        }

        //Ok(pub_confirm)
    }

    pub async fn consume_messages<F: MessageConsumer>(
        &self,
        queue_name: &str,
        consumer_name: &str,
        consumer: F,
    ) -> std::result::Result<(), Box<dyn Error>> { 
        let connection = get_rmq_con(self.pool.clone()).await?;

        let channel = connection.create_channel().await?;

        let _queue = channel
            .queue_declare(
                queue_name,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;

        let mut message_consumer = channel
            .basic_consume(
                queue_name,
                consumer_name,
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;
        
        while let Some(delivery) = message_consumer.next().await {
            if let Ok(delivery) = delivery {
                // Heavy lifting
                let answer = consumer.consume(&delivery);
                
                let properties = delivery.properties;
                
                if let Some(reply_queue) = properties.reply_to() {
                    if let Some(answer) = answer {
                        // Publish answer to `reply_to` queue
                        let exchange = "";
                        let _pub_confirm = channel
                        .basic_publish(
                            exchange,
                            reply_queue.as_str(),
                            BasicPublishOptions::default(),
                            &answer,
                            BasicProperties::default(),
                        )
                        .await?;
                    }
                }
                
                // Ack the message
                channel
                    .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                    .await?;
            }
        }

        Ok(())
    }
}

async fn get_rmq_con(pool: Pool) -> std::result::Result<Connection, PoolError> {
    let connection = pool.get().await?;

    Ok(connection)
}
