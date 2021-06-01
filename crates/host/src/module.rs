use holochain_wasmer_common::WasmError;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use wasmer::Module;
use wasmer::Store;
use wasmer::Universal;

#[derive(Default)]
pub struct ModuleCache(HashMap<[u8; 32], Arc<Module>>);

pub static MODULE_CACHE: Lazy<RwLock<ModuleCache>> =
    Lazy::new(|| RwLock::new(ModuleCache::default()));

impl ModuleCache {
    fn get_with_build_cache(
        &mut self,
        key: [u8; 32],
        wasm: &[u8],
    ) -> Result<Arc<Module>, WasmError> {
        let store = create_store();
        let module =
            Module::from_binary(&store, wasm).map_err(|e| WasmError::Compile(e.to_string()))?;
        self.0.insert(key, Arc::new(module));
        self.get(key, wasm)
    }

    pub fn get(&mut self, key: [u8; 32], wasm: &[u8]) -> Result<Arc<Module>, WasmError> {
        match self.0.get(&key) {
            Some(module) => Ok(Arc::clone(module)),
            None => self.get_with_build_cache(key, wasm),
        }
    }
}

#[cfg(target_arch = "aarch64")]
fn create_store() -> Store {
    let compiler = wasmer_compiler_cranelift::Cranelift::new();
    Store::new(&Universal::new(compiler).engine())
}

#[cfg(not(target_arch = "aarch64"))]
fn create_store() -> Store {
    let compiler = wasmer_compiler_singlepass::Singlepass::new();
    Store::new(&Universal::new(compiler).engine())
}