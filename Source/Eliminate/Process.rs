//=============================================================================//
// File Path: Element/Maintain/Source/Eliminate/Process.rs
//=============================================================================//
// Module: Process - File discovery and transformation orchestration
//=============================================================================//

use std::{
	fs,
	path::{Path, PathBuf},
};

use globset::{Glob, GlobSet, GlobSetBuilder};
use walkdir::WalkDir;

use super::{Definition, Error, Transform};

/// Discover Rust source files matching `Pattern` under `Root`, run the
/// elimination transform on each, and write back the result unless
/// `Options.DryRun` is set.
///
/// Returns aggregate [`Definition::Stats`] describing what was processed.
pub fn Process(
	Root: &Path,
	Pattern: &str,
	Options: &Definition::Options,
) -> Error::Result<Definition::Stats> {
	let mut Stats = Definition::Stats::default();

	let GlobMatcher = BuildGlobSet(Pattern)?;

	let Files = CollectFiles(Root, &GlobMatcher);

	for FilePath in Files {
		Stats.FilesProcessed += 1;

		ProcessFile(&FilePath, Options, &mut Stats)?;
	}

	Ok(Stats)
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

fn BuildGlobSet(Pattern: &str) -> Error::Result<GlobSet> {
	let mut Builder = GlobSetBuilder::new();

	Builder.add(Glob::new(Pattern)?);

	Ok(Builder.build()?)
}

fn CollectFiles(Root: &Path, GlobMatcher: &GlobSet) -> Vec<PathBuf> {
	if Root.is_file() {
		return vec![Root.to_path_buf()];
	}

	WalkDir::new(Root)
		.follow_links(false)
		.into_iter()
		.filter_map(|Entry| Entry.ok())
		.filter(|Entry| Entry.file_type().is_file())
		.filter(|Entry| {
			let RelativePath = Entry.path().strip_prefix(Root).unwrap_or(Entry.path());

			GlobMatcher.is_match(RelativePath)
		})
		.map(|Entry| Entry.path().to_path_buf())
		.collect()
}

fn ProcessFile(
	FilePath: &Path,
	Options: &Definition::Options,
	Stats: &mut Definition::Stats,
) -> Error::Result<()> {
	let Source = fs::read_to_string(FilePath)?;

	let TransformResult = Transform::Run(&Source, Options).map_err(|E| {
		if let Error::Error::Parse { Source: Src, .. } = E {
			Error::Error::Parse {
				Path: FilePath.display().to_string(),
				Source: Src,
			}
		} else {
			E
		}
	})?;

	let Some(Transformed) = TransformResult else {
		return Ok(());
	};

	Stats.FilesModified += 1;

	// Count bindings inlined: count how many fewer `let ` lines the output has.
	let Before = Source.lines().filter(|L| L.trim_start().starts_with("let ")).count();

	let After = Transformed.lines().filter(|L| L.trim_start().starts_with("let ")).count();

	Stats.BindingsInlined += Before.saturating_sub(After);

	if Options.Verbose {
		log::info!(
			"{}: {} binding(s) inlined",
			FilePath.display(),
			Before.saturating_sub(After)
		);
	}

	if Options.DryRun {
		log::info!("--- {} (dry-run, not written) ---\n{}", FilePath.display(), Transformed);
	} else {
		fs::write(FilePath, Transformed)?;
	}

	Ok(())
}
