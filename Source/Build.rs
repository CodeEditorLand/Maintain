#![allow(non_snake_case, non_upper_case_globals)]

/// Default project directory.
pub const DirectoryDefault:&str = "Element/Mountain";

/// Default project base name.
pub const NameDefault:&str = "Mountain";

/// Default bundle identifier prefix.
pub const PrefixDefault:&str = "land.editor.binary";

/// Cargo configuration filename.
pub const CargoFile:&str = "Cargo.toml";

/// Tauri JSON5 configuration filename.
pub const JsonfiveFile:&str = "tauri.conf.json5";

/// Tauri JSON configuration filename.
pub const JsonFile:&str = "tauri.conf.json";

/// Backup file suffix.
pub const BackupSuffix:&str = ".Backup";

/// Delimiter for name parts in the final product/package name.
pub const NameDelimiter:&str = "_";

/// Delimiter for bundle identifier parts.
pub const IdDelimiter:&str = ".";

// Environment Variable Names (Constants)
/// Environment variable for project directory.
pub const DirEnv:&str = "MOUNTAIN_DIR";

/// Environment variable for original base name.
pub const NameEnv:&str = "MOUNTAIN_ORIGINAL_BASE_NAME";

/// Environment variable for bundle ID prefix.
pub const PrefixEnv:&str = "MOUNTAIN_BUNDLE_ID_PREFIX";

/// Environment variable for bundle flag.
pub const BundleEnv:&str = "Bundle";

/// Environment variable for browser flag.
pub const BrowserEnv:&str = "Browser";

/// Environment variable for bundle flag.
pub const CompileEnv:&str = "Compile";

/// Environment variable for clean flag.
pub const CleanEnv:&str = "Clean";

/// Environment variable for dependency information.
pub const DependencyEnv:&str = "Dependency";

/// Environment variable for Node.js environment.
pub const NodeEnv:&str = "NODE_ENV";

/// Environment variable for log level.
pub const LogEnv:&str = "RUST_LOG";

/// Represents errors that can occur during the build script execution.
#[derive(Error, Debug)]
pub enum Error {
	/// An I/O error.
	#[error("IO: {0}")]
	Io(#[from] io::Error),

	/// A TOML editing error.
	#[error("Toml Editing: {0}")]
	Edit(#[from] toml_edit::TomlError),

	/// A TOML deserialization error.
	#[error("Toml Parsing: {0}")]
	Parse(#[from] toml::de::Error),

	/// A JSON serialization/deserialization error.
	#[error("Json: {0}")]
	Json(#[from] serde_json::Error),

	/// A JSON5 parsing error.
	#[error("Json5: {0}")]
	Jsonfive(#[from] json5::Error),

	/// A required directory is missing.
	#[error("Missing Directory: {0}")]
	Missing(PathBuf),

	/// An external command failed to execute.
	#[error("Command Failed: {0}")]
	Shell(std::process::ExitStatus),

	/// No build command was provided.
	#[error("No Command Provided")]
	Nocommand,

	/// The Tauri configuration file (JSON or JSON5) was not found.
	#[error("Tauri Configuration File Not Found")]
	Config,

	/// A backup file already exists, preventing a new backup.
	#[error("Backup File Exists: {0}")]
	Exists(PathBuf),

	/// A UTF-8 conversion error.
	#[error("UTF-8 Conversion: {0}")]
	Utf(#[from] std::string::FromUtf8Error),

	/// A required environment variable is missing.
	#[error("Environment Variable Missing: {0}")]
	Environment(String),
}

/// Represents parsed command-line arguments and environment variables.
#[derive(Parser, Debug, Clone)]
#[clap(author, version, about = "Prepares, builds, and restores project.")]
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

	/// The build command and its arguments to execute.
	#[clap(required = true, last = true)]
	Command:Vec<String>,
}

/// Represents the `package` section of a Cargo.toml manifest.
#[derive(Deserialize, Debug)]
pub struct Manifest {
	/// Package metadata.
	package:Meta,
}

/// Represents metadata within the `package` section of Cargo.toml, specifically
/// for versioning.
#[derive(Deserialize, Debug)]
pub struct Meta {
	/// The version string of the package.
	version:String,
}

/// Manages the backup and restoration of a single file.
/// Ensures that an original file is restored to its initial state when this
/// struct goes out of scope.
pub struct Guard {
	/// Path to the original file being managed.
	Path:PathBuf,

	/// Path to the backup copy of the original file.
	Store:PathBuf,

	/// Indicates if a backup was successfully created and is active.
	Active:bool,

	/// A description of the file being guarded, for logging purposes.
	Note:String,
}

impl Guard {
	/// Creates a new `Guard` for a file.
	///
	/// If the original file exists, it's copied to a backup location.
	///
	/// # Parameters
	/// - `OriginalPath`: The path to the file to guard.
	/// - `Description`: A human-readable description for logging.
	///
	/// # Errors
	/// Returns `Error::Exists` if a backup file already exists at the target
	/// location. Returns `Error::Io` if file operations (copying) fail.
	pub fn New(OriginalPath:PathBuf, Description:String) -> Result<Self, Error> {
		let BackupPath = OriginalPath.with_extension(format!(
			"{}{}",
			OriginalPath.extension().unwrap_or_default().to_str().unwrap_or(""),
			BackupSuffix
		));

		if BackupPath.exists() {
			error!(
				"Backup file {} already exists. Please remove it or use a different backup suffix.",
				BackupPath.display()
			);

			return Err(Error::Exists(BackupPath));
		}

		let mut BackupMade = false;

		if OriginalPath.exists() {
			fs::copy(&OriginalPath, &BackupPath)?;

			info!(target: "Build::Guard", "Backed {} to {}", OriginalPath.display(), BackupPath.display());

			BackupMade = true;
		} else {
			warn!(target: "Build::Guard", "Original {} not found, no backup will be created.", OriginalPath.display());
		}

		Ok(Self { Path:OriginalPath, Store:BackupPath, Active:BackupMade, Note:Description })
	}

	/// Returns a reference to the path of the original file.
	pub fn Path(&self) -> &Path { &self.Path }

	/// Returns a reference to the path of the backup file.
	pub fn Store(&self) -> &Path { &self.Store }
}

impl Drop for Guard {
	/// Restores the original file from its backup when the `Guard` is dropped.
	/// Also cleans up the backup file.
	fn drop(&mut self) {
		if self.Active && self.Store.exists() {
			info!(
				target: "Build::Guard",


				"Restoring {} from {}... ",


				self.Path.display(),


				self.Store.display()
			);

			match fs::copy(&self.Store, &self.Path) {
				Ok(_) => {
					info!(target: "Build::Guard", "Restore successful.");

					if let Err(Error) = fs::remove_file(&self.Store) {
						error!(target: "Build::Guard", "Failed delete backup {}: {}", self.Store.display(), Error);
					} else {
						info!(target: "Build::Guard", "Deleted backup {}.", self.Store.display());
					}
				},

				Err(Error) => {
					error!(
						target: "Build::Guard",


						"Restore FAILED: {}. {} is now inconsistent. Backup remains at {}.",


						Error,


						self.Path.display(),


						self.Store.display()
					)
				},
			}
		} else if self.Store.exists() {
			// Original might not have existed, or backup was not active
			warn!(
				target: "Build::Guard",


				"Found unexpected backup {} (original might not have existed or backup flag was false). Deleting... ",


				self.Store.display()
			);

			if let Err(Error) = fs::remove_file(&self.Store) {
				error!(target: "Build::Guard", "FAILED to delete unexpected backup: {}.", Error);
			} else {
				info!(target: "Build::Guard", "Deleted unexpected backup successfully.");
			}
		}

		info!(target: "Build::Guard", "Cleanup for {} finished.", self.Note);
	}
}

/// Modifies specific name fields within a TOML file.
///
/// Changes `package.name`, `package.default-run`, `lib.name`, and `bin.name`
/// entries from an old name to a new name.
///
/// # Parameters
/// - `File`: Path to the TOML file to modify.
/// - `Old`: The original name string to find and replace.
/// - `Current`: The new name string to replace with.
///
/// # Returns
/// `Ok(true)` if changes were made and written, `Ok(false)` if no changes were
/// needed or no matching names were found.
///
/// # Errors
/// Returns `Error::Io` for file read/write issues or `Error::Edit` for TOML
/// parsing/formatting issues.
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

					debug!(target: "Build::Toml", "Changed a bin.name entry");

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
		warn!(
			target: "Build::Toml",


			"Name '{}' not found in relevant sections of {}. No changes made to file.",


			Old,


			File.display()
		);

		Ok(false)
	}
}

/// Modifies root-level 'version', 'productName', and 'identifier' in a JSON or
/// JSON5 file.
///
/// # Parameters
/// - `File`: Path to the JSON/JSON5 file.
/// - `Product`: The new product name.
/// - `Id`: The new bundle identifier.
/// - `Version`: The new version string.
///
/// # Returns
/// `Ok(true)` if changes were made, `Ok(false)` otherwise.
///
/// # Errors
/// Returns `Error::Io` for file issues, `Error::Json` or `Error::Jsonfive` for
/// parsing/serialization issues, `Error::Utf` for string conversion issues, or
/// if the JSON root is not an object.
pub fn JsonEdit(File:&Path, Product:&str, Id:&str, Version:&str) -> Result<bool, Error> {
	debug!(target: "Build::Json", "Attempting to modify JSON file: {}", File.display());

	let Data = fs::read_to_string(File)?;

	let mut Parsed:JsonValue = match File.extension().and_then(|Ext| Ext.to_str()) {
		Some("json5") => json5::from_str(&Data)?,

		_ => serde_json::from_str(&Data)?,
	};

	debug!(target: "Build::Json", "Target Product: '{}', ID: '{}', Ver: '{}'", Product, Id, Version);

	let mut ModifiedItems = Vec::new();

	// Get root object, or fail if not an object
	let RootObj = Parsed.as_object_mut().ok_or_else(|| {
		error!(target: "Build::Json", "Root of JSON file {} is not an object.", File.display());

		Error::Io(io::Error::new(
			io::ErrorKind::InvalidData,
			format!("JSON root of {} is not an object", File.display()),
		))
	})?;

	// Version: Expected at root
	let VersionKey = "version";

	if let Some(VerVal) = RootObj.get_mut(VersionKey) {
		if VerVal.as_str() != Some(Version) {
			*VerVal = JsonValue::String(Version.to_string());

			ModifiedItems.push(VersionKey.to_string());

			debug!(target: "Build::Json", "Updated root '{}'", VersionKey);
		}
	} else {
		RootObj.insert(VersionKey.to_string(), JsonValue::String(Version.to_string()));

		ModifiedItems.push(format!("{} (created at root)", VersionKey));

		debug!(target: "Build::Json", "Created root '{}'", VersionKey);
	}

	// ProductName: Expected at root
	let ProductKey = "productName";

	if let Some(ProdVal) = RootObj.get_mut(ProductKey) {
		if ProdVal.as_str() != Some(Product) {
			*ProdVal = JsonValue::String(Product.to_string());

			ModifiedItems.push(ProductKey.to_string());

			debug!(target: "Build::Json", "Updated root '{}'", ProductKey);
		}
	} else {
		RootObj.insert(ProductKey.to_string(), JsonValue::String(Product.to_string()));

		ModifiedItems.push(format!("{} (created at root)", ProductKey));

		debug!(target: "Build::Json", "Created root '{}'", ProductKey);
	}

	// Identifier: Expected at root
	let IdKey = "identifier";

	if let Some(IdVal) = RootObj.get_mut(IdKey) {
		if IdVal.as_str() != Some(Id) {
			*IdVal = JsonValue::String(Id.to_string());

			ModifiedItems.push(IdKey.to_string());

			debug!(target: "Build::Json", "Updated root '{}'", IdKey);
		}
	} else {
		RootObj.insert(IdKey.to_string(), JsonValue::String(Id.to_string()));

		ModifiedItems.push(format!("{} (created at root)", IdKey));

		debug!(target: "Build::Json", "Created root '{}'", IdKey);
	}

	if !ModifiedItems.is_empty() {
		let mut Buffer = Vec::new();

		let Format = serde_json::ser::PrettyFormatter::with_indent(b"\t");

		let mut Serial = serde_json::Serializer::with_formatter(&mut Buffer, Format);

		Parsed.serialize(&mut Serial)?;

		let Output = String::from_utf8(Buffer)?;

		fs::write(File, Output)?;

		info!(
			target: "Build::Json",


			"Changed {} in {} (Product: '{}', ID: '{}', Ver: '{}')",


			ModifiedItems.join(", "), File.display(), Product, Id, Version
		);

		Ok(true)
	} else {
		info!(target: "Build::Json", "No JSON modifications needed for {}.", File.display());

		Ok(false)
	}
}

/// Converts a kebab-case or snake_case string to PascalCase.
///
/// # Parameters
/// - `Text`: The input string to convert.
///
/// # Returns
/// The PascalCase version of the input string.
pub fn Pascalize(Text:&str) -> String {
	Text.split(|Character:char| Character == '-' || Character == '_')
		.filter(|Part| !Part.is_empty())
		.map(|Part| {
			let mut Characters = Part.chars();

			match Characters.next() {
				None => String::new(),

				Some(FirstChar) => FirstChar.to_uppercase().to_string() + Characters.as_str(),
			}
		})
		.collect()
}

/// Converts a PascalCase string into a vector of its lowercase constituent
/// words.
///
/// Example: "MyExampleString" -> vec!["my", "example", "string"]
/// Example: "VSCode" -> vec!["vscode"]
/// Example: "NodeEnvironment" -> vec!["node", "environment"]
///
/// # Parameters
/// - `Text`: The PascalCase input string.
///
/// # Returns
/// A `Vec<String>` of lowercase words.
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
				// If current word is not empty and previous char was not uppercase,

				// this uppercase char starts a new word.
				Words.push(CurrentWord.to_ascii_lowercase());

				CurrentWord.clear();
			}

			CurrentWord.push(Char);

			LastCharWasUppercase = true;
		} else {
			// Lowercase or number
			if LastCharWasUppercase && Char.is_alphabetic() && !CurrentWord.ends_with(char::is_uppercase) {
				// If previous was uppercase and current is lowercase, and current word isn't
				// just an acronym This implies transition from acronym (like VS) to a new
				// word part (Code in VSCode) For simple PascalCase like "NodeEnvironment",

				// the previous if condition handles it. This handles cases like "VSCode" ->
				// "vscode", "MyID" -> "myid". If CurrentWord has multiple uppercase, it's
				// an acronym. If CurrentWord has one uppercase, and now we see a lowercase,

				// it's a new word. This part needs refinement if strict acronym handling is
				// desired. For "PascalToDotCase" -> "pascal", "to", "dot", "case"
				// A simpler approach: if current char is lowercase and previous was uppercase,

				// and current word contains more than just that previous uppercase char,

				// then the previous uppercase (and any before it if an acronym) was a word.
				if CurrentWord.chars().filter(|c| c.is_uppercase()).count() > 1 && CurrentWord.len() > 1 {

					// CurrentWord is an acronym like "VS", and now we have "C"
					// (lowercase 'c') We need to push "vs" and start "c".
					// This logic is getting complex. A regex might be better
					// for robust Pascal/camel to words. For now, let's
					// stick to a simpler split: any uppercase starts a new
					// potential word boundary.
				}
			}

			CurrentWord.push(Char);

			LastCharWasUppercase = false;
		}
	}

	if !CurrentWord.is_empty() {
		Words.push(CurrentWord.to_ascii_lowercase());
	}

	Words
}

/// Main orchestration logic for preparing and executing the build.
///
/// Modifies configuration files based on input arguments, then runs the
/// specified build command.
///
/// # Parameters
/// - `Argument`: Parsed command-line arguments and environment variables.
///
/// # Errors
/// Returns various `Error` variants if any step in the process fails.
pub fn Process(Argument:&Argument) -> Result<(), Error> {
	info!(target: "Build", "Starting build orchestration...");

	debug!(target: "Build", "Argument: {:?}", Argument);

	let ProjectDir = PathBuf::from(&Argument.Directory);

	if !ProjectDir.is_dir() {
		error!(target: "Build", "Project directory not found: {}", ProjectDir.display());

		return Err(Error::Missing(ProjectDir));
	}

	info!(target: "Build", "Using project directory: {}", ProjectDir.display());

	let CargoPath = ProjectDir.join(CargoFile);

	let JsonfivePath = ProjectDir.join(JsonfiveFile);

	let JsonPath = ProjectDir.join(JsonFile);

	let ConfigPath = if JsonfivePath.exists() {
		JsonfivePath
	} else if JsonPath.exists() {
		JsonPath
	} else {
		error!(target: "Build", "Neither {} nor {} found in {}", JsonfiveFile, JsonFile, ProjectDir.display());

		return Err(Error::Config);
	};

	info!(target: "Build", "Using Tauri config: {}", ConfigPath.display());

	let CargoGuard = Guard::New(CargoPath.clone(), "Cargo.toml".to_string())?;

	let ConfigGuard = Guard::New(ConfigPath.clone(), "Tauri config".to_string())?;

	// --- Name and Identifier Parts Construction ---
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

				debug!(target: "Build::Name", "Added NodeEnvironment parts");
			}
		}
	}

	if let Some(DependencyValue) = &Argument.Dependency {
		if !DependencyValue.is_empty() {
			let (PascalDepBase, IdDepWords) = if DependencyValue.eq_ignore_ascii_case("true") {
				("Generic".to_string(), vec!["generic".to_string()])
			} else if let Some((Org, Repo)) = DependencyValue.split_once('/') {
				let PascalOrg = Pascalize(Org);

				let PascalRepo = Pascalize(Repo);

				let Base = format!("{}{}", PascalOrg, PascalRepo);

				let mut Words = Vec::new();

				Words.extend(WordsFromPascal(&PascalOrg));

				Words.extend(WordsFromPascal(&PascalRepo));

				(Base, Words)
			} else {
				let PascalVal = Pascalize(DependencyValue);

				(PascalVal.clone(), WordsFromPascal(&PascalVal))
			};

			if !PascalDepBase.is_empty() {
				NamePartsForProductName.push(format!("{}Dependency", PascalDepBase));

				NamePartsForId.extend(IdDepWords);

				NamePartsForId.push("dependency".to_string());

				debug!(target: "Build::Name", "Added Dependency parts");
			}
		}
	}

	if Argument.Bundle.as_ref().map_or(false, |V| V.eq_ignore_ascii_case("true")) {
		NamePartsForProductName.push("Bundle".to_string());

		NamePartsForId.push("bundle".to_string());

		debug!(target: "Build::Name", "Added Bundle parts");
	}

	if Argument.Clean.as_ref().map_or(false, |V| V.eq_ignore_ascii_case("true")) {
		NamePartsForProductName.push("Clean".to_string());

		NamePartsForId.push("clean".to_string());

		debug!(target: "Build::Name", "Added Clean parts");
	}

	if Argument.Browser.as_ref().map_or(false, |V| V.eq_ignore_ascii_case("true")) {
		NamePartsForProductName.push("Browser".to_string());

		NamePartsForId.push("browser".to_string());

		debug!(target: "Build::Name", "Added Browser parts");
	}

	if Argument.Compile.as_ref().map_or(false, |V| V.eq_ignore_ascii_case("true")) {
		NamePartsForProductName.push("Compile".to_string());

		NamePartsForId.push("compile".to_string());

		debug!(target: "Build::Name", "Added Compile parts");
	}

	// --- Construct FinalName for Product/Package ---
	let ProductNamePrefix = NamePartsForProductName.join(NameDelimiter);

	debug!(target: "Build", "Full prefix string for product name: '{}'", ProductNamePrefix);

	let FinalName = if !ProductNamePrefix.is_empty() {
		format!("{}{}{}", ProductNamePrefix, NameDelimiter, Argument.Name)
	} else {
		Argument.Name.clone()
	};

	info!(target: "Build", "Final generated package/product name: '{}'", FinalName);

	// --- Construct FinalId for Bundle Identifier ---
	// Add the base name ("Mountain") to the Id parts
	NamePartsForId.extend(WordsFromPascal(&Argument.Name));

	let IdSuffix = NamePartsForId
		.into_iter()
		.filter(|s| !s.is_empty())
		.collect::<Vec<String>>()
		.join(IdDelimiter);

	debug!(target: "Build", "Generated dot.separated suffix for identifier: '{}'", IdSuffix);

	let FinalId = format!("{}{}{}", Argument.Prefix, IdDelimiter, IdSuffix);

	info!(target: "Build", "Generated bundle identifier: '{}'", FinalId);

	// --- TOML and JSON Modification ---
	if FinalName != Argument.Name {
		TomlEdit(CargoGuard.Path(), &Argument.Name, &FinalName)?;
	} else {
		info!(target: "Build", "Cargo.toml name remains '{}'.", Argument.Name);
	}

	let AppVersion = {
		let VersionFile = if CargoGuard.Store().exists() { CargoGuard.Store() } else { CargoGuard.Path() };

		debug!(target: "Build", "Reading version from: {}", VersionFile.display());

		let VersionData = fs::read_to_string(VersionFile)?;

		let CargoManifest:Manifest = toml::from_str(&VersionData).map_err(|Error| {
			error!(target: "Build", "Failed to parse TOML for version from {}: {}", VersionFile.display(), Error);

			Error::Parse(Error)
		})?;

		debug!(target: "Build", "Read version: {}", CargoManifest.package.version);

		CargoManifest.package.version
	};

	let JsonProduct = &FinalName;

	JsonEdit(ConfigGuard.Path(), JsonProduct, &FinalId, &AppVersion)?;

	// --- Command Execution ---
	if Argument.Command.is_empty() {
		error!(target: "Build", "No build command provided.");

		return Err(Error::Nocommand);
	}

	let mut ShellCommand:ProcessCommand;

	let Program = &Argument.Command[0];

	let ProgramArgument = &Argument.Command[1..];

	if cfg!(target_os = "windows") {
		ShellCommand = ProcessCommand::new("cmd");

		ShellCommand.arg("/C").arg(Program).args(ProgramArgument);
	} else {
		let mut FullCommand = Program.clone();

		for Arg in ProgramArgument {
			FullCommand.push(' ');

			FullCommand.push_str(&format!("'{}'", Arg.replace('\'', "'\\''")));
		}

		ShellCommand = ProcessCommand::new("sh");

		ShellCommand.arg("-c").arg(FullCommand);
	}

	info!(target: "Build::Exec", "Executing command (shell wrapper): {:?}", ShellCommand);

	let mut ProcessHandle = ShellCommand
		.current_dir(env::current_dir()?)
		.stdout(Stdio::inherit())
		.stderr(Stdio::inherit())
		.spawn()
		.map_err(|Error| {
			error!(target: "Build::Exec", "Failed to spawn command '{:?}': {}", ShellCommand, Error);

			Error::Io(Error)
		})?;

	let ExitStatus = ProcessHandle.wait()?;

	info!(target: "Build::Exec", "Command finished with status: {}", ExitStatus);

	if !ExitStatus.success() {
		error!(target: "Build::Exec", "Command failed with status: {}", ExitStatus);

		return Err(Error::Shell(ExitStatus));
	}

	info!(target: "Build", "Build orchestration completed successfully.");

	Ok(())
}

/// Sets up the global logger for the application.
///
/// Configures `env_logger` with a custom format and uses the `RUST_LOG`
/// environment variable (defaulting to "info") to control verbosity.
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

	info!(
		"Logger initialized with level: {} (from {} or default 'info')",
		LevelText,
		LogEnv.cyan()
	);
}

/// Verifies if all required environment variables are set.
///
/// Currently, `clap` handles defaults and requirements, so this is a
/// placeholder.
///
/// # Errors
/// Could return `Error::Environment` if a critical variable were missing.
pub fn VerifyEnv() -> Result<(), Error> { Ok(()) }

/// Entry point for running the build script logic.
///
/// Initializes logging, parses arguments, and orchestrates the build process.
pub fn Fn() {
	Logger();

	if let Err(Error) = VerifyEnv() {
		error!("Failed environment variable check: {}", Error);

		std::process::exit(1);
	}

	let Argument = Argument::parse();

	debug!("Parsed arguments: {:?}", Argument);

	match Process(&Argument) {
		Ok(_) => {
			info!("Build process completed successfully.");
		},

		Err(Failure) => {
			error!("Build process failed: {}", Failure);

			std::process::exit(1);
		},
	}
}

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
