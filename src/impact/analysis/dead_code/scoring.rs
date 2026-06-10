use super::*;

impl<'a> ConfidenceScorer<'a> {
    /// Score a single symbol. Returns `None` if the symbol is an entrypoint itself
    /// or if the confidence is below the threshold.
    pub fn score_symbol(
        &self,
        symbol: &Symbol,
        file_path: &Path,
    ) -> Result<Option<DeadCodeFinding>> {
        if filters::is_entrypoint(symbol) {
            return Ok(None);
        }

        let reachability = self.reachability_score(symbol, file_path)?;
        let git_activity = self.git_activity_score(file_path)?;
        let test_coverage = self.test_coverage_score(symbol, file_path)?;

        let confidence = self.blend(reachability, git_activity, test_coverage);

        if confidence < self.config.confidence_threshold {
            return Ok(None);
        }

        let mut factors = Vec::new();
        if reachability >= 1.0 {
            factors.push(ConfidenceFactor::UnreachableFromEntrypoints);
        }
        if git_activity > 0.0 {
            let days = self
                .days_since_last_commit(file_path)?
                .unwrap_or(self.config.git_inactivity_days);
            factors.push(ConfidenceFactor::GitInactive {
                days_since_last_commit: days,
            });
        }
        if test_coverage >= 1.0 {
            factors.push(ConfidenceFactor::NoTestCoverage);
        }

        let mut recommendation = format!(
            "Symbol '{}' in {} has {:.0}% confidence of being dead code. Consider reviewing for removal or adding tests.",
            symbol.name,
            file_path.display(),
            confidence * 100.0
        );

        if let Some(cfg) = symbol.metadata.get("cfg") {
            recommendation.push_str(&format!(" Note: symbol is feature-gated via {}.", cfg));
        }

        Ok(Some(DeadCodeFinding {
            symbol_name: symbol.name.clone(),
            file_path: file_path.to_path_buf(),
            confidence,
            factors,
            recommendation,
        }))
    }

    /// Score all symbols in a file.
    pub fn score_file(&self, file_path: &Path) -> Result<Vec<DeadCodeFinding>> {
        let symbols = self.get_symbols_for_file(file_path)?;
        let mut findings = Vec::new();
        for symbol in symbols {
            if let Some(finding) = self.score_symbol(&symbol, file_path)? {
                findings.push(finding);
            }
        }
        findings.sort_unstable();
        Ok(findings)
    }

    /// Full-repo scan (used by the standalone `dead-code` command).
    pub fn scan_repo(&self, limit: usize) -> Result<Vec<DeadCodeFinding>> {
        let symbols = self.get_all_symbols()?;
        let mut findings = Vec::new();
        for (symbol, file_path) in symbols {
            if let Some(finding) = self.score_symbol(&symbol, &file_path)? {
                findings.push(finding);
                if findings.len() >= limit {
                    break;
                }
            }
        }
        findings.sort_unstable();
        Ok(findings)
    }

    pub(super) fn blend(&self, reachability: f64, git_activity: f64, test_coverage: f64) -> f64 {
        let sum = self.config.reachability_weight
            + self.config.git_activity_weight
            + self.config.test_coverage_weight;
        if sum <= 0.0 {
            return 0.0;
        }
        (self.config.reachability_weight * reachability
            + self.config.git_activity_weight * git_activity
            + self.config.test_coverage_weight * test_coverage)
            / sum
    }
}
