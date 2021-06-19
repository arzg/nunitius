#[derive(Debug, Default)]
pub(crate) struct IdGenerator {
    current_sender_id: u32,
    current_viewer_id: u32,
}

impl IdGenerator {
    pub(crate) fn next_sender_id(&mut self) -> SenderId {
        let next_id = SenderId(self.current_sender_id);
        self.current_sender_id += 1;

        next_id
    }

    pub(crate) fn next_viewer_id(&mut self) -> ViewerId {
        let next_id = ViewerId(self.current_viewer_id);
        self.current_viewer_id += 1;

        next_id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SenderId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ViewerId(u32);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut id_generator = IdGenerator::default();

        assert_eq!(id_generator.next_sender_id(), SenderId(0));
        assert_eq!(id_generator.next_sender_id(), SenderId(1));

        assert_eq!(id_generator.next_viewer_id(), ViewerId(0));
        assert_eq!(id_generator.next_viewer_id(), ViewerId(1));

        assert_eq!(id_generator.next_viewer_id(), ViewerId(2));
        assert_eq!(id_generator.next_sender_id(), SenderId(2));
        assert_eq!(id_generator.next_viewer_id(), ViewerId(3));
        assert_eq!(id_generator.next_sender_id(), SenderId(3));
        assert_eq!(id_generator.next_viewer_id(), ViewerId(4));
    }
}
