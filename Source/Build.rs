//! # Dynamic Build Orchestrator
//!
//! This binary serves as a powerful, configurable pre-build step for a Tauri
//! application. It is designed to be called from a shell script and dynamically
//! modifies project configuration files (`Cargo.toml`, `tauri.conf.json`) based
//! on environment variables and command-line arguments.
//!
//! Its primary responsibilities include:
//! - Generating a unique `productName` and `identifier` for different build
//!   "flavours" (e.g., for different dependencies, environments, or feature
//!   sets).
//! - Dynamically selecting a sidecar binary (like a specific Node.js version),
//!
//!   staging it in a temporary location, and configuring Tauri to bundle it.
//! - Temporarily modifying configuration files, executing the final build
//!   command (e.g., `pnpm tauri build`), and then restoring the original files
//!   using a `Guard` pattern.

#![allow(non_snake_case, non_upper_case_globals)]

// --- Constants: File Paths and Delimiters ---

/// Default project directory relative to the workspace root.
pub const DirectoryDefault:&str = "Element/Mountain";

/// Default project base name, used as a suffix for generated names.
pub const NameDefault:&str = "Mountain";

/// Default bundle identifier prefix.
pub const PrefixDefault:&str = "land.editor.binary";

/// Cargo configuration filename.
pub const CargoFile:&str = "Cargo.toml";

/// Tauri JSON5 configuration filename.
pub const JsonfiveFile:&str = "tauri.conf.json5";

/// Tauri JSON configuration filename.
pub const JsonFile:&str = "tauri.conf.json";

/// Suffix used for backup files created by the `Guard`.
pub const BackupSuffix:&str = ".Backup";

/// Delimiter for parts of the generated `productName`.
pub const NameDelimiter:&str = "_";

/// Delimiter for parts of the generated bundle `identifier`.
pub const IdDelimiter:&str = ".";

// --- Constants: Environment Variable Names ---

/// Environment variable for the project directory.
pub const DirEnv:&str = "MOUNTAIN_DIR";

/// Environment variable for the original base name of the project.
pub const NameEnv:&str = "MOUNTAIN_ORIGINAL_BASE_NAME";

/// Environment variable for the bundle identifier prefix.
pub const PrefixEnv:&str = "MOUNTAIN_BUNDLE_ID_PREFIX";

/// Environment variable for the "Bundle" build flag.
pub const BundleEnv:&str = "Bundle";

/// Environment variable for the "Browser" build flag.
pub const BrowserEnv:&str = "Browser";

/// Environment variable for the "Compile" build flag.
pub const CompileEnv:&str = "Compile";

/// Environment variable for the "Clean" build flag.
pub const CleanEnv:&str = "Clean";

/// Environment variable for specifying a dependency flavour.
pub const DependencyEnv:&str = "Dependency";

/// Environment variable for the Node.js environment (`development` or
/// `production`).
pub const NodeEnv:&str = "NODE_ENV";

/// Environment variable for selecting the Node.js sidecar version.
pub const NodeVersionEnv:&str = "NODE_VERSION";

/// Environment variable for setting the log level.
pub const LogEnv:&str = "RUST_LOG";

/// Represents all possible errors that can occur during the build script's
/// execution.
#[derive(Error, Debug)]
pub enum Error {
	#[error("IO: {0}")]
	Io(#[from] io::Error),

	#[error("Toml Editing: {0}")]
	Edit(#[from] toml_edit::TomlError),

	#[error("Toml Parsing: {0}")]
	Parse(#[from] toml::de::Error),

	#[error("Json: {0}")]
	Json(#[from] serde_json::Error),

	#[error("Json5: {0}")]
	Jsonfive(#[from] json5::Error),

	#[error("Missing Directory: {0}")]
	Missing(PathBuf),

	#[error("Command Failed: {0}")]
	Shell(std::process::ExitStatus),

	#[error("No Command Provided")]
	NoCommand,

	#[error("Tauri Configuration File Not Found")]
	Config,

	#[error("Backup File Exists: {0}")]
	Exists(PathBuf),

	#[error("UTF-8 Conversion: {0}")]
	Utf(#[from] std::string::FromUtf8Error),

	#[error("Environment Variable Missing: {0}")]
	Environment(String),
}

/// Represents parsed command-line arguments and environment variables that
/// control the build.
#[derive(Parser, Debug, Clone)]
#[clap(
	author,
	version,
	about = "Prepares, builds, and restores project configurations."
)]
pub struct Argument {
	/// The main directory of the project.
	#[clap(long, env = DirEnv, default_value = DirectoryDefault)]
	Directory:String,

	/// The original base name of the project/package.
	#[clap(long, env = NameEnv, default_value = NameDefault)]
	Name:String,

	/// The prefix for the application's bundle identifier.
	#[clap(long, env = PrefixEnv, default_value = PrefixDefault)]
	Prefix:String,

	/// Flag or value indicating browser-specific build aspects.
	#[clap(long, env = BrowserEnv)]
	Browser:Option<String>,

	/// Flag or value indicating bundling-specific aspects.
	#[clap(long, env = BundleEnv)]
	Bundle:Option<String>,

	/// Flag or value indicating bundling-specific aspects.
	#[clap(long, env = CompileEnv)]
	Compile:Option<String>,

	/// Flag or value indicating cleaning-specific aspects.
	#[clap(long, env = CleanEnv)]
	Clean:Option<String>,

	/// Information about a dependency, often 'org/repo' or a boolean string.
	#[clap(long, env = DependencyEnv)]
	Dependency:Option<String>,

	/// The Node.js environment (e.g., "development", "production").
	#[clap(long, env = NodeEnv)]
	Environment:Option<String>,

	/// Specifies the Node.js sidecar version to bundle (e.g., "22").
	#[clap(long, env = NodeVersionEnv)]
	NodeVersion:Option<String>,

	/// The build command and its arguments to execute.
	#[clap(required = true, last = true)]
	Command:Vec<String>,
}

/// Represents the `package` section of a `Cargo.toml` manifest.
#[derive(Deserialize, Debug)]
pub struct Manifest {
	package:Meta,
}

/// Represents metadata within the `package` section of `Cargo.toml`.
#[derive(Deserialize, Debug)]
pub struct Meta {
	version:String,
}

/// Manages the backup and restoration of a single file using the RAII pattern.
/// Ensures that an original file is restored to its initial state when this
/// struct goes out of scope.
pub struct Guard {
	Path:PathBuf,

	Store:PathBuf,

	Active:bool,

	#[allow(dead_code)]
	Note:String,
}

impl Guard {
	pub fn New(OriginalPath:PathBuf, Description:String) -> Result<Self, Error> {
		let BackupPath = OriginalPath.with_extension(format!(
			"{}{}",
			OriginalPath.extension().unwrap_or_default().to_str().unwrap_or(""),
			BackupSuffix
		));

		if BackupPath.exists() {
			error!("Backup file {} already exists.", BackupPath.display());

			return Err(Error::Exists(BackupPath));
		}

		let mut BackupMade = false;

		if OriginalPath.exists() {
			fs::copy(&OriginalPath, &BackupPath)?;

			info!(target: "Build::Guard", "Backed {} to {}", OriginalPath.display(), BackupPath.display());

			BackupMade = true;
		}

		Ok(Self { Path:OriginalPath, Store:BackupPath, Active:BackupMade, Note:Description })
	}

	pub fn Path(&self) -> &Path { &self.Path }

	pub fn Store(&self) -> &Path { &self.Store }
}

impl Drop for Guard {
	fn drop(&mut self) {
		if self.Active && self.Store.exists() {
			info!(target: "Build::Guard", "Restoring {} from {}...", self.Path.display(), self.Store.display());

			if let Ok(_) = fs::copy(&self.Store, &self.Path) {
				info!(target: "Build::Guard", "Restore successful.");

				if let Err(e) = fs::remove_file(&self.Store) {
					error!(target: "Build::Guard", "Failed to delete backup {}: {}", self.Store.display(), e);
				}
			} else if let Err(e) = fs::copy(&self.Store, &self.Path) {
				error!(target: "Build::Guard", "Restore FAILED: {}. {} is now inconsistent.", e, self.Path.display());
			}
		}
	}
}

/// Dynamically modifies specific name fields within a `Cargo.toml` file. This
/// includes `package.name`, `package.default-run`, and `bin.name`.
pub fn TomlEdit(File:&Path, Old:&str, Current:&str) -> Result<bool, Error> {
	debug!(target: "Build::Toml", "Attempting to modify TOML file: {}", File.display());

	let Data = fs::read_to_string(File)?;

	if Old == Current {
		info!(target: "Build::Toml", "Old name '{}' is the same as current name '{}'. No changes needed for {}.", Old, Current, File.display());

		return Ok(false);
	}

	debug!(target: "Build::Toml", "Old name: '{}', Current name: '{}'", Old, Current);

	let mut Parsed:TomlDocument = Data.parse()?;

	let mut PackageChange = false;

	let mut LibraryChange = false;

	let mut BinaryChange = false;

	let mut DefaultChange = false;

	if let Some(PackageTable) = Parsed.get_mut("package").and_then(|Item| Item.as_table_mut()) {
		if let Some(NameItem) = PackageTable.get_mut("name") {
			if NameItem.as_str() == Some(Old) {
				*NameItem = TomlItem::Value(TomlValue::String(toml_edit::Formatted::new(Current.to_string())));

				PackageChange = true;

				debug!(target: "Build::Toml", "Changed package.name");
			}
		}

		if let Some(RunItem) = PackageTable.get_mut("default-run") {
			if RunItem.as_str() == Some(Old) {
				*RunItem = TomlItem::Value(TomlValue::String(toml_edit::Formatted::new(Current.to_string())));

				DefaultChange = true;

				debug!(target: "Build::Toml", "Changed package.default-run");
			}
		}
	}

	if let Some(LibTable) = Parsed.get_mut("lib").and_then(|Item| Item.as_table_mut()) {
		if let Some(NameItem) = LibTable.get_mut("name") {
			if NameItem.as_str() == Some(Old) {
				*NameItem = TomlItem::Value(TomlValue::String(toml_edit::Formatted::new(Current.to_string())));

				LibraryChange = true;

				debug!(target: "Build::Toml", "Changed lib.name");
			}
		}
	}

	if let Some(BinArray) = Parsed.get_mut("bin").and_then(|Item| Item.as_array_of_tables_mut()) {
		for Table in BinArray.iter_mut() {
			if let Some(NameItem) = Table.get_mut("name") {
				if NameItem.as_str() == Some(Old) {
					*NameItem = TomlItem::Value(TomlValue::String(toml_edit::Formatted::new(Current.to_string())));

					BinaryChange = true;

					debug!(target: "Build::Toml", "Changed a bin.name entry to '{}'", Current);

					break;
				}
			}
		}
	}

	if PackageChange || LibraryChange || BinaryChange || DefaultChange {
		let Output = Parsed.to_string();

		fs::write(File, Output)?;

		let mut ModifiedItems = Vec::new();

		if PackageChange {
			ModifiedItems.push("package.name");
		}

		if DefaultChange {
			ModifiedItems.push("package.default-run");
		}

		if LibraryChange {
			ModifiedItems.push("lib.name");
		}

		if BinaryChange {
			ModifiedItems.push("bin.name");
		}

		info!(target: "Build::Toml", "Temporarily changed {} in {} to: {}", ModifiedItems.join(", "), File.display(), Current);

		Ok(true)
	} else {
		warn!(target: "Build::Toml", "Name '{}' not found in relevant sections of {}. No changes made to file.", Old, File.display());

		Ok(false)
	}
}

/// Dynamically modifies fields in a `tauri.conf.json` or `tauri.conf.json5`
/// file, including the sidecar path.
pub fn JsonEdit(File:&Path, Product:&str, Id:&str, Version:&str, SidecarPath:Option<&str>) -> Result<bool, Error> {
	debug!(target: "Build::Json", "Attempting to modify JSON file: {}", File.display());

	let Data = fs::read_to_string(File)?;

	let mut Parsed:JsonValue = if File.extension().and_then(|s| s.to_str()) == Some("json5") {
		json5::from_str(&Data)?
	} else {
		serde_json::from_str(&Data)?
	};

	let mut Modified = false;

	let Root = Parsed
		.as_object_mut()
		.ok_or_else(|| Error::Io(io::Error::new(io::ErrorKind::InvalidData, "JSON root is not an object")))?;

	if Root.get("version").and_then(JsonValue::as_str) != Some(Version) {
		Root.insert("version".to_string(), JsonValue::String(Version.to_string()));

		Modified = true;
	}

	if Root.get("productName").and_then(JsonValue::as_str) != Some(Product) {
		Root.insert("productName".to_string(), JsonValue::String(Product.to_string()));

		Modified = true;
	}

	if Root.get("identifier").and_then(JsonValue::as_str) != Some(Id) {
		Root.insert("identifier".to_string(), JsonValue::String(Id.to_string()));

		Modified = true;
	}

	if let Some(Path) = SidecarPath {
		let Bundle = Root
			.entry("bundle")
			.or_insert_with(|| JsonValue::Object(Default::default()))
			.as_object_mut()
			.unwrap();

		let Bins = Bundle
			.entry("externalBin")
			.or_insert_with(|| JsonValue::Array(Default::default()))
			.as_array_mut()
			.unwrap();

		Bins.push(JsonValue::String(Path.to_string()));

		Modified = true;
	}

	if Modified {
		let mut Buffer = Vec::new();

		let Formatter = serde_json::ser::PrettyFormatter::with_indent(b"\t");

		let mut Serializer = serde_json::Serializer::with_formatter(&mut Buffer, Formatter);

		Parsed.serialize(&mut Serializer)?;

		fs::write(File, String::from_utf8(Buffer)?)?;

		info!(target: "Build::Json", "Dynamically configured {}", File.display());
	}

	Ok(Modified)
}

/// Converts a kebab-case or snake_case string to `PascalCase`.
pub fn Pascalize(Text:&str) -> String {
	Text.split(|c:char| c == '-' || c == '_')
		.filter(|s| !s.is_empty())
		.map(|s| {
			let mut c = s.chars();

			c.next()
				.map_or(String::new(), |f| f.to_uppercase().collect::<String>() + c.as_str())
		})
		.collect()
}

/// Converts a `PascalCase` string into a vector of its lowercase constituent
/// words.
fn WordsFromPascal(Text:&str) -> Vec<String> {
	if Text.is_empty() {
		return Vec::new();
	}

	let mut Words = Vec::new();

	let mut CurrentWord = String::new();

	let mut LastCharWasUppercase = false;

	for Char in Text.chars() {
		if Char.is_uppercase() {
			if !CurrentWord.is_empty() && !LastCharWasUppercase {
				Words.push(CurrentWord.to_ascii_lowercase());

				CurrentWord.clear();
			}

			CurrentWord.push(Char);

			LastCharWasUppercase = true;
		} else {
			CurrentWord.push(Char);

			LastCharWasUppercase = false;
		}
	}

	if !CurrentWord.is_empty() {
		Words.push(CurrentWord.to_ascii_lowercase());
	}

	Words
}

/// Gets the Tauri-compatible target triple for the current build environment.
fn GetTauriTargetTriple() -> String {
	let Os = env::consts::OS;

	let Arch = env::consts::ARCH;

	match (Os, Arch) {
		("windows", "x86_64") => "x86_64-pc-windows-msvc".to_string(),

		("linux", "x86_64") => "x86_64-unknown-linux-gnu".to_string(),

		("linux", "aarch64") => "aarch64-unknown-linux-gnu".to_string(),

		("macos", "x86_64") => "x86_64-apple-darwin".to_string(),

		("macos", "aarch64") => "aarch64-apple-darwin".to_string(),

		_ => panic!("Unsupported OS-Arch for sidecar: {}-{}", Os, Arch),
	}
}

/// Main orchestration logic for preparing and executing the build.
pub fn Process(Argument:&Argument) -> Result<(), Error> {
	info!(target: "Build", "Starting build orchestration...");

	debug!(target: "Build", "Argument: {:?}", Argument);

	let ProjectDir = PathBuf::from(&Argument.Directory);

	if !ProjectDir.is_dir() {
		return Err(Error::Missing(ProjectDir));
	}

	let CargoPath = ProjectDir.join(CargoFile);

	let ConfigPath = {
		let Jsonfive = ProjectDir.join(JsonfiveFile);

		if Jsonfive.exists() { Jsonfive } else { ProjectDir.join(JsonFile) }
	};

	if !ConfigPath.exists() {
		return Err(Error::Config);
	}

	let _CargoGuard = Guard::New(CargoPath.clone(), "Cargo.toml".to_string())?;

	let _ConfigGuard = Guard::New(ConfigPath.clone(), "Tauri config".to_string())?;

	let mut NamePartsForProductName = Vec::new();

	let mut NamePartsForId = Vec::new();

	if let Some(NodeValue) = &Argument.Environment {
		if !NodeValue.is_empty() {
			let PascalEnv = Pascalize(NodeValue);

			if !PascalEnv.is_empty() {
				NamePartsForProductName.push(format!("{}NodeEnvironment", PascalEnv));

				NamePartsForId.extend(WordsFromPascal(&PascalEnv));

				NamePartsForId.push("node".to_string());

				NamePartsForId.push("environment".to_string());
			}
		}
	}

	if let Some(DependencyValue) = &Argument.Dependency {
		if !DependencyValue.is_empty() {
			let (PascalDepBase, IdDepWords) = if DependencyValue.eq_ignore_ascii_case("true") {
				("Generic".to_string(), vec!["generic".to_string()])
			} else if let Some((Org, Repo)) = DependencyValue.split_once('/') {
				(format!("{}{}", Pascalize(Org), Pascalize(Repo)), {
					let mut w = WordsFromPascal(&Pascalize(Org));

					w.extend(WordsFromPascal(&Pascalize(Repo)));

					w
				})
			} else {
				(Pascalize(DependencyValue), WordsFromPascal(&Pascalize(DependencyValue)))
			};

			if !PascalDepBase.is_empty() {
				NamePartsForProductName.push(format!("{}Dependency", PascalDepBase));

				NamePartsForId.extend(IdDepWords);

				NamePartsForId.push("dependency".to_string());
			}
		}
	}

	if let Some(Version) = &Argument.NodeVersion {
		if !Version.is_empty() {
			let PascalVersion = format!("{}NodeVersion", Version);

			NamePartsForProductName.push(PascalVersion.clone());

			NamePartsForId.push("node".to_string());

			NamePartsForId.push(Version.to_string());
		}
	}

	if Argument.Bundle.as_ref().map_or(false, |v| v == "true") {
		NamePartsForProductName.push("Bundle".to_string());

		NamePartsForId.push("bundle".to_string());
	}

	if Argument.Clean.as_ref().map_or(false, |v| v == "true") {
		NamePartsForProductName.push("Clean".to_string());

		NamePartsForId.push("clean".to_string());
	}

	if Argument.Browser.as_ref().map_or(false, |v| v == "true") {
		NamePartsForProductName.push("Browser".to_string());

		NamePartsForId.push("browser".to_string());
	}

	if Argument.Compile.as_ref().map_or(false, |v| v == "true") {
		NamePartsForProductName.push("Compile".to_string());

		NamePartsForId.push("compile".to_string());
	}

	let ProductNamePrefix = NamePartsForProductName.join(NameDelimiter);

	let FinalName = if !ProductNamePrefix.is_empty() {
		format!("{}{}{}", ProductNamePrefix, NameDelimiter, Argument.Name)
	} else {
		Argument.Name.clone()
	};

	info!(target: "Build", "Final generated product name: '{}'", FinalName);

	NamePartsForId.extend(WordsFromPascal(&Argument.Name));

	let IdSuffix = NamePartsForId
		.into_iter()
		.filter(|s| !s.is_empty())
		.collect::<Vec<String>>()
		.join(IdDelimiter);

	let FinalId = format!("{}{}{}", Argument.Prefix, IdDelimiter, IdSuffix);

	info!(target: "Build", "Generated bundle identifier: '{}'", FinalId);

	if FinalName != Argument.Name {
		TomlEdit(&CargoPath, &Argument.Name, &FinalName)?;
	}

	let AppVersion = toml::from_str::<Manifest>(&fs::read_to_string(&CargoPath)?)?.package.version;

	// --- Sidecar Selection and Staging Logic ---
	let sidecar_bundle_path_for_tauri = if let Some(version) = &Argument.NodeVersion {
		info!(target: "Build", "Selected Node.js version: {}", version);

		let target_triple = GetTauriTargetTriple();

		// Path to the pre-downloaded Node executable
		let source_executable_path = if cfg!(target_os = "windows") {
			PathBuf::from(format!("./Element/SideCar/{}/NODE/{}/node.exe", target_triple, version))
		} else {
			PathBuf::from(format!("./Element/SideCar/{}/NODE/{}/bin/node", target_triple, version))
		};

		// Define a consistent, temporary directory inside `src-tauri` for the staged
		// binary
		let temp_sidecar_dir = ProjectDir.join("Binary");

		fs::create_dir_all(&temp_sidecar_dir)?;

		// Define the consistent name for the binary that Tauri will bundle
		let dest_executable_path = if cfg!(target_os = "windows") {
			temp_sidecar_dir.join(format!("node-{}.exe", target_triple))
		} else {
			temp_sidecar_dir.join(format!("node-{}", target_triple))
		};

		info!(
			target: "Build",

			"Staging sidecar from {} to {}",

			source_executable_path.display(),

			dest_executable_path.display()
		);

		// Perform the copy
		fs::copy(&source_executable_path, &dest_executable_path)?;

		// On non-windows, make sure the copied binary is executable
		#[cfg(not(target_os = "windows"))]
		{
			use std::os::unix::fs::PermissionsExt;

			let mut perms = fs::metadata(&dest_executable_path)?.permissions();

			perms.set_mode(0o755); // rwxr-xr-x
			fs::set_permissions(&dest_executable_path, perms)?;
		}

		Some("Binary/node".to_string())
	} else {
		info!(target: "Build", "No Node.js flavour selected for bundling.");

		None
	};

	// --- End Sidecar Logic ---

	JsonEdit(
		&ConfigPath,
		&FinalName,
		&FinalId,
		&AppVersion,
		sidecar_bundle_path_for_tauri.as_deref(),
	)?;

	if Argument.Command.is_empty() {
		return Err(Error::NoCommand);
	}

	let mut ShellCommand = if cfg!(target_os = "windows") {
		let mut cmd = ProcessCommand::new("cmd");

		cmd.arg("/C").args(&Argument.Command);

		cmd
	} else {
		let mut cmd = ProcessCommand::new(&Argument.Command[0]);

		cmd.args(&Argument.Command[1..]);

		cmd
	};

	info!(target: "Build::Exec", "Executing final build command: {:?}", ShellCommand);

	let Status = ShellCommand
		.current_dir(env::current_dir()?)
		.stdout(Stdio::inherit())
		.stderr(Stdio::inherit())
		.status()?;

	if !Status.success() {
		let temp_sidecar_dir = ProjectDir.join("bin");

		if temp_sidecar_dir.exists() {
			let _ = fs::remove_dir_all(&temp_sidecar_dir);
		}

		return Err(Error::Shell(Status));
	}

	// Final cleanup of the temporary sidecar directory after a successful build
	let temp_sidecar_dir = ProjectDir.join("bin");

	if temp_sidecar_dir.exists() {
		fs::remove_dir_all(&temp_sidecar_dir)?;

		info!(target: "Build", "Cleaned up temporary sidecar directory.");
	}

	info!(target: "Build", "Build orchestration completed successfully.");

	Ok(())
}

/// Sets up the global logger for the application.
pub fn Logger() {
	let LevelText = env::var(LogEnv).unwrap_or_else(|_| "info".to_string());

	let LogLevel = LevelText.parse::<LevelFilter>().unwrap_or(LevelFilter::Info);

	env_logger::Builder::new()
		.filter_level(LogLevel)
		.format(|Buffer, Record| {
			let LevelStyle = match Record.level() {
				log::Level::Error => "ERROR".red().bold(),

				log::Level::Warn => "WARN".yellow().bold(),

				log::Level::Info => "INFO".green(),

				log::Level::Debug => "DEBUG".blue(),

				log::Level::Trace => "TRACE".magenta(),
			};

			writeln!(Buffer, "[{}] [{}]: {}", "Build".red(), LevelStyle, Record.args())
		})
		.parse_default_env()
		.init();
}

/// Verifies if all required environment variables are set.
pub fn VerifyEnv() -> Result<(), Error> { Ok(()) }

/// The main entry point of the binary.
pub fn Fn() {
	Logger();

	if let Err(Error) = VerifyEnv() {
		error!("Failed environment variable check: {}", Error);

		std::process::exit(1);
	}

	let Argument = Argument::parse();

	debug!("Parsed arguments: {:?}", Argument);

	match Process(&Argument) {
		Ok(_) => info!("Build process completed successfully."),

		Err(e) => {
			error!("Build process failed: {}", e);

			std::process::exit(1);
		},
	}
}

/// Main executable function.
#[allow(unused)]
fn main() { Fn(); }

use std::{
	env,
	fs,
	io::{self, Write as IoWriter},
	path::{Path, PathBuf},
	process::{Command as ProcessCommand, Stdio},
};

use clap::{self, Parser};
use colored::*;
use log::{LevelFilter, debug, error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use thiserror::Error;
use toml_edit::{DocumentMut as TomlDocument, Item as TomlItem, Value as TomlValue};
