pub struct Metadata {
    pub protocol_name: String,
    pub contract: Vec<u8>,
}

impl Metadata {
    pub fn to_markdown(&self) -> String {
        format!(
            "** {} **\nContract template:\n {}",
            self.protocol_name,
            hex::encode(&self.contract)
        )
    }
}
