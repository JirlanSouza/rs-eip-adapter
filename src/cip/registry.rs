use crate::cip::{CipClassId, cip_class::CipClass};
use std::{collections::HashMap, sync::Arc};

pub struct Registry {
    classes: HashMap<u16, Arc<dyn CipClass>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
        }
    }

    pub fn register(&mut self, class: Arc<dyn CipClass>) {
        self.classes.insert(class.class_id(), class);
    }

    pub fn get(&self, class_id: u16) -> Option<Arc<dyn CipClass>> {
        self.classes.get(&class_id).cloned()
    }

    pub fn get_instance<T: 'static + Send + Sync>(
        &self,
        class_id: CipClassId,
        instance_id: u16,
    ) -> Result<Arc<T>, String> {
        let class = self
            .get(class_id.to_u16())
            .ok_or(format!("Class {} not found", class_id))?;
        let instance_ptr = class
            .instance(instance_id)
            .map_err(|_| format!("Instance {} for class {} not found", instance_id, class_id))?;

        let any_arc = instance_ptr.as_any_arc();
        any_arc
            .downcast::<T>()
            .map_err(|_| format!("Failed to downcast class {} to requested type", class_id))
    }
}
