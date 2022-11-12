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
        queue_name: &str,
        consumer_name: &str,
        payload: &[u8],
        reply_consumer: fn(&Delivery),
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

        let corr_id = Uuid::new_v4().to_string();
        let corr_id = ShortString::from(corr_id);
        let properties = BasicProperties::default().with_correlation_id(corr_id);
        let pub_confirm = channel
            .basic_publish(
                "",
                queue_name,
                BasicPublishOptions::default(),
                payload,
                properties,
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
                let properties = &delivery.properties;
                if let Some(corr_id) = properties.correlation_id() {
                    reply_consumer(&delivery);

                    // Ack if the message is consumed
                    channel
                        .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                        .await?;

                    break;
                }
            }
        }

        Ok(pub_confirm)
    }

    pub async fn consume_messages(
        &mut self,
        queue_name: &str,
        consumer_name: &str,
        //mut message_consumer: F,
    ) -> std::result::Result<(), Box<dyn Error>>
    //where
    //    F: FnMut(&Delivery) -> Option<Vec<u8>>,
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
                //let answer = message_consumer(&delivery);
                let answer = self.consumer.consume(&delivery);
                // Ack the message
                channel
                    .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                    .await?;

                // Reply with UUID and correct queue if present
                // Respond with answer
            }
        }

        Ok(())
    }
}

async fn get_rmq_con(pool: Pool) -> std::result::Result<Connection, PoolError> {
    let connection = pool.get().await?;

    Ok(connection)
}
