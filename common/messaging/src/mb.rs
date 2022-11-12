use deadpool_lapin::{Manager, Pool, PoolError};
pub use lapin::message::Delivery;
use lapin::{options::*, publisher_confirm::PublisherConfirm, types::FieldTable, *};
use std::error::Error;
use tokio_amqp::*;

use futures::{join, StreamExt};
use std::time::Duration;

use amq_protocol_types::ShortString;
use uuid::Uuid;

type Connection = deadpool::managed::Object<deadpool_lapin::Manager>;

pub trait MessageConsumer {
    fn consume(&mut self, delivery: &Delivery) -> Option<Vec<u8>>;
}

/// Generic type for instances that do not consume
impl MessageConsumer for () {
    fn consume(&mut self, delivery: &Delivery) -> Option<Vec<u8>> {
        None
    }
}

#[derive(Debug)]
pub struct Rabbit<F: MessageConsumer> {
    pool: Pool,
    consumer: F,
}

impl<F: MessageConsumer> Rabbit<F> {
    pub async fn new(url: String, consumer: F) -> Self {
        let options = ConnectionProperties::default().with_tokio();
        let manager = Manager::new(url, options);

        let pool: Pool = deadpool::managed::Pool::builder(manager)
            .max_size(10)
        .build()
        .unwrap();

        Self { pool, consumer }
    }

    pub async fn publish(
        &self,
        queue_name: &str,
        payload: &[u8],
    ) -> std::result::Result<PublisherConfirm, Box<dyn Error>> {
        let connection = get_rmq_con(self.pool.clone()).await?;

        let channel = connection.create_channel().await?;

        let queue = channel
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

        /*let queue = channel
            .queue_declare(
                queue_name,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;*/

        let reply_queue = "amq.rabbitmq.reply-to";
        let exchange = "";

        let consume_properties = BasicConsumeOptions {
            no_local: false,
            no_ack: true, // Important. Reply consumer cannot ACK
            exclusive: false,
            nowait: false
        };
        let mut consumer = channel
            .basic_consume(
                reply_queue,
                consumer_name,
                consume_properties,
                FieldTable::default(),
            )
            .await?;

        let basic_properties = BasicProperties::default()
            .with_reply_to(ShortString::from(reply_queue));
        let pub_confirm = channel
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
                    //let properties = &delivery.properties;
                    //if let Some(corr_id) = properties.correlation_id() {
                        //if corr_id = uui {
                            return Ok(delivery.data);
                        //}
                    //}
                }
            }
        }

        //Ok(pub_confirm)
    }

    pub async fn consume_messages(
        &mut self,
        queue_name: &str,
        consumer_name: &str,
    ) -> std::result::Result<(), Box<dyn Error>>
    {
        let connection = get_rmq_con(self.pool.clone()).await?;

        let channel = connection.create_channel().await?;

        let queue = channel
            .queue_declare(
                queue_name,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;

        let mut consumer = channel
            .basic_consume(
                queue_name,
                consumer_name,
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        while let Some(delivery) = consumer.next().await {
            if let Ok(delivery) = delivery {
                let answer = self.consumer.consume(&delivery);
                let properties = delivery.properties;

                if let Some(reply_queue) = properties.reply_to() {
                    if let Some(answer) = answer {
                        // Publish answer to `reply_to` queue
                        let exchange = "";
                        let pub_confirm = channel
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
