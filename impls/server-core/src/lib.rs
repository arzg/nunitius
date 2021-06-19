pub mod id;

use flume::Sender;
use id::{IdGenerator, SenderId};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Server {
    users: HashMap<SenderId, model::User>,
    id_generator: IdGenerator,
    past_events: Vec<model::Event>,
    event_tx: Sender<model::Event>,
}

impl Server {
    pub fn new(event_tx: Sender<model::Event>) -> Self {
        Self {
            users: HashMap::new(),
            id_generator: IdGenerator::default(),
            past_events: Vec::new(),
            event_tx,
        }
    }

    pub fn login(&mut self, user: model::User) -> LoginResponse {
        let is_nickname_taken = self
            .users
            .values()
            .map(|user| &user.nickname)
            .any(|nickname| *nickname == user.nickname);

        if is_nickname_taken {
            return LoginResponse::Taken;
        }

        let id = self.id_generator.next_sender_id();
        self.users.insert(id, user.clone());

        self.add_event(model::Event {
            data: model::EventData::Login,
            user,
        });

        LoginResponse::Succeeded(id)
    }

    pub fn logout(&mut self, id: SenderId) {
        self.add_event(model::Event {
            data: model::EventData::Logout,
            user: self.user_for(id).clone(),
        });

        self.users.remove(&id);
    }

    pub fn send_message(&mut self, id: SenderId, message: model::Message) {
        self.add_event(model::Event {
            data: model::EventData::Message(message),
            user: self.user_for(id).clone(),
        });
    }

    pub fn past_events(&self) -> &[model::Event] {
        &self.past_events
    }

    fn add_event(&mut self, event: model::Event) {
        self.event_tx.send(event.clone()).unwrap();
        self.past_events.push(event);
    }

    fn user_for(&self, id: SenderId) -> &model::User {
        self.users.get(&id).unwrap()
    }
}

#[derive(Debug, PartialEq)]
pub enum LoginResponse {
    Succeeded(SenderId),
    Taken,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login() {
        let alice = model::User {
            nickname: "alice".to_string(),
        };

        let (event_tx, event_rx) = flume::unbounded();
        let mut server = Server::new(event_tx);

        let response = server.login(alice.clone());
        assert!(matches!(response, LoginResponse::Succeeded(_)));

        assert_eq!(
            event_rx.drain().collect::<Vec<_>>(),
            [model::Event {
                data: model::EventData::Login,
                user: alice
            }]
        );
    }

    #[test]
    fn login_taken() {
        let alice = model::User {
            nickname: "alice".to_string(),
        };
        let bob = model::User {
            nickname: "bob".to_string(),
        };

        let (event_tx, event_rx) = flume::unbounded();
        let mut server = Server::new(event_tx);

        let response = server.login(alice.clone());
        assert!(matches!(response, LoginResponse::Succeeded(_)));

        let response = server.login(alice.clone());
        assert_eq!(response, LoginResponse::Taken);

        let response = server.login(bob.clone());
        assert!(matches!(response, LoginResponse::Succeeded(_)));

        assert_eq!(
            event_rx.drain().collect::<Vec<_>>(),
            [
                model::Event {
                    data: model::EventData::Login,
                    user: alice
                },
                model::Event {
                    data: model::EventData::Login,
                    user: bob
                }
            ]
        );
    }

    #[test]
    fn logout() {
        let alice = model::User {
            nickname: "alice".to_string(),
        };

        let (event_tx, event_rx) = flume::unbounded();
        let mut server = Server::new(event_tx);

        let response = server.login(alice.clone());
        let id = match response {
            LoginResponse::Succeeded(id) => id,
            LoginResponse::Taken => unreachable!(),
        };

        server.logout(id);

        let response = server.login(alice.clone());
        assert!(matches!(response, LoginResponse::Succeeded(_)));

        assert_eq!(
            event_rx.drain().collect::<Vec<_>>(),
            [
                model::Event {
                    data: model::EventData::Login,
                    user: alice.clone()
                },
                model::Event {
                    data: model::EventData::Logout,
                    user: alice.clone()
                },
                model::Event {
                    data: model::EventData::Login,
                    user: alice
                }
            ]
        );
    }

    #[test]
    fn send_message() {
        let alice = model::User {
            nickname: "alice".to_string(),
        };
        let message = model::Message {
            body: "Hello!".to_string(),
        };

        let (event_tx, event_rx) = flume::unbounded();
        let mut server = Server::new(event_tx);

        let response = server.login(alice.clone());
        let id = match response {
            LoginResponse::Succeeded(id) => id,
            LoginResponse::Taken => unreachable!(),
        };

        server.send_message(id, message.clone());

        assert_eq!(
            event_rx.drain().collect::<Vec<_>>(),
            [
                model::Event {
                    data: model::EventData::Login,
                    user: alice.clone()
                },
                model::Event {
                    data: model::EventData::Message(message),
                    user: alice
                }
            ]
        );
    }

    #[test]
    fn past_events() {
        let user = model::User {
            nickname: "rustc".to_string(),
        };
        let message = model::Message {
            body: "cannot borrow `foo` as mutable more than once at a time".to_string(),
        };

        let (event_tx, event_rx) = flume::unbounded();
        let mut server = Server::new(event_tx);

        let response = server.login(user.clone());
        let id = match response {
            LoginResponse::Succeeded(id) => id,
            LoginResponse::Taken => unreachable!(),
        };

        server.send_message(id, message.clone());
        server.logout(id);

        assert_eq!(
            server.past_events(),
            [
                model::Event {
                    data: model::EventData::Login,
                    user: user.clone()
                },
                model::Event {
                    data: model::EventData::Message(message),
                    user: user.clone()
                },
                model::Event {
                    data: model::EventData::Logout,
                    user
                },
            ]
        );
        assert_eq!(event_rx.drain().collect::<Vec<_>>(), server.past_events());
    }
}
