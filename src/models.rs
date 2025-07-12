use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Question {
    pub title:       String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct QuestionDetail {
    pub question_uuid: String,
    pub title:         String,
    pub description:   String,
    pub created_at:    String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionId {
    pub question_uuid: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Answer {
    pub question_uuid: String,
    pub content:       String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnswerDetail {
    pub question_uuid: String,
    pub content:       String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnswerId {
    pub answer_uuid: String,
}
