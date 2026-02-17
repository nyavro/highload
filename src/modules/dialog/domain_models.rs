use uuid::Uuid;

pub struct DialogMessage {
    pub from: Uuid,
    pub to: Uuid,
    pub text: String
}