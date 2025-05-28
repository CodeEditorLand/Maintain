#![allow(non_snake_case)]

use std::{
	env,
	fs,
	io::{self},
	path::{Path, PathBuf},
	process::{Command, Stdio},
};

use clap::{self, Parser};
use serde::{Deserialize, Serialize};
use serde_json::Value as Jsonvalue;
use thiserror::Error;
use toml_edit::{DocumentMut as Tomldocument, Item as Tomlitem, Value as Tomlvalue};

const MOUNTAIN_DIR_DEFAULT:&str = "Element/Mountain";

const ORIGINAL_BASE_NAME_DEFAULT:&str = "Mountain";

const BUNDLE_ID_PREFIX_DEFAULT:&str = "land.editor.binary";

const CARGO_TOML_FILENAME:&str = "Cargo.toml";

const TAURI_CONF_JSON5_FILENAME:&str = "tauri.conf.json5";

const TAURI_CONF_JSON_FILENAME:&str = "tauri.conf.json";

const BACKUP_SUFFIX:&str = ".Backup";

#[derive(Error, Debug)]
enum Failure {
	#[error("IO: {0}")]
	Io(#[from] io::Error),

	#[error("TomlEdit: {0}")]
	TomlEdit(#[from] toml_edit::TomlError),

	#[error("TomlDe: {0}")]
	TomlDe(#[from] toml::de::Error),

	#[error("Json: {0}")]
	Json(#[from] serde_json::Error),

	#[error("JsonFive: {0}")]
	JsonFive(#[from] json5::Error),

	#[error("MissingDir: {0}")]
	MissingDir(PathBuf),

	#[error("CommandFailed: {0}")]
	CommandFailed(std::process::ExitStatus),

	#[error("NoCommand")]
	NoCommand,

	#[error("TauriConfigPath")]
	TauriConfigPath,

	#[error("BackupExists: {0}")]
	BackupExists(PathBuf),

	#[error("UtfEight: {0}")]
	UtfEight(#[from] std::string::FromUtf8Error),
}

#[derive(Parser, Debug)]
#[clap(author, version, about = "Prepares, builds, and restores Mountain project.")]
struct Argument {
	#[clap(long, env = "LBT_MOUNTAIN_DIR", default_value = MOUNTAIN_DIR_DEFAULT)]
	DirectoryMountain:String,

	#[clap(long, env = "LBT_ORIGINAL_BASE_NAME", default_value = ORIGINAL_BASE_NAME_DEFAULT)]
	OriginalName:String,

	#[clap(long, env = "LBT_BUNDLE_ID_PREFIX", default_value = BUNDLE_ID_PREFIX_DEFAULT)]
	BundlePrefix:String,

	#[clap(long, env = "Browser")]
	Browser:Option<String>,

	#[clap(long, env = "Bundle")]
	Bundle:Option<String>,

	#[clap(long, env = "Clean")]
	Clean:Option<String>,

	#[clap(long, env = "Dependency")]
	Dependency:Option<String>,

	#[clap(long, env = "NODE_ENV")]
	NodeEnvironment:Option<String>,

	#[clap(required = true, last = true)]
	BuildCommand:Vec<String>,
}

#[derive(Deserialize)]
struct Manifest {
	package:Metadatapackage,
}

#[derive(Deserialize)]
struct Metadatapackage {
	version:String,
}

struct Fileguard {
	PathOriginal:PathBuf,

	PathBackup:PathBuf,

	Backup:bool,

	Description:String,
}

impl Fileguard {
	fn New(PathOriginal:PathBuf, Description:String) -> Result<Self, Failure> {
		let PathBackup = PathOriginal.with_extension(format!(
			"{}{}",
			PathOriginal.extension().unwrap_or_default().to_str().unwrap_or(""),
			BACKUP_SUFFIX
		));

		if PathBackup.exists() {
			return Err(Failure::BackupExists(PathBackup));
		}

		let mut Backup = false;

		if PathOriginal.exists() {
			fs::copy(&PathOriginal, &PathBackup)?;

			println!("Maintain: Guard: Backed {} to {}", PathOriginal.display(), PathBackup.display());

			Backup = true;
		} else {
			println!("Maintain: Guard: Original {} not found, no backup.", PathOriginal.display());
		}

		Ok(Self { PathOriginal, PathBackup, Backup, Description })
	}

	fn Original(&self) -> &Path { &self.PathOriginal }

	fn Backup(&self) -> &Path { &self.PathBackup }
}

impl Drop for Fileguard {
	fn drop(&mut self) {
		if self.Backup && self.PathBackup.exists() {
			print!(
				"Maintain: Guard: Restoring {} from {}... ",
				self.PathOriginal.display(),
				self.PathBackup.display()
			);

			match fs::copy(&self.PathBackup, &self.PathOriginal) {
				Ok(_) => {
					println!("Done.");

					if let Err(Error) = fs::remove_file(&self.PathBackup) {
						eprintln!("Maintain: Guard: Failed delete backup {}: {}", self.PathBackup.display(), Error);
					} else {
						println!("Maintain: Guard: Deleted backup {}.", self.PathBackup.display());
					}
				},

				Err(Error) => {
					eprintln!(
						"FAILED: {}. {} inconsistent. Backup at {}.",
						Error,
						self.PathOriginal.display(),
						self.PathBackup.display()
					)
				},
			}
		} else if self.PathBackup.exists() {
			print!(
				"Maintain: Guard: Found unexpected backup {}. Deleting... ",
				self.PathBackup.display()
			);

			if let Err(Error) = fs::remove_file(&self.PathBackup) {
				eprintln!("FAILED to delete: {}.", Error);
			} else {
				println!("Done.");
			}
		}

		println!("Maintain: Guard: Cleanup for {} done.", self.Description);
	}
}

fn TomlModify(Path:&Path, Original:&str, New:&str) -> Result<bool, Failure> {
	let Content = fs::read_to_string(Path)?;

	if Original == New {
		return Ok(false);
	}

	let mut Document:Tomldocument = Content.parse()?;

	let mut PackageChanged = false;

	let mut LibChanged = false;

	let mut BinChanged = false;

	let mut DefaultRunUnchanged = false;

	if let Some(PackageTable) = Document.get_mut("package").and_then(|Item| Item.as_table_mut()) {
		if let Some(Nameitem) = PackageTable.get_mut("name") {
			if Nameitem.as_str() == Some(Original) {
				*Nameitem = Tomlitem::Value(Tomlvalue::String(toml_edit::Formatted::new(New.to_string())));

				PackageChanged = true;
			}
		}

		if let Some(RunItem) = PackageTable.get_mut("default-run") {
			if RunItem.as_str() == Some(Original) {
				*RunItem = Tomlitem::Value(Tomlvalue::String(toml_edit::Formatted::new(New.to_string())));

				DefaultRunUnchanged = true;
			}
		}
	}

	if let Some(LibTable) = Document.get_mut("lib").and_then(|Item| Item.as_table_mut()) {
		if let Some(Nameitem) = LibTable.get_mut("name") {
			if Nameitem.as_str() == Some(Original) {
				*Nameitem = Tomlitem::Value(Tomlvalue::String(toml_edit::Formatted::new(New.to_string())));

				LibChanged = true;
			}
		}
	}

	if let Some(BinTable) = Document.get_mut("bin").and_then(|Item| Item.as_array_of_tables_mut()) {
		for Table in BinTable.iter_mut() {
			if let Some(Nameitem) = Table.get_mut("name") {
				if Nameitem.as_str() == Some(Original) {
					*Nameitem = Tomlitem::Value(Tomlvalue::String(toml_edit::Formatted::new(New.to_string())));

					BinChanged = true;

					break;
				}
			}
		}
	}

	if PackageChanged || LibChanged || BinChanged {
		let Tomlstring = Document.to_string();

		fs::write(Path, Tomlstring)?;

		let mut Parts = Vec::new();

		if PackageChanged {
			Parts.push("package.name");
		}

		if DefaultRunUnchanged {
			Parts.push("package.default-run");
		}

		if LibChanged {
			Parts.push("lib.name");
		}

		if BinChanged {
			Parts.push("bin.name");
		}

		println!("Maintain: Temp changed {} in {} to: {}", Parts.join(", "), Path.display(), New);

		Ok(true)
	} else {
		if Original != New {
			println!(
				"Maintain: Name '{}' not found in {} sections of {}.",
				Original,
				"package, lib, or bin",
				Path.display()
			);
		}

		Ok(false)
	}
}

fn JSONModify(Path:&Path, Product:&str, Identifier:&str, Version:&str) -> Result<bool, Failure> {
	let Content = fs::read_to_string(Path)?;

	let mut Json:Jsonvalue = match Path.extension().and_then(|Ext| Ext.to_str()) {
		Some("json5") => json5::from_str(&Content)?,

		_ => serde_json::from_str(&Content)?,
	};

	let mut Changed = Vec::new();

	let Verfield = "version";

	if let Some(V) = Json.get_mut(Verfield) {
		if V.as_str() != Some(Version) {
			*V = Jsonvalue::String(Version.to_string());

			Changed.push(Verfield.to_string());
		}
	} else {
		if let Some(Obj) = Json.as_object_mut() {
			Obj.insert(Verfield.to_string(), Jsonvalue::String(Version.to_string()));

			Changed.push(format!("{} (created)", Verfield));
		}
	}

	let FieldProduct = "productName";

	if let Some(P) = Json.get_mut(FieldProduct) {
		if P.as_str() != Some(Product) {
			*P = Jsonvalue::String(Product.to_string());

			Changed.push(FieldProduct.to_string());
		}
	} else {
		if let Some(Object) = Json.as_object_mut() {
			Object.insert(FieldProduct.to_string(), Jsonvalue::String(Product.to_string()));

			Changed.push(format!("{} (created)", FieldProduct));
		}
	}

	let FieldIdentifier = "identifier";

	if let Some(I) = Json.get_mut(FieldIdentifier) {
		if I.as_str() != Some(Identifier) {
			*I = Jsonvalue::String(Identifier.to_string());

			Changed.push(FieldIdentifier.to_string());
		}
	} else {
		if let Some(Object) = Json.as_object_mut() {
			Object.insert(FieldIdentifier.to_string(), Jsonvalue::String(Identifier.to_string()));

			Changed.push(format!("{} (created)", FieldIdentifier));
		}
	}

	if !Changed.is_empty() {
		let mut Writer = Vec::new();

		let Formatter = serde_json::ser::PrettyFormatter::with_indent(b"\t");

		let mut Serializer = serde_json::Serializer::with_formatter(&mut Writer, Formatter);

		Json.serialize(&mut Serializer)?;

		let ContentNew = String::from_utf8(Writer)?;

		fs::write(Path, ContentNew)?;

		println!(
			"Maintain: Temp changed {} in {} (Product: {}, ID: {}, Ver: {})",
			Changed.join(", "),
			Path.display(),
			Product,
			Identifier,
			Version
		);

		Ok(true)
	} else {
		Ok(false)
	}
}

fn Orchestrate(Argument:Argument) -> Result<(), Failure> {
	let PathMountain = PathBuf::from(&Argument.DirectoryMountain);

	if !PathMountain.is_dir() {
		return Err(Failure::MissingDir(PathMountain));
	}

	let PathToml = PathMountain.join(CARGO_TOML_FILENAME);

	let PathJSON5 = PathMountain.join(TAURI_CONF_JSON5_FILENAME);

	let PathJSON = PathMountain.join(TAURI_CONF_JSON_FILENAME);

	let Jsonactualpath = if PathJSON5.exists() {
		PathJSON5
	} else if PathJSON.exists() {
		PathJSON
	} else {
		return Err(Failure::TauriConfigPath);
	};

	let GuardToml = Fileguard::New(PathToml.clone(), "Cargo.toml".to_string())?;

	let GuardJSON = Fileguard::New(Jsonactualpath.clone(), "Tauri config".to_string())?;

	let mut Suffix:Vec<String> = Vec::new();

	if Argument.Browser.map_or(false, |V| V.eq_ignore_ascii_case("true")) {
		Suffix.push("Br".to_string());
	}

	if Argument.Bundle.map_or(false, |V| V.eq_ignore_ascii_case("true")) {
		Suffix.push("Bu".to_string());
	}

	if Argument.Clean.map_or(false, |V| V.eq_ignore_ascii_case("true")) {
		Suffix.push("Cl".to_string());
	}

	if let Some(ValueDependency) = Argument.Dependency {
		if !ValueDependency.is_empty() {
			if let Some((Organization, Repository)) = ValueDependency.split_once('/') {
				if let (Some(IndexOrganization), Some(IndexRepository)) = (
					Organization
						.chars()
						.next()
						.map(|C| C.to_uppercase().to_string())
						.filter(|S| !S.is_empty()),
					Repository
						.chars()
						.next()
						.map(|C| C.to_uppercase().to_string())
						.filter(|S| !S.is_empty()),
				) {
					Suffix.push(format!("Dp{}{}", IndexOrganization, IndexRepository));
				}
			}
		}
	}

	if let Some(ValueNode) = Argument.NodeEnvironment {
		let Code = match ValueNode.to_lowercase().as_str() {
			"production" => Some("P"),

			"development" => Some("D"),

			"test" => Some("T"),

			_ => None,
		};

		if let Some(Code) = Code {
			Suffix.push(format!("Ne{}", Code));
		}
	}

	let Suffix = Suffix.join("");

	let Newname = if !Suffix.is_empty() {
		format!("{}{}", Argument.OriginalName, Suffix)
	} else {
		Argument.OriginalName.clone()
	};

	if Newname != Argument.OriginalName {
		TomlModify(GuardToml.Original(), &Argument.OriginalName, &Newname)?;
	} else {
		println!("Maintain: No suffix for Cargo.toml, remaining '{}'.", Argument.OriginalName);
	}

	let Version = {
		let Content = if GuardToml.Backup().exists() {
			fs::read_to_string(GuardToml.Backup())?
		} else {
			fs::read_to_string(GuardToml.Original())?
		};

		let M:Manifest = toml::from_str(&Content)?;

		M.package.version
	};

	let Slug = Newname
		.to_lowercase()
		.replace(['_', '-'], "")
		.chars()
		.filter(|C| C.is_alphanumeric())
		.collect::<String>();

	let Generatedidentifier = format!("{}.{}", Argument.BundlePrefix, Slug);

	let Productnameforjson = &Newname;

	JSONModify(GuardJSON.Original(), Productnameforjson, &Generatedidentifier, &Version)?;

	if Argument.BuildCommand.is_empty() {
		return Err(Failure::NoCommand);
	}

	let mut ExecuteCommand:Command;

	let NameProgram = &Argument.BuildCommand[0];

	let ArgumentProgram = &Argument.BuildCommand[1..];

	if cfg!(target_os = "windows") {
		ExecuteCommand = Command::new("cmd");

		ExecuteCommand.arg("/C");

		ExecuteCommand.arg(NameProgram);

		ExecuteCommand.args(ArgumentProgram);
	} else {
		let mut CommandFull = NameProgram.clone();

		for Arg in ArgumentProgram {
			CommandFull.push(' ');

			CommandFull.push('\'');

			CommandFull.push_str(&Arg.replace('\'', "'\\''"));

			CommandFull.push('\'');
		}

		ExecuteCommand = Command::new("sh");

		ExecuteCommand.arg("-c");

		ExecuteCommand.arg(CommandFull);
	}

	println!("Maintain: --- Executing (Shell Wrapper): {:?} ---", ExecuteCommand);

	let mut Child = ExecuteCommand
		.current_dir(env::current_dir()?)
		.stdout(Stdio::inherit())
		.stderr(Stdio::inherit())
		.spawn()?;

	let Status = Child.wait()?;

	println!("Maintain: --- Command Finished (status: {}) ---", Status);

	if !Status.success() {
		return Err(Failure::CommandFailed(Status));
	}

	Ok(())
}

pub fn Fn() {
	match Orchestrate(Argument::parse()) {
		Ok(_) => {
			println!("Maintain: Process completed.");
		},

		Err(Error) => {
			eprintln!("Maintain Failure: {}", Error);

			std::process::exit(1);
		},
	}
}
