//=============================================================================//
// File Path: Element/Maintain/Source/Build/Definition/GuardDefinition.rs
//=============================================================================//
// Module: GuardDefinition
//
// Brief Description: Defines the Guard RAII pattern for file backup and restoration.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Backup original files before modification
// - Automatically restore files when dropped (RAII pattern)
// - Ensure cleanup even on panic or early return
//
// Secondary:
// - Prevent accidental overwriting of existing backups
// - Provide access to file paths for the build system
//
// ARCHITECTURAL ROLE:
// ===================
//
// Position:
// - Infrastructure/Resource management layer
// - RAII pattern implementation
//
// Dependencies (What this module requires):
// - External crates: std (fs, path, log)
// - Internal modules: Error::BuildError, Constant::BackupSuffixConstant
// - Traits implemented: Drop
//
// Dependents (What depends on this module):
// - Build orchestration functions
// - Configuration editing functions
//
// IMPLEMENTATION DETAILS:
// =======================
//
// Design Patterns:
// - RAII (Resource Acquisition Is Initialization) pattern
// - Guard/Scope guard pattern
//
// Performance Considerations:
// - Complexity: O(n) - file copy operation
// - Memory usage patterns: Two PathBuf fields (original and backup paths)
// - Hot path optimizations: None needed
//
// Thread Safety:
// - Thread-safe: No (not designed for concurrent access)
// - Synchronization mechanisms used: None
// - Interior mutability considerations: None
//
// Error Handling:
// - Error types returned: BuildError
// - Recovery strategies: Return error if backup already exists
//
// EXAMPLES:
// =========
//
// Example 1: Creating a guard for a configuration file
/// ```rust
/// use crate::Maintain::Source::Build::Definition::GuardDefinition;
/// let cargo_path = PathBuf::from("Element/Mountain/Cargo.toml");
/// let _guard = Guard::New(cargo_path, "Cargo.toml".to_string())?;
/// // File is now backed up
/// // ... modify the file ...
/// // Guard restores original when it goes out of scope
/// ```
//
// Example 2: Automatic restoration on error
/// ```rust
/// use crate::Maintain::Source::Build::Definition::GuardDefinition;
/// let config_path = PathBuf::from("tauri.conf.json");
/// let _guard = Guard::New(config_path, "Tauri config".to_string())?;
/// modify_config()?;
/// // If modify_config fails, Guard still restores the file
/// ```
//
// Example 3: Accessing the original and backup paths
/// ```rust
/// use crate::Maintain::Source::Build::Definition::GuardDefinition;
/// let guard = Guard::New(original_path, description.to_string())?;
/// println!("Original: {}", guard.Path().display());
/// println!("Backup: {}", guard.Store().display());
/// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

use crate::Build::{BackupSuffix, BuildError};

use log::{error, info};
use std::{fs, path::Path, path::PathBuf};

/// Manages the backup and restoration of a single file using the RAII pattern.
///
/// This struct ensures that an original file is restored to its initial state
/// when the Guard goes out of scope, providing a safe way to temporarily
/// modify configuration files during the build process.
///
/// # RAII Pattern
///
/// The Guard implements the RAII (Resource Acquisition Is Initialization)
/// pattern:
/// - **Acquisition**: When created, it creates a backup of the original file
/// - **Release**: When dropped, it restores the original file from the backup
///
/// This ensures that files are restored even if:
/// - The function returns early
/// - An error occurs and propagates up
/// - A panic occurs
///
/// # Backup Naming
///
/// Backup files are created by appending a suffix to the original file extension:
/// - `Cargo.toml` → `Cargo.toml.Backup`
/// - `tauri.conf.json` → `tauri.conf.json.Backup`
///
/// # Error Handling
///
/// The Guard constructor returns an error if:
/// - A backup file already exists at the target location
/// - The original file cannot be copied (IO error)
///
/// This prevents accidental overwriting of existing backups and ensures
/// data safety.
///
/// # Fields
///
/// - **Path**: The path to the original file
/// - **Store**: The path to the backup file
/// - **Active**: Whether a backup was created
/// - **Note**: A descriptive note for logging purposes
pub struct Guard {
	/// The path to the original file that will be modified.
	Path: PathBuf,

	/// The path to the backup file created by this guard.
	Store: PathBuf,

	/// Whether a backup was actually created
	/// (false if the original file didn't exist).
	Active: bool,

	/// A descriptive note for logging and debugging purposes.
	#[allow(dead_code)]
	Note: String,
}

impl Guard {
	/// Creates a new Guard that backs up the specified file.
	///
	/// This constructor creates a backup of the original file if it exists,
	/// and prepares to restore it when the Guard is dropped. The backup
	/// file is created with a special suffix to prevent accidental conflicts.
	///
	/// # Parameters
	///
	/// * `OriginalPath` - The path to the file to be backed up
	/// * `Description` - A descriptive note for logging purposes
	///
	/// # Returns
	///
	/// Returns a `Result` containing the Guard or a `BuildError` if:
	/// - A backup file already exists
	/// - The file cannot be copied
	///
	/// # Errors
	///
	/// * `BuildError::Exists` - If a backup file already exists
	/// * `BuildError::Io` - If the file copy operation fails
	///
	/// # Example
	///
	/// ```no_run
	/// use crate::Maintain::Source::Build::Definition::GuardDefinition;
	/// let path = PathBuf::from("Cargo.toml");
	/// let guard = Guard::New(path, "Cargo manifest".to_string())?;
	/// ```
	///
	/// # Safety
	///
	/// The Guard ensures that the original file is restored even in the
	/// presence of panics, providing exception safety.
	pub fn New(OriginalPath: PathBuf, Description: String) -> Result<Self, BuildError> {
		let BackupPath = OriginalPath.with_extension(format!(
			"{}{}",
			OriginalPath.extension().unwrap_or_default().to_str().unwrap_or(""),
			BackupSuffix
		));

		if BackupPath.exists() {
			error!("Backup file {} already exists.", BackupPath.display());

			return Err(BuildError::Exists(BackupPath));
		}

		let mut BackupMade = false;

		if OriginalPath.exists() {
			fs::copy(&OriginalPath, &BackupPath)?;

			info!(
				target: "Build::Guard",
				"Backed {} to {}",
				OriginalPath.display(),
				BackupPath.display()
			);

			BackupMade = true;
		}

		Ok(Self {
			Path: OriginalPath,
			Store: BackupPath,
			Active: BackupMade,
			Note: Description,
		})
	}

	/// Returns a reference to the original file path.
	///
	/// This method provides read access to the path of the original file
	/// that this guard is protecting.
	///
	/// # Returns
	///
	/// A reference to the original file's `Path`.
	///
	/// # Example
	///
	/// ```no_run
	/// use crate::Maintain::Source::Build::Definition::GuardDefinition;
	/// let guard = Guard::New(original_path, description.to_string())?;
	/// println!("Original file: {}", guard.Path().display());
	/// ```
	pub fn Path(&self) -> &Path {
		&self.Path
	}

	/// Returns a reference to the backup file path.
	///
	/// This method provides read access to the path where the backup
	/// file is stored.
	///
	/// # Returns
	///
	/// A reference to the backup file's `Path`.
	///
	/// # Example
	///
	/// ```no_run
	/// use crate::Maintain::Source::Build::Definition::GuardDefinition;
	/// let guard = Guard::New(original_path, description.to_string())?;
	/// println!("Backup file: {}", guard.Store().display());
	/// ```
	pub fn Store(&self) -> &Path {
		&self.Store
	}
}

/// Drop implementation that automatically restores the original file.
///
/// This is the core of the RAII pattern: when the Guard goes out of scope,
/// it automatically restores the original file from the backup. This ensures
/// that file modifications are temporary and that the system is left in a
/// consistent state even if an error occurs.
///
/// # Behavior
///
/// - If no backup was created (original file didn't exist), does nothing
/// - If backup exists, copies it back to the original location
/// - After successful restore, deletes the backup file
/// - Logs success or failure of the restoration process
///
/// # Panics
///
/// This implementation handles panics internally and does not propagate them.
/// If the restoration fails, it logs an error but does not panic, ensuring
/// that cleanup failures don't cause secondary failures.
impl Drop for Guard {
	fn drop(&mut self) {
		if self.Active && self.Store.exists() {
			info!(
				target: "Build::Guard",
				"Restoring {} from {}...",
				self.Path.display(),
				self.Store.display()
			);

			if let Ok(_) = fs::copy(&self.Store, &self.Path) {
				info!(target: "Build::Guard", "Restore successful.");

				if let Err(e) = fs::remove_file(&self.Store) {
					error!(
						target: "Build::Guard",
						"Failed to delete backup {}: {}",
						self.Store.display(),
						e
					);
				}
			} else if let Err(e) = fs::copy(&self.Store, &self.Path) {
				error!(
					target: "Build::Guard",
					"Restore FAILED: {}. {} is now inconsistent.",
					e,
					self.Path.display()
				);
			}
		}
	}
}
