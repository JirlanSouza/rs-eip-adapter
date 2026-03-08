use thiserror::Error;

use super::ClassCode;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("class {class_id} not found")]
    ClassNotFound { class_id: ClassCode },

    #[error("instance {instance_id} for class {class_id} not found")]
    InstanceNotFound {
        class_id: ClassCode,
        instance_id: u16,
    },

    #[error("failed to downcast class {class_id} to requested type")]
    DowncastFailed { class_id: ClassCode },
}
