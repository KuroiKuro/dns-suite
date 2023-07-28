pub mod types;
pub mod domain;

// Placeholders
struct Header;
struct Question;
struct Answer;
struct Authority;
struct Additional;

pub struct DnsQuery {
    header: Header,
    question: Question,
    answer: Answer,
    authority: Authority,
    additional: Additional,
}

pub struct DnsAnswer {

}
