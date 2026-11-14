use std::collections::HashSet;

use jsonwebtoken::errors::Result as JWTResult;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};

use crate::modal::Claims;

pub fn extract_claim(token: &str) -> JWTResult<Claims> {
    let secret = b"qwertyuiopasdfghjklzxcvbnm123456"; // Replace with your secret key

    // Decoding the token with the secret key
    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 0;
    validation.aud = Some(HashSet::from(["www.lekhabook.com".to_string()]));
    validation.validate_exp = true;
    // Decode the token
    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)?;

    // Return the claims
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
