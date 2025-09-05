use crate::event::Event;
use crate::error::{ValidationError, NostrError};
use crate::constants::*;

/// Trait for validating Nostr objects
pub trait Validate {
    type Error;
    
    fn validate(&self) -> Result<(), Self::Error>;
}

impl Validate for Event {
    type Error = NostrError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        // Validate event size
        let json_size = serde_json::to_string(self)
            .map_err(|e| NostrError::InvalidJson(e))?
            .len();
        
        if json_size > MAX_EVENT_SIZE {
            return Err(ValidationError::EventTooLarge {
                size: json_size,
                max: MAX_EVENT_SIZE,
            }.into());
        }
        
        // Validate number of tags
        if self.tags.len() > MAX_TAGS_COUNT {
            return Err(ValidationError::TooManyTags {
                count: self.tags.len(),
                max: MAX_TAGS_COUNT,
            }.into());
        }
        
        // Validate tag values
        for tag in &self.tags {
            for value in tag.values() {
                if value.len() > MAX_TAG_VALUE_LENGTH {
                    return Err(ValidationError::TagValueTooLong {
                        length: value.len(),
                        max: MAX_TAG_VALUE_LENGTH,
                    }.into());
                }
            }
        }
        
        // Validate content length
        if self.content.len() > MAX_CONTENT_LENGTH {
            return Err(ValidationError::ContentTooLong {
                length: self.content.len(),
                max: MAX_CONTENT_LENGTH,
            }.into());
        }
        
        // Validate timestamp (not too far in the future or past)
        let now = chrono::Utc::now().timestamp();
        let max_future = now + 600; // 10 minutes in the future
        let min_past = 946684800; // Year 2000
        
        if self.created_at > max_future {
            return Err(ValidationError::InvalidFieldFormat(
                "Timestamp too far in the future".to_string()
            ).into());
        }
        
        if self.created_at < min_past {
            return Err(ValidationError::InvalidFieldFormat(
                "Timestamp too far in the past".to_string()
            ).into());
        }
        
        // Validate event ID matches content
        if !self.verify_id() {
            return Err(NostrError::InvalidEvent(
                "Event ID does not match content".to_string()
            ));
        }
        
        // Validate signature
        if !self.verify_signature()? {
            return Err(NostrError::InvalidSignature);
        }
        
        // Validate kind-specific rules
        self.validate_kind_specific()?;
        
        Ok(())
    }
}

impl Event {
    /// Validate kind-specific rules
    fn validate_kind_specific(&self) -> Result<(), NostrError> {
        match self.kind {
            // Text note
            1 => {
                // No specific validation for text notes
                Ok(())
            },
            
            // Contact list (kind 3)
            3 => {
                // Should be replaceable
                if !self.is_replaceable() && self.kind != 3 {
                    return Err(NostrError::InvalidEvent(
                        "Contact list should be replaceable".to_string()
                    ));
                }
                Ok(())
            },
            
            // DM (kind 4) - deprecated but still validate
            4 => {
                // Should have exactly one 'p' tag for recipient
                let p_tags: Vec<_> = self.tags.iter()
                    .filter(|tag| tag.tag_name() == Some("p"))
                    .collect();
                
                if p_tags.len() != 1 {
                    return Err(NostrError::InvalidEvent(
                        "DM must have exactly one 'p' tag".to_string()
                    ));
                }
                Ok(())
            },
            
            // Deletion (kind 5)
            5 => {
                // Must have at least one 'e' tag referencing events to delete
                let has_e_tag = self.tags.iter()
                    .any(|tag| tag.tag_name() == Some("e"));
                
                if !has_e_tag {
                    return Err(NostrError::InvalidEvent(
                        "Deletion event must have at least one 'e' tag".to_string()
                    ));
                }
                Ok(())
            },
            
            // Reaction (kind 7)
            7 => {
                // Should have an 'e' tag referencing the event being reacted to
                let has_e_tag = self.tags.iter()
                    .any(|tag| tag.tag_name() == Some("e"));
                
                if !has_e_tag {
                    return Err(NostrError::InvalidEvent(
                        "Reaction must reference an event with 'e' tag".to_string()
                    ));
                }
                Ok(())
            },
            
            // Metadata (kind 0)
            0 => {
                // Content should be valid JSON
                if !self.content.is_empty() {
                    serde_json::from_str::<serde_json::Value>(&self.content)
                        .map_err(|_| NostrError::InvalidEvent(
                            "Metadata content must be valid JSON".to_string()
                        ))?;
                }
                Ok(())
            },
            
            // Parameterized replaceable events (30000-39999)
            30000..=39999 => {
                // Must have a 'd' tag for the identifier
                if self.d_tag().is_none() {
                    return Err(NostrError::InvalidEvent(
                        "Parameterized replaceable event must have a 'd' tag".to_string()
                    ));
                }
                Ok(())
            },
            
            // Regular events and others
            _ => Ok(()),
        }
    }
}

/// Validator for subscription filters
pub struct FilterValidator;

impl FilterValidator {
    /// Validate that a filter is reasonable and not abusive
    pub fn validate_subscription_filters(filters: &[crate::filter::Filter]) -> Result<(), ValidationError> {
        if filters.is_empty() {
            return Err(ValidationError::InvalidFieldFormat(
                "At least one filter is required".to_string()
            ));
        }
        
        if filters.len() > 10 {
            return Err(ValidationError::InvalidFieldFormat(
                "Too many filters in subscription".to_string()
            ));
        }
        
        for filter in filters {
            Self::validate_single_filter(filter)?;
        }
        
        Ok(())
    }
    
    fn validate_single_filter(filter: &crate::filter::Filter) -> Result<(), ValidationError> {
        // Check limits
        if let Some(limit) = filter.limit {
            if limit > 5000 {
                return Err(ValidationError::InvalidFieldFormat(
                    "Filter limit too high (max 5000)".to_string()
                ));
            }
        }
        
        // Check time range
        if let (Some(since), Some(until)) = (filter.since, filter.until) {
            if since >= until {
                return Err(ValidationError::InvalidFieldFormat(
                    "Invalid time range: since >= until".to_string()
                ));
            }
            
            // Prevent queries for very large time ranges without other constraints
            let time_range = until - since;
            if time_range > 86400 * 30 && // 30 days
               filter.ids.is_none() && 
               filter.authors.is_none() &&
               filter.kinds.is_none() {
                return Err(ValidationError::InvalidFieldFormat(
                    "Time range too large without other constraints".to_string()
                ));
            }
        }
        
        // Check array sizes
        if let Some(ref ids) = filter.ids {
            if ids.len() > 1000 {
                return Err(ValidationError::InvalidFieldFormat(
                    "Too many IDs in filter".to_string()
                ));
            }
        }
        
        if let Some(ref authors) = filter.authors {
            if authors.len() > 1000 {
                return Err(ValidationError::InvalidFieldFormat(
                    "Too many authors in filter".to_string()
                ));
            }
        }
        
        if let Some(ref kinds) = filter.kinds {
            if kinds.len() > 20 {
                return Err(ValidationError::InvalidFieldFormat(
                    "Too many kinds in filter".to_string()
                ));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{PublicKey, Signature};
    use crate::event::EventBuilder;
    use crate::filter::Filter;
    
    #[test]
    fn test_event_validation() {
        let pubkey = PublicKey::new("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string()).unwrap();
        let sig = Signature::new("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string()).unwrap();
        
        let unsigned = EventBuilder::new()
            .pubkey(pubkey)
            .kind(1)
            .content("Hello Nostr!")
            .created_at(chrono::Utc::now().timestamp())
            .build_unsigned()
            .unwrap();
        
        let mut event = unsigned.sign(sig);
        
        // This will fail signature validation but we can test other validations
        match event.validate() {
            Err(NostrError::InvalidSignature) => {
                // Expected - we're using a mock signature
            },
            Err(e) => panic!("Unexpected validation error: {:?}", e),
            Ok(_) => panic!("Expected validation to fail with invalid signature"),
        }
        
        // Test content too long
        event.content = "x".repeat(MAX_CONTENT_LENGTH + 1);
        match event.validate() {
            Err(NostrError::ValidationError(ValidationError::ContentTooLong { .. })) => {
                // Expected
            },
            _ => panic!("Expected content too long error"),
        }
    }
    
    #[test]
    fn test_filter_validation() {
        // Valid filter
        let filter = Filter::new().kind(1).limit(100);
        assert!(FilterValidator::validate_subscription_filters(&[filter]).is_ok());
        
        // Invalid limit
        let filter = Filter::new().limit(10000);
        assert!(FilterValidator::validate_subscription_filters(&[filter]).is_err());
        
        // Too many filters
        let filters = vec![Filter::new(); 15];
        assert!(FilterValidator::validate_subscription_filters(&filters).is_err());
    }
}
