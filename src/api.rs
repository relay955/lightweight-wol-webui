use validator::Validate;
use crate::error::SystemError;

pub mod user_api;
pub mod catcher;
pub mod device_api;

/// 밸리데이션을 수행하고 에러 메시지를 포맷팅하는 헬퍼 함수
fn validate_request<T: Validate>(data: &T) -> Result<(), SystemError> {
    if let Err(errors) = data.validate() {
        let error_messages: Vec<String> = errors
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |e| {
                    format!("{}: {}", field, e.message.as_ref().unwrap_or(&std::borrow::Cow::Borrowed("유효하지 않은 값")))
                })
            })
            .collect();
        return Err(SystemError::APIError(400, 0, error_messages.join(", ")));
    }
    Ok(())
}

