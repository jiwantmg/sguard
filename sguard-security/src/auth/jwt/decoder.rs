use std::collections::HashSet;
use std::sync::LazyLock;

use jsonwebtoken::errors::Result as JWTResult;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};

use crate::modal::Claims;

static JWT_SECRET: &[u8] = b"qwertyuiopasdfghjklzxcvbnm123456";

static DECODING_KEY: LazyLock<DecodingKey> =
    LazyLock::new(|| DecodingKey::from_secret(JWT_SECRET));

static VALIDATION: LazyLock<Validation> = LazyLock::new(|| {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 0;
    validation.aud = Some(HashSet::from(["www.lekhabook.com".to_string()]));
    validation.validate_exp = true;
    validation
});

pub fn extract_claim(token: &str) -> JWTResult<Claims> {
    let token_data = decode::<Claims>(token, &DECODING_KEY, &VALIDATION)?;
    Ok(token_data.claims)
}

pub fn get_issuer_from_token(token: &str) -> Result<Option<String>, String> {
    // Decode the header of the JWT
    let header = decode_header(token).map_err(|e| format!("Failed to decode header: {}", e))?;
    println!("JWT Header: {:?}", header);

    // Skip signature verification and decode only the payload (unsafe operation)
    // WARNING: Only do this if you trust the token source; otherwise, validate the token first.
    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = false;
    validation.algorithms = vec![Algorithm::RS256, Algorithm::HS256];

    let fake_decoding_key = DecodingKey::from_secret("dummy_key".as_ref()); // Dummy key since we're skipping verification

    let decoded = jsonwebtoken::decode::<Claims>(token, &fake_decoding_key, &validation)
        .map_err(|e| format!("Failed to decode token: {}", e))?;

    // Extract the issuer
    Ok(decoded.claims.iss)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};

    #[test]
    fn test_extract_claim_with_cached_key() {
        let claims = Claims {
            sub: Some("user123".to_string()),
            exp: Some(
                (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    + 3600) as usize,
            ),
            iss: Some("test-issuer".to_string()),
            aud: Some("www.lekhabook.com".to_string()),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(JWT_SECRET),
        )
        .unwrap();

        let extracted = extract_claim(&token).expect("token must decode successfully");
        assert_eq!(extracted.sub, Some("user123".to_string()));
        assert_eq!(extracted.aud, Some("www.lekhabook.com".to_string()));
    }
}
