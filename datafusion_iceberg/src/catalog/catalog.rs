use std::{any::Any, sync::Arc};

use datafusion::{
    catalog::{CatalogProvider, SchemaProvider},
    error::Result,
};
use iceberg_rust::catalog::{namespace::Namespace, Catalog};

use crate::catalog::{mirror::Mirror, schema::IcebergSchema};

#[derive(Debug)]
pub struct IcebergCatalog {
    catalog: Arc<Mirror>,
}

impl IcebergCatalog {
    pub async fn new(catalog: Arc<dyn Catalog>, branch: Option<&str>) -> Result<Self> {
        Ok(IcebergCatalog {
            catalog: Arc::new(Mirror::new(catalog, branch.map(ToOwned::to_owned)).await?),
        })
    }

    pub fn new_sync(catalog: Arc<dyn Catalog>, branch: Option<&str>) -> Self {
        IcebergCatalog {
            catalog: Arc::new(Mirror::new_sync(catalog, branch.map(ToOwned::to_owned))),
        }
    }

    pub fn catalog(&self) -> Arc<dyn Catalog> {
        self.catalog.catalog()
    }

    pub fn mirror(&self) -> Arc<Mirror> {
        self.catalog.clone()
    }
}

impl CatalogProvider for IcebergCatalog {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn schema_names(&self) -> Vec<String> {
        let namespaces = self.catalog.schema_names(None);
        match namespaces {
            Err(_) => vec![],
            Ok(namespaces) => namespaces.into_iter().map(|x| x.to_string()).collect(),
        }
    }
    fn schema(&self, name: &str) -> Option<Arc<dyn SchemaProvider>> {
        if !self.catalog.schema_exists(name) {
            return None;
        }
        Some(Arc::new(IcebergSchema::new(
            Namespace::try_new(
                &name
                    .split('.')
                    .map(|z| z.to_owned())
                    .collect::<Vec<String>>(),
            )
            .ok()?,
            Arc::clone(&self.catalog),
        )) as Arc<dyn SchemaProvider>)
    }

    fn register_schema(
        &self,
        name: &str,
        _schema: Arc<dyn SchemaProvider>,
    ) -> Result<Option<Arc<dyn SchemaProvider>>> {
        let old_namespace = self.catalog.register_schema(name)?;
        if old_namespace.is_some() {
            Ok(self.schema(name))
        } else {
            Ok(None)
        }
    }

    fn deregister_schema(
        &self,
        name: &str,
        _cascade: bool,
    ) -> Result<Option<Arc<dyn SchemaProvider>>> {
        if self.catalog.deregister_schema(name)?.is_some() {
            Ok(self.schema(name))
        } else {
            Ok(None)
        }
    }
}
