//! Aeonmi Glyph Identity System
//! Implements the full Glyph-as-Soul spec: MGK, UGST, GDF, Vault, Ceremony, Anomaly.

pub mod anomaly;
pub mod ceremony;
pub mod gdf;
pub mod mgk;
pub mod ugst;
pub mod vault;

pub use ceremony::{boot, init_shard, BootResult};
pub use gdf::GlyphParams;
pub use mgk::MasterGlyphKey;
pub use ugst::{current_ugst, current_window, derive_glyph_seed, derive_ugst};
pub use vault::{GlyphVault, VaultRecord};
