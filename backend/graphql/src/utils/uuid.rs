use async_graphql::ID;
use std::convert::TryFrom;
use uuid::Uuid;

pub fn generate_uuid_v7_binary() -> Vec<u8> {
    Uuid::now_v7().as_bytes().to_vec()
}

#[derive(Debug)]
pub struct UuidVec(pub Vec<u8>);

impl TryFrom<ID> for UuidVec {
    type Error = uuid::Error;

    fn try_from(id: ID) -> Result<Self, Self::Error> {
        let uuid = Uuid::parse_str(&id.to_string())?;
        Ok(UuidVec(uuid.as_bytes().to_vec()))
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_uuid_v7_binary() {
        let uuid_bytes = generate_uuid_v7_binary();
        assert_eq!(uuid_bytes.len(), 16);

        // 生成されたバイト列が有効なUUIDであることを確認
        let uuid = Uuid::from_bytes(uuid_bytes.try_into().unwrap());
        assert_eq!(uuid.get_version().unwrap(), uuid::Version::SortRand);
    }

    #[test]
    fn test_parse_valid_uuid_string() {
        let test_uuid = ID("018e1234-5678-90ab-cdef-1234567890ab".to_string());
        let result = UuidVec::try_from(test_uuid.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.len(), 16);
    }

    #[test]
    fn test_parse_invalid_uuid_string() {
        let test_uuid = ID("invalid-uuid".to_string());
        let result = UuidVec::try_from(test_uuid);
        assert!(result.is_err());
    }
}
